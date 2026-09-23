use std::{env, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::{auth::AuthenticatedAdmin, validation::PasswordBlocklist},
};

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

fn app(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn views_a_non_deleted_account_and_hides_credentials() {
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let account_id = seed_account(
        &pool,
        &format!("view-{}@example.com", Uuid::new_v4()),
        "active",
        None,
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/admin/accounts/{account_id}"))
        .extension(AuthenticatedAdmin::from_verified_account(Uuid::new_v4()))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(request).await.unwrap();

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

    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(account_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn returns_account_not_found_for_missing_and_soft_deleted_accounts() {
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let deleted_id = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(time::OffsetDateTime::now_utc()),
    )
    .await;

    for account_id in [Uuid::new_v4(), deleted_id] {
        let request = Request::builder()
            .method("GET")
            .uri(format!("/admin/accounts/{account_id}"))
            .extension(AuthenticatedAdmin::from_verified_account(Uuid::new_v4()))
            .body(Body::empty())
            .unwrap();

        let response = app(pool.clone()).oneshot(request).await.unwrap();
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

    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(deleted_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn rejects_invalid_account_id_before_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let request = Request::builder()
        .method("GET")
        .uri("/admin/accounts/not-a-uuid")
        .extension(AuthenticatedAdmin::from_verified_account(Uuid::new_v4()))
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