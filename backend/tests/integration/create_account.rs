use std::{env, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use serde_json::json;
use sha1::{Digest, Sha1};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
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

async fn test_pool() -> Option<PgPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;

    Some(
        PgPool::connect(&database_url)
            .await
            .expect("connect to TEST_DATABASE_URL"),
    )
}

async fn seed_admin(pool: &PgPool) -> Uuid {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (status, deleted_at)
        VALUES ('active', NULL)
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("seed admin account");

    let email = format!("seed-{account_id}@example.com");

    sqlx::query(
        r#"
        INSERT INTO account_credentials (
            account_id,
            email,
            email_normalized,
            password_hash
        )
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(account_id)
    .bind(&email)
    .bind(&email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("seed admin credentials");

    account_id
}

fn app(pool: PgPool) -> Router {
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

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn creates_account_and_credentials_atomically() {
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = seed_admin(&pool).await;

    let email = format!("AC_UC_01-{Uuid::new_v4()}@example.com");
    let password = "a secure password with enough length";

    let request = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::from(
            json!({
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let response = app(pool.clone())
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert!(
        response
            .headers()
            .contains_key(header::LOCATION)
    );

    let body = axum::body::to_bytes(response.into_body(), 32 * 1024)
        .await
        .unwrap();

    let payload: serde_json::Value =
        serde_json::from_slice(&body).unwrap();

    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());

    let id =
        Uuid::parse_str(payload["id"].as_str().unwrap()).unwrap();

    let account =
        sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Uuid,
                String,
                Option<time::OffsetDateTime>,
            ),
        >(
            "SELECT id, created_by, updated_by, status, deleted_at \
             FROM account WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(account.0, id);
    assert_eq!(account.1, admin_id);
    assert_eq!(account.2, admin_id);
    assert_eq!(account.3, "active");
    assert!(account.4.is_none());

    let credential_hash =
        sqlx::query_scalar::<_, String>(
            "SELECT password_hash \
             FROM account_credentials WHERE account_id = $1",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(credential_hash.starts_with("$argon2id$v=19$"));

    sqlx::query("DELETE FROM account WHERE id IN ($1, $2)")
        .bind(id)
        .bind(admin_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_duplicate_email_using_database_uniqueness() {
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let admin_id = seed_admin(&pool).await;
    let email =
        format!("duplicate-{Uuid::new_v4()}@example.com");

    let seed_request = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::from(
            json!({
                "email": email,
                "password": "first secure password"
            })
            .to_string(),
        ))
        .unwrap();

    assert_eq!(
        app(pool.clone())
            .oneshot(seed_request)
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED,
    );

    let duplicate_request = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .extension(AuthenticatedAdmin::from_verified_account(admin_id))
        .body(Body::from(
            json!({
                "email": email.to_uppercase(),
                "password": "second secure password"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app(pool.clone())
        .oneshot(duplicate_request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    sqlx::query(
        "DELETE FROM account WHERE id = \
         (SELECT account_id FROM account_credentials \
          WHERE email_normalized = $1)",
    )
    .bind(email.to_lowercase())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(admin_id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn unauthenticated_create_account_is_rejected_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": "admin@example.com",
                "password": "a secure password"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app(pool)
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn invalid_password_is_rejected_before_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json")
        .extension(
            AuthenticatedAdmin::from_verified_account(Uuid::new_v4()),
        )
        .body(Body::from(
            json!({
                "email": "admin@example.com",
                "password": "too-short"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app(pool)
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
}