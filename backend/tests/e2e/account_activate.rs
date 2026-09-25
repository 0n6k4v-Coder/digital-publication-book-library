use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use reqwest::{Client, Response};
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

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
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
) -> Uuid {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (status, deleted_at, created_by, updated_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .bind(created_by)
    .bind(updated_by)
    .fetch_one(pool)
    .await
    .expect("seed account");

    sqlx::query(
        r#"
        INSERT INTO account_credentials (
            account_id, email, email_normalized, password_hash
        )
        VALUES ($1, $2, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("seed account credentials");

    account_id
}

async fn assign_role(pool: &PgPool, account_id: Uuid, role_name: &str) {
    let role_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authorization_role WHERE name = $1",
    )
    .bind(role_name)
    .fetch_one(pool)
    .await
    .expect("find authorization role");

    sqlx::query(
        r#"
        INSERT INTO authorization_account_role (account_id, role_id, created_by)
        VALUES ($1, $2, $1)
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .execute(pool)
    .await
    .expect("assign authorization role");
}

async fn seed_principal(pool: &PgPool, role_name: &str) -> (Uuid, String) {
    let account_id = seed_account(
        pool,
        &format!("e2e-principal-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        None,
        None,
    )
    .await;

    let session_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO authentication_session (
            account_id, expires_at, last_authenticated_at
        )
        VALUES ($1, CURRENT_TIMESTAMP + INTERVAL '1 hour', CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("seed authentication session");

    let token = format!("e2e-activate-{}-{}", Uuid::new_v4(), Uuid::new_v4());

    sqlx::query(
        r#"
        INSERT INTO authentication_access_token (session_id, token_hash, expires_at)
        VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '1 hour')
        "#,
    )
    .bind(session_id)
    .bind(sha256_token_verifier(&token))
    .execute(pool)
    .await
    .expect("seed access token");

    assign_role(pool, account_id, role_name).await;
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
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());
    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
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

async fn assert_problem(response: Response, expected_status: reqwest::StatusCode) -> Value {
    assert_eq!(response.status(), expected_status);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    response.json().await.expect("decode problem response")
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn activate_account_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;
    digital_publication_backend::MIGRATOR.run(&pool).await.expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;
    let target_id = seed_account(
        &pool,
        &format!("e2e-target-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let (address, server) = start_server(test_router(pool.clone())).await;
    let response = Client::new()
        .post(format!(
            "http://{address}/admin/accounts/{target_id}/activate"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send activation request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    let body: Value = response.json().await.expect("decode account response");
    assert_eq!(body["id"], target_id.to_string());
    assert_eq!(body["status"], "active");
    assert_eq!(body["deleted_at"], Value::Null);
    assert!(body.get("password").is_none());
    assert!(body.get("password_hash").is_none());

    let status = sqlx::query_scalar::<_, String>("SELECT status FROM account WHERE id = $1")
        .bind(target_id)
        .fetch_one(&pool)
        .await
        .expect("read activated account");
    let updated_by =
        sqlx::query_scalar::<_, Option<Uuid>>("SELECT updated_by FROM account WHERE id = $1")
            .bind(target_id)
            .fetch_one(&pool)
            .await
            .expect("read activation actor");

    assert_eq!(status, "active");
    assert_eq!(updated_by, Some(admin_id));

    server.abort();
    cleanup_accounts(&pool, &[target_id, admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn activate_account_authentication_authorization_and_conflicts_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;
    digital_publication_backend::MIGRATOR.run(&pool).await.expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;
    let (viewer_id, viewer_token) = seed_principal(&pool, "account_viewer").await;
    let target_id = seed_account(
        &pool,
        &format!("e2e-security-target-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let (address, server) = start_server(test_router(pool.clone())).await;
    let client = Client::new();

    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{target_id}/activate"
        ))
        .send()
        .await
        .expect("send unauthenticated request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api""#)
    );
    assert_problem(response, reqwest::StatusCode::UNAUTHORIZED).await;

    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{target_id}/activate"
        ))
        .bearer_auth("invalid-e2e-activate-token")
        .send()
        .await
        .expect("send invalid-token request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api", error="invalid_token""#)
    );
    assert_problem(response, reqwest::StatusCode::UNAUTHORIZED).await;

    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{target_id}/activate"
        ))
        .bearer_auth(&viewer_token)
        .send()
        .await
        .expect("send unauthorized request");
    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(response.headers().get("www-authenticate").is_none());
    let body = assert_problem(response, reqwest::StatusCode::FORBIDDEN).await;
    assert_eq!(body["code"], "FORBIDDEN");

    let active_id = seed_account(
        &pool,
        &format!("e2e-active-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;
    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{active_id}/activate"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send active-account request");
    let body = assert_problem(response, reqwest::StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "ACCOUNT_ALREADY_ACTIVE");

    let deleted_id = seed_account(
        &pool,
        &format!("e2e-deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(OffsetDateTime::now_utc()),
        Some(admin_id),
        Some(admin_id),
    )
    .await;
    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{deleted_id}/activate"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send soft-deleted account request");
    let body = assert_problem(response, reqwest::StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "ACCOUNT_SOFT_DELETED");

    let missing_id = Uuid::new_v4();
    let response = client
        .post(format!(
            "http://{address}/admin/accounts/{missing_id}/activate"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send missing-account request");
    let body = assert_problem(response, reqwest::StatusCode::NOT_FOUND).await;
    assert_eq!(body["code"], "ACCOUNT_NOT_FOUND");

    server.abort();
    cleanup_accounts(
        &pool,
        &[target_id, active_id, deleted_id, admin_id, viewer_id],
    )
    .await;
}