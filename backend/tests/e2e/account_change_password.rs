use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use reqwest::{Client, Response};
use secrecy::SecretString;
use serde_json::Value;
use sha1::{Digest, Sha1};
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::{hash_password, PasswordBlocklist},
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

fn sha1_hash(password: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

async fn connect_database() -> PgPool {
    let database_url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("connect to test database")
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    status: &str,
    deleted_at: Option<OffsetDateTime>,
    password: &str,
) -> Uuid {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (status, deleted_at)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .fetch_one(pool)
    .await
    .expect("seed account");

    let password_hash =
        hash_password(SecretString::from(password.to_owned())).expect("hash seed password");

    sqlx::query(
        r#"
        INSERT INTO account_credentials (
            account_id,
            email,
            email_normalized,
            password_hash
        )
        VALUES ($1, $2, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("seed account credentials");

    account_id
}

async fn seed_principal(pool: &PgPool, role_name: &str) -> (Uuid, String) {
    let account_id = seed_account(
        pool,
        &format!("e2e-principal-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "principal seed password with enough length",
    )
    .await;

    let session_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO authentication_session (
            account_id,
            expires_at,
            last_authenticated_at
        )
        VALUES ($1, CURRENT_TIMESTAMP + INTERVAL '1 hour', CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("seed authentication session");

    let token = format!(
        "e2e-change-password-{}-{}-{}",
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4()
    );

    sqlx::query(
        r#"
        INSERT INTO authentication_access_token (
            session_id,
            token_hash,
            expires_at
        )
        VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '1 hour')
        "#,
    )
    .bind(session_id)
    .bind(sha256_token_verifier(&token))
    .execute(pool)
    .await
    .expect("seed access token");

    let role_id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM authorization_role WHERE name = $1")
            .bind(role_name)
            .fetch_one(pool)
            .await
            .expect("find authorization role");

    sqlx::query(
        r#"
        INSERT INTO authorization_account_role (
            account_id,
            role_id,
            created_by
        )
        VALUES ($1, $2, $1)
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .execute(pool)
    .await
    .expect("seed authorization role");

    (account_id, token)
}

async fn cleanup_accounts(pool: &PgPool, account_ids: &[Uuid]) {
    if account_ids.is_empty() {
        return;
    }

    let account_ids = account_ids.to_vec();

    sqlx::query("DELETE FROM authorization_account_role WHERE account_id = ANY($1)")
        .bind(&account_ids)
        .execute(pool)
        .await
        .expect("cleanup authorization roles");

    sqlx::query("DELETE FROM authentication_session WHERE account_id = ANY($1)")
        .bind(&account_ids)
        .execute(pool)
        .await
        .expect("cleanup authentication sessions");

    sqlx::query("DELETE FROM account WHERE id = ANY($1)")
        .bind(&account_ids)
        .execute(pool)
        .await
        .expect("cleanup accounts");
}

fn test_router(pool: PgPool) -> Router {
    build_router(AppState::new(
        pool,
        Arc::new(PasswordBlocklist::from_hashes(
            "test",
            [sha1_hash("password-password")],
        )),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(app: Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind e2e server");

    let address = listener.local_addr().expect("read e2e server address");

    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve e2e application");
    });

    (address, task)
}

async fn assert_problem(response: Response, expected_status: reqwest::StatusCode, code: &str) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/problem+json")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("no-store")
    );

    let payload: Value = response.json().await.expect("decode problem response");
    assert_eq!(payload["code"], code);
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn change_password_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!().run(&pool).await.expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;
    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "old secure password with enough length",
    )
    .await;

    let (address, server) = start_server(test_router(pool.clone())).await;
    let client = Client::new();

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}/password"
        ))
        .bearer_auth(&admin_token)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "password": "new secure password with enough length"
        }))
        .send()
        .await
        .expect("send change-password request");

    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("no-store")
    );
    assert!(response.text().await.expect("read empty response").is_empty());

    let stored_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM account_credentials WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read password hash");

    assert!(stored_hash.starts_with("$argon2id$v=19$"));

    server.abort();
    cleanup_accounts(&pool, &[target_id, admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn change_password_authentication_authorization_and_validation_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!().run(&pool).await.expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;
    let (viewer_id, viewer_token) = seed_principal(&pool, "account_viewer").await;

    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "old secure password with enough length",
    )
    .await;

    let (address, server) = start_server(test_router(pool.clone())).await;
    let client = Client::new();
    let uri = format!("http://{address}/admin/accounts/{target_id}/password");

    let unauthenticated = client
        .patch(&uri)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "password": "new secure password with enough length"
        }))
        .send()
        .await
        .expect("send unauthenticated request");

    assert_eq!(
        unauthenticated.status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        unauthenticated
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok()),
        Some(r#"Bearer realm="admin-api""#)
    );

    let forbidden = client
        .patch(&uri)
        .bearer_auth(&viewer_token)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "password": "new secure password with enough length"
        }))
        .send()
        .await
        .expect("send forbidden request");

    assert_problem(forbidden, reqwest::StatusCode::FORBIDDEN, "FORBIDDEN").await;

    let invalid = client
        .patch(&uri)
        .bearer_auth(&admin_token)
        .header("content-type", "application/json")
        .json(&serde_json::json!({"password":"too-short"}))
        .send()
        .await
        .expect("send invalid request");

    assert_problem(
        invalid,
        reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        "PASSWORD_POLICY_VIOLATION",
    )
    .await;

    server.abort();
    cleanup_accounts(&pool, &[target_id, viewer_id, admin_id]).await;
}