use std::{env, sync::Arc};

use argon2::{
    password_hash::{phc::PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    response::Response,
    Router,
};
use secrecy::SecretString;
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tower::ServiceExt;
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::{hash_password, PasswordBlocklist},
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

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

async fn seed_account(
    pool: &PgPool,
    email: &str,
    status: &str,
    deleted_at: Option<OffsetDateTime>,
    password: &str,
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

    let password_hash =
        hash_password(SecretString::from(password.to_owned())).expect("hash seed password");

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
    .bind(password_hash)
    .execute(pool)
    .await
    .expect("seed account credentials");

    account_id
}

async fn seed_principal(pool: &PgPool, role_name: &str) -> (Uuid, String) {
    let account_id = seed_account(
        pool,
        &format!("principal-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "principal seed password with enough length",
    )
    .await;

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
        "test-change-password-{}-{}-{}",
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

    let role_id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM authorization_role WHERE name = $1")
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

    sqlx::query("DELETE FROM authorization_account_role WHERE account_id = ANY($1)")
        .bind(&account_ids)
        .execute(pool)
        .await
        .expect("cleanup authorization roles");

    sqlx::query("DELETE FROM authentication_session WHERE account_id = ANY($1)")
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

fn app(pool: PgPool) -> Router {
    build_router(AppState::new(
        pool,
        Arc::new(PasswordBlocklist::from_hashes(
            "test",
            [sha1_hash("password-password")],
        )),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

fn change_password_request(
    account_id: Uuid,
    token: Option<&str>,
    password: &str,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method("PATCH")
        .uri(format!("/admin/accounts/{account_id}/password"))
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    builder
        .body(Body::from(json!({ "password": password }).to_string()))
        .expect("build request")
}

async fn assert_problem(response: Response, expected_status: StatusCode, expected_code: &str) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let body = to_bytes(response.into_body(), 32 * 1024)
        .await
        .expect("read problem response");
    let payload: Value = serde_json::from_slice(&body).expect("decode problem response");

    assert_eq!(payload["code"], expected_code);
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());
}

#[tokio::test]
async fn unauthenticated_change_password_is_rejected() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let response = app(pool)
        .oneshot(change_password_request(
            Uuid::new_v4(),
            None,
            "a secure password with enough length",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn changes_password_without_changing_account_metadata() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    digital_publication_backend::MIGRATOR.run(&pool).await.expect("run migrations");

    let (admin_id, token) = seed_principal(&pool, "account_admin").await;
    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "old secure password with enough length",
    )
    .await;

    let before = sqlx::query_as::<_, (OffsetDateTime, OffsetDateTime, String)>(
        r#"
        SELECT a.updated_at, ac.updated_at, ac.password_hash
        FROM account AS a
        INNER JOIN account_credentials AS ac
            ON ac.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read before snapshot");

    let new_password = "new secure password with enough length";
    let response = app(pool.clone())
        .oneshot(change_password_request(
            target_id,
            Some(&token),
            new_password,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let body = to_bytes(response.into_body(), 32 * 1024)
        .await
        .expect("read empty response body");
    assert!(body.is_empty());

    let after = sqlx::query_as::<_, (OffsetDateTime, OffsetDateTime, String)>(
        r#"
        SELECT a.updated_at, ac.updated_at, ac.password_hash
        FROM account AS a
        INNER JOIN account_credentials AS ac
            ON ac.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read after snapshot");

    assert_eq!(after.0, before.0);
    assert!(after.1 > before.1);
    assert_ne!(after.2, before.2);
    assert!(after.2.starts_with("$argon2id$v=19$"));
    Argon2::default()
        .verify_password(
            new_password.as_bytes(),
            &PasswordHash::new(&after.2).expect("parse password hash"),
        )
        .expect("verify new password");

    cleanup_accounts(&pool, &[target_id, admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_password_policy_violations_missing_and_soft_deleted_targets() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    digital_publication_backend::MIGRATOR.run(&pool).await.expect("run migrations");

    let (admin_id, token) = seed_principal(&pool, "account_admin").await;
    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "old secure password with enough length",
    )
    .await;

    let before_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM account_credentials WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read password hash");

    let too_short = app(pool.clone())
        .oneshot(change_password_request(target_id, Some(&token), "too-short"))
        .await
        .unwrap();

    assert_problem(
        too_short,
        StatusCode::UNPROCESSABLE_ENTITY,
        "PASSWORD_POLICY_VIOLATION",
    )
    .await;

    let blocklisted = app(pool.clone())
        .oneshot(change_password_request(
            target_id,
            Some(&token),
            "password-password",
        ))
        .await
        .unwrap();

    assert_problem(
        blocklisted,
        StatusCode::UNPROCESSABLE_ENTITY,
        "PASSWORD_POLICY_VIOLATION",
    )
    .await;

    let after_hash = sqlx::query_scalar::<_, String>(
        "SELECT password_hash FROM account_credentials WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read unchanged password hash");

    assert_eq!(after_hash, before_hash);

    let missing = app(pool.clone())
        .oneshot(change_password_request(
            Uuid::new_v4(),
            Some(&token),
            "new secure password with enough length",
        ))
        .await
        .unwrap();

    assert_problem(missing, StatusCode::NOT_FOUND, "ACCOUNT_NOT_FOUND").await;

    let deleted_id = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(OffsetDateTime::now_utc()),
        "old secure password with enough length",
    )
    .await;

    let deleted = app(pool.clone())
        .oneshot(change_password_request(
            deleted_id,
            Some(&token),
            "new secure password with enough length",
        ))
        .await
        .unwrap();

    assert_problem(deleted, StatusCode::NOT_FOUND, "ACCOUNT_NOT_FOUND").await;

    cleanup_accounts(&pool, &[target_id, deleted_id, admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn account_change_password_requires_change_password_permission() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    digital_publication_backend::MIGRATOR.run(&pool).await.expect("run migrations");

    let (viewer_id, viewer_token) = seed_principal(&pool, "account_viewer").await;
    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        "old secure password with enough length",
    )
    .await;

    let response = app(pool.clone())
        .oneshot(change_password_request(
            target_id,
            Some(&viewer_token),
            "new secure password with enough length",
        ))
        .await
        .unwrap();

    assert_problem(response, StatusCode::FORBIDDEN, "FORBIDDEN").await;

    cleanup_accounts(&pool, &[target_id, viewer_id]).await;
}