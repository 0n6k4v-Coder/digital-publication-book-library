use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
    sync::Arc,
};

use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::{header, Request, StatusCode},
    Router,
};
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

static TEST_DATABASE_LOCK: tokio::sync::Mutex<()> =
    tokio::sync::Mutex::const_new(());

const TEST_PASSWORD: &str = "an extremely secure password";

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        Vec::<[u8; 20]>::new(),
    );

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        NonZeroUsize::new(2).unwrap(),
    ))
}

async fn database() -> PgPool {
    PgPool::connect(
        &env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set"),
    )
    .await
    .expect("connect to test database")
}

async fn reset(pool: &PgPool) {
    sqlx::query("DELETE FROM authentication_login_attempt")
        .execute(pool)
        .await
        .expect("clear login attempts");

    sqlx::query("DELETE FROM authorization_account_role")
        .execute(pool)
        .await
        .expect("clear role assignments");

    sqlx::query("DELETE FROM authentication_session")
        .execute(pool)
        .await
        .expect("clear sessions");

    sqlx::query("DELETE FROM account")
        .execute(pool)
        .await
        .expect("clear accounts");
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Uuid {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO account (status, deleted_at) \
         VALUES ('active', NULL) \
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    let email = normalize_email(email).unwrap();
    let password_hash =
        hash_password(SecretString::from(password.to_owned())).unwrap();

    sqlx::query(
        "INSERT INTO account_credentials \
         (account_id, email, email_normalized, password_hash) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(account_id)
    .bind(email.canonical)
    .bind(email.normalized)
    .bind(password_hash)
    .execute(pool)
    .await
    .unwrap();

    account_id
}

async fn seed_failed_attempts(
    pool: &PgPool,
    email_key: &str,
    source_ip_key: &str,
    count: usize,
) {
    for index in 0..count {
        sqlx::query(
            "INSERT INTO authentication_login_attempt \
             (email_key, source_ip_key, failed, email_counted, source_ip_counted) \
             VALUES ($1, $2, TRUE, TRUE, TRUE)",
        )
        .bind(email_key)
        .bind(format!("{source_ip_key}-{index}"))
        .execute(pool)
        .await
        .unwrap();
    }
}

fn login_request(
    email: &str,
    password: &str,
    source_ip: IpAddr,
) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            Body::from(
                json!({
                    "email": email,
                    "password": password
                })
                .to_string(),
            ),
        )
        .unwrap();

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(
            source_ip,
            40000,
        )));

    request
}

fn refresh_request(refresh_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            Body::from(
                json!({
                    "refresh_token": refresh_token
                })
                .to_string(),
            ),
        )
        .unwrap()
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(
        response.into_body(),
        64 * 1024,
    )
    .await
    .unwrap();

    serde_json::from_slice(&body).unwrap()
}

