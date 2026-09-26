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
    domains::authentication::{
        extractor::sha256_token_verifier,
        model::{
            ACCESS_TOKEN_EXPIRES_IN,
            AUTHENTICATION_SESSION_EXPIRES_IN,
            REFRESH_TOKEN_POLICY_EXPIRES_IN,
        },
    },
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

static TEST_DATABASE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const TEST_PASSWORD: &str = "an extremely secure password";
const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        NonZeroUsize::new(2).unwrap(),
    ))
}

async fn database() -> PgPool {
    PgPool::connect(&env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set"))
        .await
        .expect("connect to test database")
}

async fn reset(pool: &PgPool) {
    sqlx::query("DELETE FROM authentication_login_attempt")
        .execute(pool)
        .await
        .expect("clear login-attempt state");

    sqlx::query("DELETE FROM authorization_account_role")
        .execute(pool)
        .await
        .expect("clear role assignments");

    sqlx::query("DELETE FROM authentication_session")
        .execute(pool)
        .await
        .expect("clear authentication sessions");

    sqlx::query("DELETE FROM account")
        .execute(pool)
        .await
        .expect("clear accounts");
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    password: &str,
    status: &str,
    deleted: bool,
) -> Uuid {
    let deleted_at = deleted.then(OffsetDateTime::now_utc);

    let email = normalize_email(email).expect("valid test email");
    let password_hash =
        hash_password(SecretString::from(password.to_owned())).expect("hash test password");

    sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (
                status,
                deleted_at
            )
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
            $4,
            $5
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .bind(email.canonical)
    .bind(email.normalized)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .expect("seed account")
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
        .expect("seed failed login attempt");
    }
}

async fn seed_ip_failures(pool: &PgPool, source_ip_key: &str, count: usize) {
    for index in 0..count {
        sqlx::query(
            "INSERT INTO authentication_login_attempt \
             (email_key, source_ip_key, failed, email_counted, source_ip_counted) \
             VALUES ($1, $2, TRUE, TRUE, TRUE)",
        )
        .bind(format!("email-key-{index}"))
        .bind(source_ip_key)
        .execute(pool)
        .await
        .expect("seed source-ip login attempt");
    }
}

fn login_request(email: &str, password: &str, source_ip: IpAddr) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "password": password }).to_string(),
        ))
        .expect("build login request");

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(source_ip, 40000)));

    request
}

fn refresh_cookie_value(response: &axum::response::Response) -> String {
    let header = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("login response must set the refresh cookie")
        .to_str()
        .expect("refresh cookie must be valid ASCII");

    let prefix = format!("{REFRESH_COOKIE_NAME}=");

    header
        .strip_prefix(&prefix)
        .expect("refresh cookie must use the required __Host- name")
        .split(';')
        .next()
        .expect("refresh cookie must contain a value")
        .to_owned()
}

fn refresh_cookie_max_age_seconds(response: &axum::response::Response) -> u64 {
    let header = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("login response must set the refresh cookie")
        .to_str()
        .expect("refresh cookie must be valid ASCII");

    header
        .split(';')
        .map(str::trim)
        .find_map(|attribute| attribute.strip_prefix("Max-Age="))
        .and_then(|value| value.parse::<u64>().ok())
        .expect("refresh cookie must contain a valid Max-Age")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("read response body");

    serde_json::from_slice(&body).expect("decode response JSON")
}

