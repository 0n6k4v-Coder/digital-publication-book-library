use std::{env, net::SocketAddr, sync::Arc};

use axum::{extract::Request, middleware, Router};
use reqwest::Client;
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

async fn seed_accounts(pool: &PgPool) -> Vec<Uuid> {
    let active_one = seed_account(
        pool,
        &format!("e2e-view-active-one-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;

    let active_two = seed_account(
        pool,
        &format!("e2e-view-active-two-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;

    let inactive = seed_account(
        pool,
        &format!("e2e-view-inactive-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
    )
    .await;

    let deleted = seed_account(
        pool,
        &format!("e2e-view-deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    vec![active_one, active_two, inactive, deleted]
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
                request
                    .extensions_mut()
                    .insert(AuthenticatedAdmin::from_verified_account(admin_id));
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
async fn view_accounts_end_to_end() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let seeded = seed_accounts(&pool).await;

    let admin_id = seeded[0];

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;

    let client = Client::new();

    let response = client
        .get(format!("http://{address}/admin/accounts"))
        .header("x-test-admin-id", admin_id.to_string())
        .send()
        .await
        .expect("send view-accounts request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

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
        Some("application/json")
    );

    let response_body: Value = response
        .json()
        .await
        .expect("decode account-list response");

    assert_eq!(
        response_body
            .get("page")
            .and_then(Value::as_u64),
        Some(1)
    );

    assert_eq!(
        response_body
            .get("page_size")
            .and_then(Value::as_u64),
        Some(20)
    );

    assert_eq!(
        response_body
            .get("total")
            .and_then(Value::as_i64),
        Some(3)
    );

    let items = response_body
        .get("items")
        .and_then(Value::as_array)
        .expect("items array");

    assert_eq!(items.len(), 3);

    for item in items {
        assert!(item.get("id").and_then(Value::as_str).is_some());
        assert!(item.get("email").and_then(Value::as_str).is_some());
        assert!(item.get("status").and_then(Value::as_str).is_some());
        assert!(item.get("created_at").and_then(Value::as_str).is_some());
        assert!(item.get("updated_at").and_then(Value::as_str).is_some());
        assert!(item.get("deleted_at").is_some());

        assert!(item.get("password").is_none());
        assert!(item.get("password_hash").is_none());

        assert_eq!(
            item.get("deleted_at")
                .expect("deleted_at field for non-deleted account"),
            &Value::Null
        );
    }

    assert!(
        !items.iter().any(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id == seeded[3].to_string())
        })
    );

    server.abort();

    sqlx::query("DELETE FROM account WHERE id = ANY($1)")
        .bind(&seeded)
        .execute(&pool)
        .await
        .expect("cleanup seeded accounts");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn view_accounts_supports_filters_and_pagination_end_to_end() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let seeded = seed_accounts(&pool).await;

    let admin_id = seeded[0];

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;

    let client = Client::new();

    let active_response = client
        .get(format!(
            "http://{address}/admin/accounts?status=active"
        ))
        .header("x-test-admin-id", admin_id.to_string())
        .send()
        .await
        .expect("send active filter request");

    assert_eq!(
        active_response.status(),
        reqwest::StatusCode::OK
    );

    assert_eq!(
        active_response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let active_body: Value = active_response
        .json()
        .await
        .expect("decode active filter response");

    assert_eq!(
        active_body
            .get("total")
            .and_then(Value::as_i64),
        Some(2)
    );

    assert!(active_body["items"]
        .as_array()
        .expect("active items")
        .iter()
        .all(|item| item["status"] == "active"));

    let page_one_response = client
        .get(format!(
            "http://{address}/admin/accounts?page=1&page_size=2"
        ))
        .header("x-test-admin-id", admin_id.to_string())
        .send()
        .await
        .expect("send page-one request");

    assert_eq!(
        page_one_response.status(),
        reqwest::StatusCode::OK
    );

    let page_one_body: Value = page_one_response
        .json()
        .await
        .expect("decode page-one response");

    assert_eq!(
        page_one_body
            .get("page")
            .and_then(Value::as_u64),
        Some(1)
    );

    assert_eq!(
        page_one_body
            .get("page_size")
            .and_then(Value::as_u64),
        Some(2)
    );

    assert_eq!(
        page_one_body
            .get("total")
            .and_then(Value::as_i64),
        Some(3)
    );

    assert_eq!(
        page_one_body["items"]
            .as_array()
            .expect("page-one items")
            .len(),
        2
    );

    let page_two_response = client
        .get(format!(
            "http://{address}/admin/accounts?page=2&page_size=2"
        ))
        .header("x-test-admin-id", admin_id.to_string())
        .send()
        .await
        .expect("send page-two request");

    assert_eq!(
        page_two_response.status(),
        reqwest::StatusCode::OK
    );

    let page_two_body: Value = page_two_response
        .json()
        .await
        .expect("decode page-two response");

    assert_eq!(
        page_two_body["items"]
            .as_array()
            .expect("page-two items")
            .len(),
        1
    );

    let deleted_response = client
        .get(format!(
            "http://{address}/admin/accounts?include_deleted=true"
        ))
        .header("x-test-admin-id", admin_id.to_string())
        .send()
        .await
        .expect("send include-deleted request");

    assert_eq!(
        deleted_response.status(),
        reqwest::StatusCode::OK
    );

    let deleted_body: Value = deleted_response
        .json()
        .await
        .expect("decode include-deleted response");

    assert_eq!(
        deleted_body
            .get("total")
            .and_then(Value::as_i64),
        Some(4)
    );

    assert!(deleted_body["items"]
        .as_array()
        .expect("include-deleted items")
        .iter()
        .any(|item| {
            item.get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id == seeded[3].to_string())
        }));

    server.abort();

    sqlx::query("DELETE FROM account WHERE id = ANY($1)")
        .bind(&seeded)
        .execute(&pool)
        .await
        .expect("cleanup seeded accounts");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn unauthenticated_view_accounts_end_to_end_returns_401() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let app = test_router(pool);
    let (address, server) = start_server(app).await;

    let response = Client::new()
        .get(format!("http://{address}/admin/accounts"))
        .send()
        .await
        .expect("send unauthenticated view request");

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
async fn invalid_view_accounts_query_end_to_end_returns_appropriate_status() {
    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = Uuid::new_v4();

    let app = test_router(pool);
    let (address, server) = start_server(app).await;

    for (query, expected_status) in [
        ("page=abc", reqwest::StatusCode::BAD_REQUEST),
        ("page=0", reqwest::StatusCode::UNPROCESSABLE_ENTITY),
        ("page_size=0", reqwest::StatusCode::UNPROCESSABLE_ENTITY),
        ("page_size=101", reqwest::StatusCode::UNPROCESSABLE_ENTITY),
        (
            "status=unknown",
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let response = client_for_test()
            .get(format!("http://{address}/admin/accounts?{query}"))
            .header("x-test-admin-id", admin_id.to_string())
            .send()
            .await
            .expect("send invalid query request");

        assert_eq!(
            response.status(),
            expected_status,
            "query={query}"
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
    }

    server.abort();
}

fn client_for_test() -> Client {
    Client::new()
}