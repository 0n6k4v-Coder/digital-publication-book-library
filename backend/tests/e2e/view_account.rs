use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use reqwest::Client;
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
        .expect("connect to test database")
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    status: &str,
    deleted_at: Option<time::OffsetDateTime>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (status, deleted_at)
            VALUES ($1, $2)
            RETURNING id
        )
        INSERT INTO account_credentials (
            account_id,
            email,
            email_normalized,
            password_hash
        )
        SELECT
            id,
            $3,
            $3,
            $4
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .fetch_one(pool)
    .await
    .expect("seed account")
}

async fn seed_access_token(
    pool: &PgPool,
    account_id: Uuid,
    role_name: Option<&str>,
) -> String {
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
        "test-view-{}-{}-{}",
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

    if let Some(role_name) = role_name {
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
            VALUES ($1, $2, $1)
            "#,
        )
        .bind(account_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("seed authorization role");
    }

    token
}

async fn cleanup_account(pool: &PgPool, account_id: Uuid) {
    sqlx::query(
        "DELETE FROM authorization_account_role WHERE account_id = $1",
    )
    .bind(account_id)
    .execute(pool)
    .await
    .expect("cleanup authorization role");

    sqlx::query(
        "DELETE FROM authentication_session WHERE account_id = $1",
    )
    .bind(account_id)
    .execute(pool)
    .await
    .expect("cleanup authentication session");

    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("cleanup account");
}

fn test_router(pool: PgPool) -> Router {
    let blocklist =
        PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(
    app: Router,
) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind e2e server");

    let address =
        listener.local_addr().expect("read e2e server address");

    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve e2e application");
    });

    (address, task)
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn views_account_end_to_end_and_hides_sensitive_fields() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let principal_id = seed_account(
        &pool,
        &format!(
            "e2e-principal-{}@example.com",
            Uuid::new_v4()
        ),
        "active",
        None,
    )
    .await;

    let account_id = seed_account(
        &pool,
        &format!(
            "e2e-view-{}@example.com",
            Uuid::new_v4()
        ),
        "active",
        None,
    )
    .await;

    let token =
        seed_access_token(
            &pool,
            principal_id,
            Some("account_viewer"),
        )
        .await;

    let app = test_router(pool.clone());
    let (address, server) =
        start_server(app).await;

    let client = Client::new();

    let response = client
        .get(format!(
            "http://{address}/admin/accounts/{account_id}"
        ))
        .header(
            "authorization",
            format!("Bearer {token}"),
        )
        .send()
        .await
        .expect("send view-account request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::OK
    );

    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let payload: Value = response
        .json()
        .await
        .expect("decode account response");

    assert_eq!(payload["id"], account_id.to_string());
    assert_eq!(payload["status"], "active");
    assert_eq!(payload["deleted_at"], Value::Null);
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());

    let deleted_id = seed_account(
        &pool,
        &format!(
            "e2e-view-deleted-{}@example.com",
            Uuid::new_v4()
        ),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    let response = client
        .get(format!(
            "http://{address}/admin/accounts/{deleted_id}"
        ))
        .header(
            "authorization",
            format!("Bearer {token}"),
        )
        .send()
        .await
        .expect("send deleted-account request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::NOT_FOUND
    );

    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let payload: Value =
        response.json().await.expect("decode problem response");

    assert_eq!(payload["code"], "ACCOUNT_NOT_FOUND");

    cleanup_account(&pool, deleted_id).await;
    cleanup_account(&pool, account_id).await;
    cleanup_account(&pool, principal_id).await;

    server.abort();
}

#[tokio::test]
async fn invalid_account_id_and_unauthenticated_request_are_rejected_end_to_end() {
    let pool =
        sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let app = test_router(pool);
    let (address, server) =
        start_server(app).await;

    let client = Client::new();

    let invalid = client
        .get(format!(
            "http://{address}/admin/accounts/not-a-uuid"
        ))
        .send()
        .await
        .expect("send invalid-id request");

    assert_eq!(
        invalid.status(),
        reqwest::StatusCode::BAD_REQUEST
    );

    let payload: Value = invalid
        .json()
        .await
        .expect("decode invalid-id problem response");

    assert_eq!(payload["code"], "INVALID_ACCOUNT_ID");

    let unauthenticated = client
        .get(format!(
            "http://{address}/admin/accounts/{}",
            Uuid::new_v4()
        ))
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
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api""#)
    );

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn authenticated_principal_without_view_permission_is_rejected_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let principal_id = seed_account(
        &pool,
        &format!(
            "e2e-forbidden-{}@example.com",
            Uuid::new_v4()
        ),
        "active",
        None,
    )
    .await;

    let token =
        seed_access_token(&pool, principal_id, None).await;

    let app = test_router(pool.clone());
    let (address, server) =
        start_server(app).await;

    let response = Client::new()
        .get(format!(
            "http://{address}/admin/accounts/{}",
            Uuid::new_v4()
        ))
        .header(
            "authorization",
            format!("Bearer {token}"),
        )
        .send()
        .await
        .expect("send forbidden request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::FORBIDDEN
    );

    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    assert!(
        response
            .headers()
            .get("www-authenticate")
            .is_none()
    );

    cleanup_account(&pool, principal_id).await;
    server.abort();
}