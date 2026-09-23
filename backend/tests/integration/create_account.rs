use std::{env, sync::Arc};

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use serde_json::json;
use sha1::{Digest, Sha1};
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use tower::ServiceExt;
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

async fn test_pool() -> Option<PgPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;

    Some(
        PgPool::connect(&database_url)
            .await
            .expect("connect to TEST_DATABASE_URL"),
    )
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
        .expect("reset accounts");
}

async fn seed_principal(
    pool: &PgPool,
    role_name: &str,
) -> (Uuid, String) {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (status, deleted_at)
        VALUES ('active', NULL)
        RETURNING id
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("seed principal account");

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
    .expect("seed principal credentials");

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
        "test-admin-{}-{}-{}",
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

    (account_id, token)
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

fn create_request(token: Option<&str>, email: &str, password: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/admin/accounts")
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = token {
        builder = builder.header(
            header::AUTHORIZATION,
            format!("Bearer {token}"),
        );
    }

    builder
        .body(Body::from(
            json!({
                "email": email,
                "password": password,
                "account_id": Uuid::new_v4(),
                "roles": ["account_admin"],
                "permissions": ["account:create"]
            })
            .to_string(),
        ))
        .unwrap()
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn creates_account_and_credentials_atomically() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");
    reset_test_database(&pool).await;

    let (admin_id, token) = seed_principal(&pool, "account_admin").await;

    let email = format!(
        "AC_UC_01-{}@example.com",
        Uuid::new_v4()
    );
    let password = "a secure password with enough length";

    let response = app(pool.clone())
        .oneshot(create_request(Some(&token), &email, password))
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

    assert_eq!(payload["email"], email);
    assert_eq!(payload["status"], "active");
    assert!(payload["deleted_at"].is_null());
    assert!(payload.get("password").is_none());
    assert!(payload.get("password_hash").is_none());

    let id =
        Uuid::parse_str(payload["id"].as_str().unwrap()).unwrap();

    let account = sqlx::query_as::<
        _,
        (
            Uuid,
            Option<Uuid>,
            Option<Uuid>,
            String,
            Option<OffsetDateTime>,
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
    assert_eq!(account.1, Some(admin_id));
    assert_eq!(account.2, Some(admin_id));
    assert_eq!(account.3, "active");
    assert!(account.4.is_none());

    let credential = sqlx::query_as::<_, (String, String)>(
        "SELECT email, password_hash \
         FROM account_credentials WHERE account_id = $1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(credential.0, email);
    assert!(credential.1.starts_with("$argon2id$v=19$"));

    let token_hash = sqlx::query_scalar::<_, String>(
        "SELECT token_hash FROM authentication_access_token LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    let expected_token_hash = sha256_token_verifier(&token);
    assert_eq!(token_hash.as_deref(), Some(expected_token_hash.as_str()));

    sqlx::query("DELETE FROM authorization_account_role")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authentication_session")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM account")
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_duplicate_email_using_database_uniqueness() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");
    reset_test_database(&pool).await;

    let (admin_id, token) = seed_principal(&pool, "account_admin").await;
    let email = format!("duplicate-{}@example.com", Uuid::new_v4());

    let first = app(pool.clone())
        .oneshot(create_request(
            Some(&token),
            &email,
            "first secure password",
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = app(pool.clone())
        .oneshot(create_request(
            Some(&token),
            &email.to_uppercase(),
            "second secure password",
        ))
        .await
        .unwrap();

    assert_eq!(duplicate.status(), StatusCode::CONFLICT);

    let created_by_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account WHERE created_by = $1",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(created_by_count, 1);

    sqlx::query("DELETE FROM authorization_account_role")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authentication_session")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM account")
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn unauthenticated_create_account_is_rejected_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let response = app(pool)
        .oneshot(create_request(
            None,
            "admin@example.com",
            "a secure password",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
}

#[tokio::test]
async fn malformed_bearer_is_rejected_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();
    let mut request = create_request(
        None,
        "admin@example.com",
        "a secure password",
    );
    request.headers_mut().insert(
        header::AUTHORIZATION,
        "Basic abc".parse().unwrap(),
    );

    let response = app(pool).oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn authenticated_password_validation_is_applied_after_authentication() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");
    reset_test_database(&pool).await;

    let (_, token) = seed_principal(&pool, "account_admin").await;

    let response = app(pool.clone())
        .oneshot(create_request(
            Some(&token),
            "admin@example.com",
            "too-short",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );

    reset_test_database(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn authenticated_principal_without_account_create_permission_is_rejected() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");
    reset_test_database(&pool).await;

    let (viewer_id, token) = seed_principal(&pool, "account_viewer").await;

    let response = app(pool.clone())
        .oneshot(create_request(
            Some(&token),
            "forbidden@example.com",
            "a secure password with enough length",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert!(
        response.headers().get(header::WWW_AUTHENTICATE).is_none()
    );

    let created = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account WHERE created_by = $1",
    )
    .bind(viewer_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(created, 0);

    reset_test_database(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn expired_revoked_disabled_deleted_or_unknown_bearer_is_rejected() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");
    reset_test_database(&pool).await;

    let cases = [
        "expired-token",
        "revoked-token",
        "expired-session-token",
        "disabled-account-token",
        "deleted-account-token",
    ];

    for (index, case) in cases.iter().enumerate() {
        let (account_id, token) = seed_principal(&pool, "account_admin").await;
        let session_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT session_id FROM authentication_access_token WHERE token_hash = $1",
        )
        .bind(sha256_token_verifier(&token))
        .fetch_one(&pool)
        .await
        .unwrap();

        match *case {
            "expired-token" => {
                sqlx::query(
                    "UPDATE authentication_access_token \
                     SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
                     WHERE session_id = $1",
                )
                .bind(session_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "revoked-token" => {
                sqlx::query(
                    "UPDATE authentication_session SET revoked_at = CURRENT_TIMESTAMP \
                     WHERE id = $1",
                )
                .bind(session_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "expired-session-token" => {
                sqlx::query(
                    "UPDATE authentication_session SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
                     WHERE id = $1",
                )
                .bind(session_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "disabled-account-token" => {
                sqlx::query("UPDATE account SET status = 'inactive' WHERE id = $1")
                    .bind(account_id)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            "deleted-account-token" => {
                sqlx::query(
                    "UPDATE account SET status = 'inactive', deleted_at = CURRENT_TIMESTAMP \
                     WHERE id = $1",
                )
                .bind(account_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            _ => unreachable!(),
        }

        let response = app(pool.clone())
            .oneshot(create_request(
                Some(&token),
                &format!("invalid-{index}@example.com"),
                "a secure password with enough length",
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "case={case}"
        );
        assert_eq!(
            response.headers()[header::WWW_AUTHENTICATE],
            r#"Bearer realm="admin-api", error="invalid_token""#
        );
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store"
        );

        reset_test_database(&pool).await;
    }

    let unknown_token = "unknown-token";
    let response = app(pool.clone())
        .oneshot(create_request(
            Some(unknown_token),
            "unknown@example.com",
            "a secure password with enough length",
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );

    reset_test_database(&pool).await;
}