async fn login(
    pool: PgPool,
    email: &str,
    source_ip: IpAddr,
) -> Value {
    let response = test_router(pool)
        .oneshot(login_request(
            email,
            TEST_PASSWORD,
            source_ip,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    json_body(response).await
}

#[tokio::test]
async fn malformed_refresh_json_is_rejected_without_database_access() {
    let pool =
        sqlx::PgPool::connect_lazy("postgres://invalid")
            .unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .unwrap();

    let response = test_router(pool)
        .oneshot(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn refresh_rotates_tokens_and_preserves_absolute_session_state() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("refresh-{}@example.com", Uuid::new_v4());
    let account_id = seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
    )
    .await;

    let source_ip: IpAddr = "192.0.2.30"
        .parse()
        .unwrap();

    let login_body = login(
        pool.clone(),
        &email,
        source_ip,
    )
    .await;

    let old_refresh =
        login_body["refresh_token"]
            .as_str()
            .unwrap()
            .to_owned();

    let old_refresh_hash =
        sha256_token_verifier(&old_refresh);

    let session_before =
        sqlx::query_as::<_, (Uuid, OffsetDateTime, OffsetDateTime)>(
            "SELECT id, expires_at, last_authenticated_at \
             FROM authentication_session \
             WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let email_key =
        sha256_token_verifier(
            &normalize_email(&email).unwrap().normalized,
        );

    let source_ip_key =
        sha256_token_verifier(&source_ip.to_string());

    seed_failed_attempts(
        &pool,
        &email_key,
        &source_ip_key,
        10,
    )
    .await;

    let response = test_router(pool.clone())
        .oneshot(refresh_request(&old_refresh))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );

    let body = json_body(response).await;

    let new_access =
        body["access_token"].as_str().unwrap();

    let new_refresh =
        body["refresh_token"].as_str().unwrap();

    assert_ne!(
        new_access,
        login_body["access_token"].as_str().unwrap()
    );

    assert_ne!(new_refresh, old_refresh);

    let session_after =
        sqlx::query_as::<_, (OffsetDateTime, OffsetDateTime)>(
            "SELECT expires_at, last_authenticated_at \
             FROM authentication_session \
             WHERE id = $1",
        )
        .bind(session_before.0)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(session_after.0, session_before.1);
    assert_eq!(session_after.1, session_before.2);

    let old_refresh_row =
        sqlx::query_scalar::<_, Option<OffsetDateTime>>(
            "SELECT used_at \
             FROM authentication_refresh_token \
             WHERE token_hash = $1",
        )
        .bind(&old_refresh_hash)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(old_refresh_row.is_some());

    let new_refresh_hash =
        sha256_token_verifier(new_refresh);

    let replacement_count =
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) \
             FROM authentication_refresh_token \
             WHERE token_hash = $1 \
               AND session_id = $2",
        )
        .bind(new_refresh_hash)
        .bind(session_before.0)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(replacement_count, 1);

    let email_failure_count =
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) \
             FROM authentication_login_attempt \
             WHERE email_key = $1 \
               AND email_counted = TRUE",
        )
        .bind(email_key)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(email_failure_count, 10);

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn replayed_refresh_token_is_rejected_without_issuing_more_credentials() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!(
        "replay-{}@example.com",
        Uuid::new_v4()
    );

    seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
    )
    .await;

    let login_body = login(
        pool.clone(),
        &email,
        "192.0.2.31".parse().unwrap(),
    )
    .await;

    let refresh_token =
        login_body["refresh_token"]
            .as_str()
            .unwrap()
            .to_owned();

    let first = test_router(pool.clone())
        .oneshot(refresh_request(&refresh_token))
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::OK);

    let token_count_before =
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) \
             FROM authentication_access_token",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

    let replay = test_router(pool.clone())
        .oneshot(refresh_request(&refresh_token))
        .await
        .unwrap();

    assert_eq!(
        replay.status(),
        StatusCode::UNAUTHORIZED
    );

    assert_eq!(
        replay.headers()[header::CACHE_CONTROL],
        "no-store"
    );

    assert!(
        replay.headers()
            .get(header::WWW_AUTHENTICATE)
            .is_none()
    );

    assert_eq!(
        json_body(replay).await["code"],
        "INVALID_REFRESH_TOKEN"
    );

    let token_count_after =
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) \
             FROM authentication_access_token",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        token_count_after,
        token_count_before
    );

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn expired_revoked_or_account_invalid_refresh_tokens_are_rejected() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();
    reset(&pool).await;

    for case in [
        "expired",
        "revoked",
        "expired_session",
        "revoked_session",
        "inactive_account",
        "deleted_account",
    ] {
        let email =
            format!("{case}-{}@example.com", Uuid::new_v4());

        let account_id = seed_account(
            &pool,
            &email,
            TEST_PASSWORD,
        )
        .await;

        let login_body = login(
            pool.clone(),
            &email,
            "192.0.2.32".parse().unwrap(),
        )
        .await;

        let refresh_token =
            login_body["refresh_token"]
                .as_str()
                .unwrap()
                .to_owned();

        let session_id =
            sqlx::query_scalar::<_, Uuid>(
                "SELECT id \
                 FROM authentication_session \
                 WHERE account_id = $1",
            )
            .bind(account_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        let refresh_hash =
            sqlx::query_scalar::<_, String>(
                "SELECT token_hash \
                 FROM authentication_refresh_token \
                 WHERE session_id = $1 \
                   AND used_at IS NULL",
            )
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        match case {
            "expired" => {
                sqlx::query(
                    "UPDATE authentication_refresh_token \
                     SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
                     WHERE token_hash = $1",
                )
                .bind(&refresh_hash)
                .execute(&pool)
                .await
                .unwrap();
            }
            "revoked" => {
                sqlx::query(
                    "UPDATE authentication_refresh_token \
                     SET revoked_at = CURRENT_TIMESTAMP \
                     WHERE token_hash = $1",
                )
                .bind(&refresh_hash)
                .execute(&pool)
                .await
                .unwrap();
            }
            "expired_session" => {
                sqlx::query(
                    "UPDATE authentication_session \
                     SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 second' \
                     WHERE id = $1",
                )
                .bind(session_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "revoked_session" => {
                sqlx::query(
                    "UPDATE authentication_session \
                     SET revoked_at = CURRENT_TIMESTAMP \
                     WHERE id = $1",
                )
                .bind(session_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "inactive_account" => {
                sqlx::query(
                    "UPDATE account \
                     SET status = 'inactive' \
                     WHERE id = $1",
                )
                .bind(account_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            "deleted_account" => {
                sqlx::query(
                    "UPDATE account \
                     SET status = 'inactive', \
                         deleted_at = CURRENT_TIMESTAMP \
                     WHERE id = $1",
                )
                .bind(account_id)
                .execute(&pool)
                .await
                .unwrap();
            }
            _ => unreachable!(),
        }

        let response = test_router(pool.clone())
            .oneshot(refresh_request(&refresh_token))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED
        );

        reset(&pool).await;
    }
}