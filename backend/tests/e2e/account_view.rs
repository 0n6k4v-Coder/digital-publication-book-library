use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use reqwest::Client;
use serde_json::Value;
use sha1::{Digest, Sha1};
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

async fn reset_test_database(pool: &PgPool) {
    sqlx::query("DELETE FROM authorization_account_role")
        .execute(pool)
        .await
        .expect("reset authorization account roles");

    sqlx::query("DELETE FROM authentication_session")
        .execute(pool)
        .await
        .expect("reset authentication sessions");

    sqlx::query("DELETE FROM account")
        .execute(pool)
        .await
        .expect("reset test database");
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
    role_name: &str,
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

    token
}

async fn seed_accounts(pool: &PgPool) -> Vec<Uuid> {
    let active_one = seed_account(
        pool,
        &format!(
            "e2e-view-active-one-{}@example.com",
            Uuid::new_v4()
        ),
        "active",
        None,
    )
    .await;

    let active_two = seed_account(
        pool,
        &format!(
            "e2e-view-active-two-{}@example.com",
            Uuid::new_v4()
        ),
        "active",
        None,
    )
    .await;

    let inactive = seed_account(
        pool,
        &format!(
            "e2e-view-inactive-{}@example.com",
            Uuid::new_v4()
        ),
        "inactive",
        None,
    )
    .await;

    let deleted = seed_account(
        pool,
        &format!(
            "e2e-view-deleted-{}@example.com",
            Uuid::new_v4()
        ),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    vec![active_one, active_two, inactive, deleted]
}

fn test_router(pool: PgPool) -> Router {
    let blocklist =
        PasswordBlocklist::from_hashes(
            "test",
            [sha1_hash("password-password")],
        );

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

async fn cleanup_accounts(pool: &PgPool, account_ids: &[Uuid]) {
    sqlx::query(
        "DELETE FROM authorization_account_role WHERE account_id = ANY($1)",
    )
    .bind(account_ids)
    .execute(pool)
    .await
    .expect("cleanup authorization roles");

    sqlx::query(
        "DELETE FROM authentication_session WHERE account_id = ANY($1)",
    )
    .bind(account_ids)
    .execute(pool)
    .await
    .expect("cleanup authentication sessions");

    sqlx::query("DELETE FROM account WHERE id = ANY($1)")
        .bind(account_ids)
        .execute(pool)
        .await
        .expect("cleanup seeded accounts");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn view_accounts_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    reset_test_database(&pool).await;

    let seeded = seed_accounts(&pool).await;
    let token =
        seed_access_token(&pool, seeded[0], "account_viewer").await;

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;

    let response = Client::new()
        .get(format!("http://{address}/admin/accounts"))
        .bearer_auth(&token)
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

    assert_eq!(response_body["page"], 1);
    assert_eq!(response_body["page_size"], 20);
    assert_eq!(response_body["total"], 3);

    let items = response_body["items"]
        .as_array()
        .expect("items array");

    assert_eq!(items.len(), 3);

    let ids: Vec<&str> = items
        .iter()
        .map(|item| item["id"].as_str().expect("account id"))
        .collect();

    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));

    for item in items {
        assert!(item["id"].as_str().is_some());
        assert!(item["email"].as_str().is_some());
        assert!(item["status"].as_str().is_some());
        assert!(item["created_at"].as_str().is_some());
        assert!(item["updated_at"].as_str().is_some());
        assert!(item.get("deleted_at").is_some());
        assert!(item.get("password").is_none());
        assert!(item.get("password_hash").is_none());
        assert_eq!(item["deleted_at"], Value::Null);
    }

    assert!(!items.iter().any(|item| {
        item["id"]
            .as_str()
            .is_some_and(|id| id == seeded[3].to_string())
    }));

    server.abort();
    cleanup_accounts(&pool, &seeded).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn view_accounts_supports_filters_and_pagination_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    reset_test_database(&pool).await;

    let seeded = seed_accounts(&pool).await;
    let token =
        seed_access_token(&pool, seeded[0], "account_viewer").await;

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;
    let client = Client::new();

    let active_response = client
        .get(format!(
            "http://{address}/admin/accounts?status=active"
        ))
        .bearer_auth(&token)
        .send()
        .await
        .expect("send active filter request");

    assert_eq!(active_response.status(), reqwest::StatusCode::OK);
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

    assert_eq!(active_body["total"], 2);
    assert!(active_body["items"]
        .as_array()
        .expect("active items")
        .iter()
        .all(|item| item["status"] == "active"));

    let page_one_response = client
        .get(format!(
            "http://{address}/admin/accounts?page=1&page_size=2"
        ))
        .bearer_auth(&token)
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

    assert_eq!(page_one_body["page"], 1);
    assert_eq!(page_one_body["page_size"], 2);
    assert_eq!(page_one_body["total"], 3);
    assert_eq!(
        page_one_body["items"].as_array().unwrap().len(),
        2
    );

    let page_two_response = client
        .get(format!(
            "http://{address}/admin/accounts?page=2&page_size=2"
        ))
        .bearer_auth(&token)
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
        page_two_body["items"].as_array().unwrap().len(),
        1
    );

    let deleted_response = client
        .get(format!(
            "http://{address}/admin/accounts?include_deleted=true"
        ))
        .bearer_auth(&token)
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

    assert_eq!(deleted_body["total"], 4);

    assert!(deleted_body["items"]
        .as_array()
        .expect("include-deleted items")
        .iter()
        .any(|item| {
            item["id"]
                .as_str()
                .is_some_and(|id| id == seeded[3].to_string())
        }));

    server.abort();
    cleanup_accounts(&pool, &seeded).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn unauthenticated_view_accounts_end_to_end_returns_401() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let app = test_router(pool.clone());
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
async fn invalid_bearer_view_accounts_end_to_end_returns_401() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    reset_test_database(&pool).await;
    let seeded = seed_accounts(&pool).await;

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;

    let response = Client::new()
        .get(format!("http://{address}/admin/accounts"))
        .bearer_auth("unknown-view-token")
        .send()
        .await
        .expect("send invalid bearer request");

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
    cleanup_accounts(&pool, &seeded).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn invalid_view_accounts_query_end_to_end_returns_appropriate_status() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    reset_test_database(&pool).await;

    let seeded = seed_accounts(&pool).await;
    let token =
        seed_access_token(&pool, seeded[0], "account_viewer").await;

    let app = test_router(pool.clone());
    let (address, server) = start_server(app).await;

    for (query, expected_status) in [
        ("page=abc", reqwest::StatusCode::BAD_REQUEST),
        ("page=0", reqwest::StatusCode::UNPROCESSABLE_ENTITY),
        (
            "page_size=0",
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "page_size=101",
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "status=unknown",
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        ),
    ] {
        let response = Client::new()
            .get(format!(
                "http://{address}/admin/accounts?{query}"
            ))
            .bearer_auth(&token)
            .send()
            .await
            .expect("send invalid query request");

        assert_eq!(response.status(), expected_status, "query={query}");
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
    cleanup_accounts(&pool, &seeded).await;
}