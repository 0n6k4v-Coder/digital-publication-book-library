use std::{env, net::SocketAddr, sync::Arc};

use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{json, Value};
use sha1::{Digest as Sha1Digest, Sha1};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

fn sha1_hash(password: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

async fn connect_database() -> PgPool {
    let database_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("connect to test database")
}

async fn seed_principal(
    pool: &PgPool,
    role_name: &str,
) -> (Uuid, String) {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (status, deleted_at)
            VALUES ('active', NULL)
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
            $1,
            $1,
            $2
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(format!("seed-{}@example.com", Uuid::new_v4()))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .fetch_one(pool)
    .await
    .expect("seed principal account");

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
        "test-admin-{}-{}-{}",
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

    (account_id, token)
}

async fn cleanup_accounts(pool: &PgPool, account_ids: &[Uuid]) {
    if account_ids.is_empty() {
        return;
    }

    let account_ids = account_ids.to_vec();

    sqlx::query(
        "DELETE FROM authorization_account_role WHERE account_id = ANY($1)",
    )
    .bind(&account_ids)
    .execute(pool)
    .await
    .expect("cleanup authorization roles");

    sqlx::query(
        "DELETE FROM authentication_session WHERE account_id = ANY($1)",
    )
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
        [sha1_hash("password-password")],
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

    let address = listener
        .local_addr()
        .expect("read e2e server address");

    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve e2e application");
    });

    (address, task)
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn create_account_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, token) =
        seed_principal(&pool, "account_admin").await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let email = format!("e2e-{}@example.com", Uuid::new_v4());
    let password = "an extremely secure password";

    let response = Client::new()
        .post(format!("http://{address}/admin/accounts"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .json(&json!({
            "email": email,
            "password": password,
            "account_id": Uuid::new_v4(),
            "roles": ["account_viewer"],
            "permissions": ["account:view"]
        }))
        .send()
        .await
        .expect("send create-account request");

    assert_eq!(response.status(), reqwest::StatusCode::CREATED);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(response.headers().contains_key("location"));

    let response_body: Value = response
        .json()
        .await
        .expect("decode account response");

    assert!(response_body
        .get("id")
        .and_then(Value::as_str)
        .is_some());

    assert_eq!(
        response_body.get("email").and_then(Value::as_str),
        Some(email.as_str())
    );

    assert_eq!(
        response_body.get("status").and_then(Value::as_str),
        Some("active")
    );

    assert!(
        response_body
            .get("deleted_at")
            .is_some_and(Value::is_null)
    );

    assert!(response_body.get("password").is_none());
    assert!(response_body.get("password_hash").is_none());

    let account_id = Uuid::parse_str(
        response_body["id"]
            .as_str()
            .expect("account id"),
    )
    .expect("valid UUID");

    let stored =
        sqlx::query_as::<_, (Uuid, String, String, Uuid, Uuid)>(
            r#"
            SELECT
                a.id,
                a.status,
                ac.password_hash,
                a.created_by,
                a.updated_by
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
            "#,
        )
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .expect("fetch created account");

    assert_eq!(stored.0, account_id);
    assert_eq!(stored.1, "active");
    assert!(stored.2.starts_with("$argon2id$v=19$"));
    assert_eq!(stored.3, admin_id);
    assert_eq!(stored.4, admin_id);

    cleanup_accounts(&pool, &[account_id, admin_id]).await;
    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn unauthenticated_create_account_end_to_end_returns_401() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (address, server) =
        start_server(test_router(pool)).await;

    let response = Client::new()
        .post(format!(
            "http://{address}/admin/accounts?access_token=query-or-body-token"
        ))
        .header("content-type", "application/json")
        .json(&json!({
            "email": "admin@example.com",
            "password": "an extremely secure password",
            "access_token": "query-or-body-token"
        }))
        .send()
        .await
        .expect("send unauthenticated request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED
    );

    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api""#)
    );

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

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn invalid_bearer_authentication_returns_401_with_invalid_token_challenge() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (address, server) =
        start_server(test_router(pool)).await;

    let response = Client::new()
        .post(format!("http://{address}/admin/accounts"))
        .header("content-type", "application/json")
        .header("authorization", "Basic not-a-bearer-token")
        .json(&json!({
            "email": "admin@example.com",
            "password": "an extremely secure password"
        }))
        .send()
        .await
        .expect("send invalid-authentication request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNAUTHORIZED
    );

    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api", error="invalid_token""#)
    );

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

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn authenticated_principal_without_account_create_permission_returns_403() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (viewer_id, token) =
        seed_principal(&pool, "account_viewer").await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let response = Client::new()
        .post(format!("http://{address}/admin/accounts"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .json(&json!({
            "email": "forbidden@example.com",
            "password": "an extremely secure password"
        }))
        .send()
        .await
        .expect("send forbidden request");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);

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

    assert!(response.headers().get("www-authenticate").is_none());

    let created = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account WHERE created_by = $1",
    )
    .bind(viewer_id)
    .fetch_one(&pool)
    .await
    .expect("count created accounts");

    assert_eq!(created, 0);

    cleanup_accounts(&pool, &[viewer_id]).await;
    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn malformed_json_and_blocklisted_password_remain_account_validation_errors() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, token) =
        seed_principal(&pool, "account_admin").await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let client = Client::new();

    let malformed = client
        .post(format!("http://{address}/admin/accounts"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body("{")
        .send()
        .await
        .expect("send malformed-json request");

    assert_eq!(
        malformed.status(),
        reqwest::StatusCode::BAD_REQUEST
    );

    assert_eq!(
        malformed
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );

    let blocked = SecretString::from(
        "password-password".to_owned(),
    );

    let response = client
        .post(format!("http://{address}/admin/accounts"))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .json(&json!({
            "email": format!(
                "blocked-{}@example.com",
                Uuid::new_v4()
            ),
            "password": blocked.expose_secret()
        }))
        .send()
        .await
        .expect("send blocked-password request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNPROCESSABLE_ENTITY
    );

    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    cleanup_accounts(&pool, &[admin_id]).await;
    server.abort();
}