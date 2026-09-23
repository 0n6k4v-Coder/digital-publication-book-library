use std::{
    env,
    net::SocketAddr,
    sync::Arc,
};

use axum::{
    extract::Request,
    middleware,
    Router,
};
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use sqlx::PgPool;
use tokio::net::TcpListener;
use uuid::Uuid;

use digital_publication_backend::{
    app::{
        router::build_router,
        state::AppState,
    },
    shared::{
        auth::AuthenticatedAdmin,
        validation::PasswordBlocklist,
    },
};

fn sha1_hash(password: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

async fn connect_database() -> PgPool {
    let database_url =
        env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("connect to test database")
}

async fn seed_admin(pool: &PgPool) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (status, deleted_at)
        VALUES ('active', NULL)
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("seed administrator")
}

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        [sha1_hash("password-password")],
    );

    let state = AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    );

    let application = build_router(state);

    application.layer(middleware::from_fn(
        |mut request: Request, next: middleware::Next| async move {
            let admin_id = request
                .headers()
                .get("x-test-admin-id")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Uuid::parse_str(value).ok());

            if let Some(admin_id) = admin_id {
                request.extensions_mut().insert(
                    AuthenticatedAdmin::from_verified_account(admin_id),
                );
            }

            next.run(request).await
        },
    ))
}

async fn start_server(
    app: Router,
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
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = seed_admin(&pool).await;

    let app = test_router(pool.clone());

    let (address, server) = start_server(app).await;

    let client = Client::new();

    let email = format!(
        "e2e-{}@example.com",
        Uuid::new_v4()
    );

    let password = "an extremely secure password";

    let response = client
        .post(format!(
            "http://{address}/admin/accounts"
        ))
        .header("content-type", "application/json")
        .header(
            "x-test-admin-id",
            admin_id.to_string(),
        )
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("send create-account request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::CREATED
    );

    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    assert!(
        response
            .headers()
            .contains_key("location")
    );

    let response_body: Value = response
        .json()
        .await
        .expect("decode account response");

    assert!(
        response_body
            .get("id")
            .and_then(Value::as_str)
            .is_some()
    );

    assert_eq!(
        response_body
            .get("email")
            .and_then(Value::as_str),
        Some(email.as_str())
    );

    assert_eq!(
        response_body
            .get("status")
            .and_then(Value::as_str),
        Some("active")
    );

    assert!(
        response_body
            .get("deleted_at")
            .is_some_and(Value::is_null)
    );

    assert!(
        response_body
            .get("password")
            .is_none()
    );

    assert!(
        response_body
            .get("password_hash")
            .is_none()
    );

    let account_id = Uuid::parse_str(
        response_body["id"]
            .as_str()
            .expect("account id"),
    )
    .expect("valid UUID");

    let stored =
        sqlx::query_as::<
            _,
            (Uuid, String, String, Uuid, Uuid),
        >(
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
    assert!(
        stored
            .2
            .starts_with("$argon2id$v=19$")
    );
    assert_eq!(stored.3, admin_id);
    assert_eq!(stored.4, admin_id);

    sqlx::query(
        "DELETE FROM account WHERE id IN ($1, $2)",
    )
    .bind(account_id)
    .bind(admin_id)
    .execute(&pool)
    .await
    .expect("cleanup test accounts");

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn unauthenticated_create_account_end_to_end_returns_401() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let app = test_router(pool);

    let (address, server) =
        start_server(app).await;

    let response = Client::new()
        .post(format!(
            "http://{address}/admin/accounts"
        ))
        .header("content-type", "application/json")
        .json(&json!({
            "email": "admin@example.com",
            "password": "an extremely secure password"
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
async fn blocklisted_password_is_rejected_end_to_end() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = seed_admin(&pool).await;

    let blocklisted_password =
        SecretString::from(
            "password-password".to_owned(),
        );

    let app = test_router(pool.clone());

    let (address, server) =
        start_server(app).await;

    let response = Client::new()
        .post(format!(
            "http://{address}/admin/accounts"
        ))
        .header("content-type", "application/json")
        .header(
            "x-test-admin-id",
            admin_id.to_string(),
        )
        .json(&json!({
            "email": format!(
                "blocked-{}@example.com",
                Uuid::new_v4()
            ),
            "password":
                blocklisted_password.expose_secret()
        }))
        .send()
        .await
        .expect("send blocked-password request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::UNPROCESSABLE_ENTITY
    );

    server.abort();

    sqlx::query(
        "DELETE FROM account WHERE id = $1",
    )
    .bind(admin_id)
    .execute(&pool)
    .await
    .expect("cleanup administrator");
}