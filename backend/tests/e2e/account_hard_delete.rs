use std::{env, net::SocketAddr, sync::Arc};

use reqwest::{Client, Response};
use serde_json::Value;
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

async fn connect_database() -> PgPool {
    let database_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("connect to TEST_DATABASE_URL")
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    status: &str,
    deleted_at: Option<time::OffsetDateTime>,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
) -> Uuid {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (
            status,
            deleted_at,
            created_by,
            updated_by
        )
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
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("seed account credentials");

    account_id
}

async fn assign_role_as(
    pool: &PgPool,
    account_id: Uuid,
    role_name: &str,
    created_by: Uuid,
) {
    let role_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authorization_role WHERE name = $1",
    )
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
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("assign authorization role");
}

async fn seed_tokens(pool: &PgPool, account_id: Uuid) -> String {
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

    let access_token = format!(
        "e2e-purge-access-{}-{}",
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
    .bind(sha256_token_verifier(&access_token))
    .execute(pool)
    .await
    .expect("seed access token");

    let refresh_token = format!(
        "e2e-purge-refresh-{}-{}",
        Uuid::new_v4(),
        Uuid::new_v4()
    );

    sqlx::query(
        r#"
        INSERT INTO authentication_refresh_token (
            session_id,
            token_hash,
            expires_at
        )
        VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '30 days')
        "#,
    )
    .bind(session_id)
    .bind(sha256_token_verifier(&refresh_token))
    .execute(pool)
    .await
    .expect("seed refresh token");

    access_token
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

    let token = seed_tokens(pool, account_id).await;

    assign_role_as(pool, account_id, role_name, account_id).await;

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

fn test_router(pool: PgPool) -> axum::Router {
    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        Vec::<[u8; 20]>::new(),
    );

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(
    app: axum::Router,
) -> (SocketAddr, tokio::task::JoinHandle<()>) {
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

async fn assert_problem(
    response: Response,
    expected_status: reqwest::StatusCode,
) -> Value {
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
async fn hard_delete_account_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) =
        seed_principal(&pool, "account_admin").await;

    let target_id = seed_account(
        &pool,
        &format!("e2e-target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let target_token = seed_tokens(&pool, target_id).await;

    assign_role_as(
        &pool,
        target_id,
        "account_admin",
        admin_id,
    )
    .await;

    let other_admin_id = seed_account(
        &pool,
        &format!("e2e-other-admin-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    assign_role_as(
        &pool,
        other_admin_id,
        "account_admin",
        admin_id,
    )
    .await;

    let survivor_id = seed_account(
        &pool,
        &format!("e2e-survivor-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(target_id),
        Some(target_id),
    )
    .await;

    assign_role_as(
        &pool,
        survivor_id,
        "account_viewer",
        target_id,
    )
    .await;

    let (address, server) = start_server(test_router(pool.clone())).await;

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/{target_id}/purge"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send purge request");

    assert_eq!(response.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let body = response
        .bytes()
        .await
        .expect("read purge response body");
    assert!(body.is_empty());

    let target_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account WHERE id = $1)",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("check target account");
    assert!(!target_exists);

    let survivor_creator = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT created_by
        FROM authorization_account_role
        WHERE account_id = $1
          AND role_id = (
              SELECT id
              FROM authorization_role
              WHERE name = 'account_viewer'
          )
        "#,
    )
    .bind(survivor_id)
    .fetch_optional(&pool)
    .await
    .expect("fetch surviving creator");

    assert_eq!(survivor_creator.flatten(), None);

    let response = Client::new()
        .get(format!(
            "http://{address}/admin/accounts/{survivor_id}"
        ))
        .bearer_auth(&target_token)
        .send()
        .await
        .expect("send post-delete authentication request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api", error="invalid_token""#)
    );

    let _ = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authentication_refresh_token",
    )
    .fetch_one(&pool)
    .await
    .expect("verify refresh-token table exists");

    cleanup_accounts(
        &pool,
        &[target_id, survivor_id, other_admin_id, admin_id],
    )
    .await;

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn hard_delete_account_end_to_end_enforces_authz_and_last_admin_rule() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) =
        seed_principal(&pool, "account_admin").await;
    let (viewer_id, viewer_token) =
        seed_principal(&pool, "account_viewer").await;

    let (address, server) = start_server(test_router(pool.clone())).await;

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/{admin_id}/purge"
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

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/{admin_id}/purge"
        ))
        .bearer_auth(&viewer_token)
        .send()
        .await
        .expect("send unauthorized request");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    assert_problem(response, reqwest::StatusCode::FORBIDDEN).await;

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/{admin_id}/purge"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send last-administrator request");

    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);

    let body = assert_problem(
        response,
        reqwest::StatusCode::CONFLICT,
    )
    .await;
    assert_eq!(body["code"], "LAST_ACTIVE_ADMINISTRATOR");

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/not-a-uuid/purge"
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send invalid-account-id request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
    let body = assert_problem(
        response,
        reqwest::StatusCode::BAD_REQUEST,
    )
    .await;
    assert_eq!(body["code"], "INVALID_ACCOUNT_ID");

    let response = Client::new()
        .delete(format!(
            "http://{address}/admin/accounts/{}/purge",
            Uuid::new_v4()
        ))
        .bearer_auth(&admin_token)
        .send()
        .await
        .expect("send missing-account request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    let body = assert_problem(
        response,
        reqwest::StatusCode::NOT_FOUND,
    )
    .await;
    assert_eq!(body["code"], "ACCOUNT_NOT_FOUND");

    cleanup_accounts(&pool, &[admin_id, viewer_id]).await;

    server.abort();
}