use std::{env, net::SocketAddr, sync::Arc};

use axum::{extract::Request, middleware, Router};
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::{auth::AuthenticatedAdmin, validation::PasswordBlocklist},
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
    deleted_at: Option<time::OffsetDateTime>,
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

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());
    let application = build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ));

    application.layer(middleware::from_fn(
        |mut request: Request, next: middleware::Next| async move {
            let admin_id = request
                .headers()
                .get("x-test-admin-id")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| Uuid::parse_str(value).ok());

            if let Some(admin_id) = admin_id {
                request
                    .extensions_mut()
                    .insert(AuthenticatedAdmin::from_verified_account(admin_id));
            }

            next.run(request).await
        },
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

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn views_account_end_to_end_and_hides_sensitive_fields() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let account_id = seed_account(
        &pool,
        &format!("e2e-view-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;
    let client = Client::new();

    let response = client
        .get(format!("http://{address}/admin/accounts/{account_id}"))
        .header("x-test-admin-id", Uuid::new_v4().to_string())
        .send()
        .await
        .expect("send view-account request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
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
        &format!("e2e-view-deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    let response = client
        .get(format!("http://{address}/admin/accounts/{deleted_id}"))
        .header("x-test-admin-id", Uuid::new_v4().to_string())
        .send()
        .await
        .expect("send deleted-account request");

    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let payload: Value = response.json().await.expect("decode problem response");
    assert_eq!(payload["code"], "ACCOUNT_NOT_FOUND");

    sqlx::query("DELETE FROM account WHERE id IN ($1, $2)")
        .bind(account_id)
        .bind(deleted_id)
        .execute(&pool)
        .await
        .expect("cleanup test accounts");

    server.abort();
}

#[tokio::test]
async fn invalid_account_id_and_unauthenticated_request_are_rejected_end_to_end() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let app = test_router(pool);
    let (address, server) = start_server(app).await;
    let client = Client::new();

    let invalid = client
        .get(format!("http://{address}/admin/accounts/not-a-uuid"))
        .header("x-test-admin-id", Uuid::new_v4().to_string())
        .send()
        .await
        .expect("send invalid-id request");

    assert_eq!(invalid.status(), reqwest::StatusCode::BAD_REQUEST);
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