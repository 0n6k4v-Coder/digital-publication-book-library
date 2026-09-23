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
async fn lists_accounts_with_filters_and_pagination() {
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = Uuid::new_v4();
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

    let all_accounts_request = Request::builder()
        .method("GET")
        .uri("/admin/accounts")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone())
        .oneshot(all_accounts_request)
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
    assert!(ordered_ids.contains(&active_one.to_string()));
    assert!(ordered_ids.contains(&active_two.to_string()));
    assert!(ordered_ids.contains(&inactive.to_string()));
    assert!(!ordered_ids.contains(&deleted.to_string()));

    for item in payload["items"].as_array().unwrap() {
        assert!(item.get("password").is_none());
        assert!(item.get("password_hash").is_none());
    }

    let page_one = Request::builder()
        .method("GET")
        .uri("/admin/accounts?page=1&page_size=2")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(page_one).await.unwrap();
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

    let page_two = Request::builder()
        .method("GET")
        .uri("/admin/accounts?page=2&page_size=2")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(page_two).await.unwrap();
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

    let with_deleted = Request::builder()
        .method("GET")
        .uri("/admin/accounts?include_deleted=true")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(with_deleted).await.unwrap();
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

    let inactive_only = Request::builder()
        .method("GET")
        .uri("/admin/accounts?status=inactive")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(inactive_only).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["total"], 1);
    assert_eq!(
        payload["items"][0]["id"].as_str(),
        Some(inactive.to_string().as_str())
    );

    let empty_page = Request::builder()
        .method("GET")
        .uri("/admin/accounts?page=99&page_size=20")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::empty())
        .unwrap();

    let response = app(pool.clone()).oneshot(empty_page).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();
    let payload: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(payload["items"].as_array().unwrap().len(), 0);
    assert_eq!(payload["total"], 3);

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
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}

#[tokio::test]
async fn rejects_invalid_view_accounts_query_before_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let admin_id = Uuid::new_v4();

    for uri in [
        "/admin/accounts?page=abc",
        "/admin/accounts?page=0",
        "/admin/accounts?page_size=0",
        "/admin/accounts?page_size=101",
        "/admin/accounts?status=unknown",
    ] {
        let request = Request::builder()
            .method("GET")
            .uri(uri)
            .extension(AuthenticatedAdmin::from_verified_account(admin_id))
            .body(Body::empty())
            .unwrap();

        let response = app(pool.clone()).oneshot(request).await.unwrap();
        let expected_status = if uri.contains("page=abc") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::UNPROCESSABLE_ENTITY
        };

        assert_eq!(response.status(), expected_status, "uri={uri}");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}