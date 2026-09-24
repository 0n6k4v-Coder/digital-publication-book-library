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

async fn seed_access_token(pool: &PgPool, account_id: Uuid, role_name: Option<&str>) -> String {
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
    sqlx::query("DELETE FROM authorization_account_role WHERE account_id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("cleanup authorization role");

    sqlx::query("DELETE FROM authentication_session WHERE account_id = $1")
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
async fn views_a_non_deleted_account_and_hides_credentials() {
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
        &format!("principal-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let account_id = seed_account(
        &pool,
        &format!("view-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let token = seed_access_token(&pool, principal_id, Some("account_viewer")).await;

    let response = app(pool.clone())
        .oneshot(bearer_request(
            &format!("/admin/accounts/{account_id}"),
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["id"], account_id.to_string());
    assert_eq!(payload["status"], "active");
    assert!(payload["email"].is_string());
    assert!(payload["created_at"].is_string());
    assert!(payload["updated_at"].is_string());
    assert_eq!(payload["deleted_at"], Value::Null);
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());

    cleanup_account(&pool, account_id).await;
    cleanup_account(&pool, principal_id).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn returns_account_not_found_for_missing_and_soft_deleted_accounts() {
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
        &format!("principal-not-found-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let deleted_id = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;
    let token = seed_access_token(&pool, principal_id, Some("account_viewer")).await;

    for account_id in [Uuid::new_v4(), deleted_id] {
        let response = app(pool.clone())
            .oneshot(bearer_request(
                &format!("/admin/accounts/{account_id}"),
                &token,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );

        let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["code"], "ACCOUNT_NOT_FOUND");
    }

    cleanup_account(&pool, deleted_id).await;
    cleanup_account(&pool, principal_id).await;
}

#[tokio::test]
async fn rejects_invalid_account_id_before_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let request = Request::builder()
        .method("GET")
        .uri("/admin/accounts/not-a-uuid")
        .body(Body::empty())
        .unwrap();

    let response = app(pool).oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );

    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(payload["code"], "INVALID_ACCOUNT_ID");
}

#[tokio::test]
async fn rejects_unauthenticated_view_account_before_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let request = Request::builder()
        .method("GET")
        .uri(format!("/admin/accounts/{}", Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let response = app(pool).oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_authenticated_principal_without_account_view_permission() {
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
        &format!("forbidden-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;
    let token = seed_access_token(&pool, principal_id, None).await;

    let response = app(pool.clone())
        .oneshot(bearer_request(
            &format!("/admin/accounts/{}", Uuid::new_v4()),
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());

    cleanup_account(&pool, principal_id).await;
}