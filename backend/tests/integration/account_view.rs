use std::{env, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use serde_json::Value;
use sqlx::PgPool;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

async fn test_pool() -> Option<PgPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;

    Some(
        PgPool::connect(&database_url)
            .await
            .expect("connect to TEST_DATABASE_URL"),
    )
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

async fn seed_view_only_role(pool: &PgPool) -> Uuid {
    let role_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO authorization_role (name, description)
        VALUES ($1, 'Test role with only account:view.')
        RETURNING id
        "#,
    )
    .bind(format!("test-account-view-{}", Uuid::new_v4()))
    .fetch_one(pool)
    .await
    .expect("seed view-only role");

    let permission_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM authorization_permission WHERE name = 'account:view'",
    )
    .fetch_one(pool)
    .await
    .expect("find account:view permission");

    sqlx::query(
        "INSERT INTO authorization_role_permission (role_id, permission_id) VALUES ($1, $2)",
    )
    .bind(role_id)
    .bind(permission_id)
    .execute(pool)
    .await
    .expect("seed view-only role permission");

    role_id
}

async fn seed_account_role(pool: &PgPool, account_id: Uuid, role_id: Uuid) -> String {
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
        "test-view-only-{}-{}-{}",
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

fn app(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

fn bearer_request(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn lists_accounts_with_filters_and_pagination() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let active_one = seed_account(
        &pool,
        &format!("active-one-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let active_two = seed_account(
        &pool,
        &format!("active-two-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let inactive = seed_account(
        &pool,
        &format!("inactive-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
    )
    .await;
    let deleted = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    let token = seed_access_token(&pool, active_one, "account_viewer").await;

    let response = app(pool.clone())
        .oneshot(bearer_request("/admin/accounts", &token))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["page"], 1);
    assert_eq!(payload["page_size"], 20);
    assert_eq!(payload["total"], 3);

    let ordered_ids: Vec<String> = payload["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(ordered_ids.len(), 3);
    assert!(ordered_ids.windows(2).all(|ids| ids[0] < ids[1]));
    assert!(ordered_ids.contains(&active_one.to_string()));
    assert!(ordered_ids.contains(&active_two.to_string()));
    assert!(ordered_ids.contains(&inactive.to_string()));
    assert!(!ordered_ids.contains(&deleted.to_string()));

    for item in payload["items"].as_array().unwrap() {
        assert!(item.get("password").is_none());
        assert!(item.get("password_hash").is_none());
    }

    let response = app(pool.clone())
        .oneshot(bearer_request("/admin/accounts?page=1&page_size=2", &token))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    let page_one_ids: Vec<String> = payload["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(page_one_ids, ordered_ids[..2].to_vec());
    assert_eq!(payload["total"], 3);

    let response = app(pool.clone())
        .oneshot(bearer_request("/admin/accounts?page=2&page_size=2", &token))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    let page_two_ids: Vec<String> = payload["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(page_two_ids, ordered_ids[2..].to_vec());

    let response = app(pool.clone())
        .oneshot(bearer_request(
            "/admin/accounts?include_deleted=true",
            &token,
        ))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["total"], 4);
    assert!(payload["items"].as_array().unwrap().iter().any(|item| {
        item["id"]
            .as_str()
            .is_some_and(|id| id == deleted.to_string())
            && item["deleted_at"].is_string()
    }));

    for item in payload["items"].as_array().unwrap() {
        assert!(item.get("password").is_none());
        assert!(item.get("password_hash").is_none());
    }

    let response = app(pool.clone())
        .oneshot(bearer_request("/admin/accounts?status=inactive", &token))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["total"], 1);
    assert_eq!(
        payload["items"][0]["id"].as_str(),
        Some(inactive.to_string().as_str())
    );

    let response = app(pool.clone())
        .oneshot(bearer_request("/admin/accounts?page=99&page_size=20", &token))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["items"].as_array().unwrap().len(), 0);
    assert_eq!(payload["total"], 3);

    sqlx::query("DELETE FROM authorization_account_role WHERE account_id IN ($1, $2, $3, $4)")
        .bind(active_one)
        .bind(active_two)
        .bind(inactive)
        .bind(deleted)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authentication_session WHERE account_id IN ($1, $2, $3, $4)")
        .bind(active_one)
        .bind(active_two)
        .bind(inactive)
        .bind(deleted)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM account WHERE id IN ($1, $2, $3, $4)")
        .bind(active_one)
        .bind(active_two)
        .bind(inactive)
        .bind(deleted)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn denies_include_deleted_without_account_view_deleted_permission() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let principal_id = seed_account(
        &pool,
        &format!("view-only-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let role_id = seed_view_only_role(&pool).await;
    let token = seed_account_role(&pool, principal_id, role_id).await;

    let response = app(pool.clone())
        .oneshot(bearer_request(
            "/admin/accounts?include_deleted=true",
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/problem+json");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());

    sqlx::query("DELETE FROM authorization_account_role WHERE account_id = $1")
        .bind(principal_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authentication_session WHERE account_id = $1")
        .bind(principal_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authorization_role_permission WHERE role_id = $1")
        .bind(role_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authorization_role WHERE id = $1")
        .bind(role_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(principal_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn unauthenticated_view_accounts_is_rejected_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let request = Request::builder()
        .method("GET")
        .uri("/admin/accounts")
        .body(Body::empty())
        .unwrap();

    let response = app(pool).oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}