#[tokio::test]
async fn malformed_login_json_is_rejected_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .unwrap();

    let mut request = request;

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(
            Ipv4Addr::LOCALHOST.into(),
            40000,
        )));

    let response = test_router(pool).oneshot(request).await.unwrap();

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
async fn invalid_email_is_rejected_as_an_invalid_request_without_database_access() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let response = test_router(pool)
        .oneshot(login_request(
            "not-an-email",
            TEST_PASSWORD,
            "192.0.2.9".parse().unwrap(),
        ))
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
async fn successful_login_creates_session_and_token_verifiers_with_defined_lifetimes() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("login-{}@example.com", Uuid::new_v4());

    let account_id =
        seed_account(&pool, &email, TEST_PASSWORD, "active", false).await;

    let source_ip: IpAddr = "192.0.2.10".parse().unwrap();

    let response = test_router(pool.clone())
        .oneshot(login_request(&email, TEST_PASSWORD, source_ip))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );

    let refresh_cookie = refresh_cookie_value(&response);
    let refresh_cookie_max_age = refresh_cookie_max_age_seconds(&response);

    assert_eq!(refresh_cookie.len(), 96);
    assert!(refresh_cookie_max_age > 0);
    assert!(
        refresh_cookie_max_age <= AUTHENTICATION_SESSION_EXPIRES_IN
    );

    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();

    assert!(set_cookie.contains("Path=/"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(!set_cookie.contains("Domain="));

    let body = response_json(response).await;
    let access_token = body["access_token"].as_str().unwrap();

    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], ACCESS_TOKEN_EXPIRES_IN);
    assert!(body.get("refresh_token").is_none());
    assert!(body.get("refresh_expires_in").is_none());
    assert!(!body.to_string().contains(&refresh_cookie));
    assert_eq!(access_token.len(), 96);

    let session = sqlx::query_as::<_, (Uuid, i64, OffsetDateTime)>(
        "SELECT id, \
                EXTRACT(EPOCH FROM (expires_at - created_at))::BIGINT, \
                last_authenticated_at \
         FROM authentication_session \
         WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(session.1, AUTHENTICATION_SESSION_EXPIRES_IN as i64);
    assert!(session.2 <= OffsetDateTime::now_utc());

    let access = sqlx::query_as::<_, (String, i64)>(
        "SELECT token_hash, \
                EXTRACT(EPOCH FROM (expires_at - created_at))::BIGINT \
         FROM authentication_access_token \
         WHERE session_id = $1",
    )
    .bind(session.0)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(access.0, sha256_token_verifier(access_token));
    assert_ne!(access.0, access_token);
    assert_eq!(access.1, ACCESS_TOKEN_EXPIRES_IN as i64);

    let refresh = sqlx::query_as::<_, (String, i64)>(
        "SELECT token_hash, \
                EXTRACT(EPOCH FROM (expires_at - created_at))::BIGINT \
         FROM authentication_refresh_token \
         WHERE session_id = $1",
    )
    .bind(session.0)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(refresh.0, sha256_token_verifier(&refresh_cookie));
    assert_ne!(refresh.0, refresh_cookie);
    assert!(refresh.1 <= AUTHENTICATION_SESSION_EXPIRES_IN as i64);
    assert!(refresh.1 <= REFRESH_TOKEN_POLICY_EXPIRES_IN as i64);

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn invalid_missing_inactive_and_deleted_accounts_return_generic_credentials_error() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let source_ip: IpAddr = "192.0.2.11".parse().unwrap();

    let missing_email = format!("missing-{}@example.com", Uuid::new_v4());
    let inactive_email = format!("inactive-{}@example.com", Uuid::new_v4());
    let deleted_email = format!("deleted-{}@example.com", Uuid::new_v4());

    seed_account(
        &pool,
        &inactive_email,
        TEST_PASSWORD,
        "inactive",
        false,
    )
    .await;

    seed_account(
        &pool,
        &deleted_email,
        TEST_PASSWORD,
        "inactive",
        true,
    )
    .await;

    for email in [missing_email, inactive_email, deleted_email] {
        let response = test_router(pool.clone())
            .oneshot(login_request(
                &email,
                TEST_PASSWORD,
                source_ip,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-store"
        );
        assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());

        let body = response_json(response).await;

        assert_eq!(body["code"], "INVALID_CREDENTIALS");
        assert_eq!(
            body["detail"],
            "The supplied credentials are invalid."
        );
    }

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn failed_password_authentication_is_generic_and_counted() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("wrong-password-{}@example.com", Uuid::new_v4());
    seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
        "active",
        false,
    )
    .await;

    let source_ip: IpAddr = "203.0.113.10".parse().unwrap();

    let response = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            "the wrong password",
            source_ip,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
    assert_eq!(
        response_json(response).await["code"],
        "INVALID_CREDENTIALS"
    );

    let email_key =
        sha256_token_verifier(&normalize_email(&email).unwrap().normalized);
    let source_ip_key =
        sha256_token_verifier(&source_ip.to_string());

    let counts = sqlx::query_as::<_, (i64, i64)>(
        "SELECT \
            COUNT(*) FILTER (WHERE email_key = $1 AND email_counted = TRUE), \
            COUNT(*) FILTER (WHERE source_ip_key = $2 AND source_ip_counted = TRUE) \
         FROM authentication_login_attempt",
    )
    .bind(email_key)
    .bind(source_ip_key)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(counts, (1, 1));

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn login_rate_limits_email_and_source_ip_before_password_verification() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("rate-{}@example.com", Uuid::new_v4());
    seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
        "active",
        false,
    )
    .await;

    let normalized_email = normalize_email(&email).unwrap().normalized;
    let email_key = sha256_token_verifier(&normalized_email);

    seed_failed_attempts(
        &pool,
        &email_key,
        "source-ip",
        10,
    )
    .await;

    let response = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            TEST_PASSWORD,
            "198.51.100.20".parse().unwrap(),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
    assert_eq!(
        response_json(response).await["code"],
        "AUTHENTICATION_RATE_LIMITED"
    );

    reset(&pool).await;

    let email = format!("ip-rate-{}@example.com", Uuid::new_v4());

    seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
        "active",
        false,
    )
    .await;

    let source_ip: IpAddr = "198.51.100.21".parse().unwrap();
    let source_ip_key = sha256_token_verifier(&source_ip.to_string());

    seed_ip_failures(&pool, &source_ip_key, 50).await;

    let response = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            TEST_PASSWORD,
            source_ip,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        response_json(response).await["code"],
        "AUTHENTICATION_RATE_LIMITED"
    );

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn successful_login_resets_email_failures_but_preserves_existing_source_ip_failures() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("reset-{}@example.com", Uuid::new_v4());
    seed_account(
        &pool,
        &email,
        TEST_PASSWORD,
        "active",
        false,
    )
    .await;

    let normalized_email =
        normalize_email(&email).unwrap().normalized;
    let email_key = sha256_token_verifier(&normalized_email);

    let source_ip: IpAddr = "203.0.113.25".parse().unwrap();
    let source_ip_key = sha256_token_verifier(&source_ip.to_string());

    sqlx::query(
        "INSERT INTO authentication_login_attempt \
         (email_key, source_ip_key, failed, email_counted, source_ip_counted) \
         VALUES ($1, $2, TRUE, TRUE, TRUE)",
    )
    .bind(&email_key)
    .bind(&source_ip_key)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO authentication_login_attempt \
         (email_key, source_ip_key, failed, email_counted, source_ip_counted) \
         VALUES ($1, $2, TRUE, TRUE, TRUE)",
    )
    .bind(&email_key)
    .bind(&source_ip_key)
    .execute(&pool)
    .await
    .unwrap();

    let response = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            TEST_PASSWORD,
            source_ip,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let email_failures = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) \
         FROM authentication_login_attempt \
         WHERE email_key = $1 \
           AND failed = TRUE \
           AND email_counted = TRUE",
    )
    .bind(&email_key)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(email_failures, 0);

    let ip_failures = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) \
         FROM authentication_login_attempt \
         WHERE source_ip_key = $1 \
           AND failed = TRUE \
           AND source_ip_counted = TRUE",
    )
    .bind(&source_ip_key)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(ip_failures, 2);

    reset(&pool).await;
}