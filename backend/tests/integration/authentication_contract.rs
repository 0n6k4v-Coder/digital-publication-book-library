use std::{
    env,
    net::{IpAddr, SocketAddr},
    num::NonZeroUsize,
    sync::Arc,
};

use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::{header, HeaderValue, Request, StatusCode},
    response::Response,
    Router,
};
use digital_publication_backend::{
    app::{
        router::build_router,
        security::apply_security,
        state::AppState,
    },
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

static TEST_DATABASE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const ALLOWED_ORIGIN: &str = "https://admin.example.com";
const TEST_PASSWORD: &str = "an extremely secure password";
const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    apply_security(
        build_router(AppState::new(
            pool,
            Arc::new(blocklist),
            NonZeroUsize::new(2).unwrap(),
        )),
        vec![HeaderValue::from_static(ALLOWED_ORIGIN)],
    )
}

async fn database() -> PgPool {
    PgPool::connect(
        &env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set"),
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

async fn seed_account(pool: &PgPool, email: &str) -> Uuid {
    let email = normalize_email(email).expect("test email must be valid");
    let password_hash =
        hash_password(SecretString::from(TEST_PASSWORD.to_owned()))
            .expect("test password hashing must succeed");

    sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (status, deleted_at)
            VALUES ('active', NULL)
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
            $1,
            $2,
            $3
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(email.canonical)
    .bind(email.normalized)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .expect("seed account");
    
}

fn login_request(email: &str, source_ip: IpAddr) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::ORIGIN, ALLOWED_ORIGIN)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({
                "email": email,
                "password": TEST_PASSWORD
            })
            .to_string(),
        ))
        .expect("build login request");

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(source_ip, 40000)));

    request
}

fn refresh_request(refresh_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(header::ORIGIN, ALLOWED_ORIGIN)
        .header(
            header::COOKIE,
            format!("{REFRESH_COOKIE_NAME}={refresh_token}"),
        )
        .body(Body::empty())
        .expect("build refresh request")
}

fn logout_request(refresh_token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .header(header::ORIGIN, ALLOWED_ORIGIN);

    if let Some(refresh_token) = refresh_token {
        builder = builder.header(
            header::COOKIE,
            format!("{REFRESH_COOKIE_NAME}={refresh_token}"),
        );
    }

    builder
        .body(Body::empty())
        .expect("build logout request")
}

fn assert_cors_contract(response: &Response) {
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|value| value.to_str().ok()),
        Some(ALLOWED_ORIGIN)
    );

    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
            .and_then(|value| value.to_str().ok()),
        Some("true")
    );

    assert!(
        response
            .headers()
            .get_all(header::VARY)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(','))
            .any(|value| value.trim().eq_ignore_ascii_case("Origin")),
        "response must vary by Origin"
    );
}

fn set_cookie_header(response: &Response) -> &str {
    response
        .headers()
        .get(header::SET_COOKIE)
        .expect("response must include Set-Cookie")
        .to_str()
        .expect("Set-Cookie must be valid ASCII")
}

fn refresh_cookie_value(response: &Response) -> String {
    let cookie = set_cookie_header(response);

    let prefix = format!("{REFRESH_COOKIE_NAME}=");

    cookie
        .strip_prefix(&prefix)
        .expect("refresh cookie must use the required __Host- name")
        .split(';')
        .next()
        .expect("refresh cookie must contain a value")
        .to_owned()
}

fn assert_refresh_cookie_contract(cookie: &str) {
    assert!(
        cookie.starts_with(&format!("{REFRESH_COOKIE_NAME}=")),
        "refresh cookie must use the __Host- prefix"
    );
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(!cookie.contains("Domain="));

    let max_age = cookie
        .split(';')
        .map(str::trim)
        .find_map(|attribute| attribute.strip_prefix("Max-Age="))
        .expect("refresh cookie must contain Max-Age")
        .parse::<i64>()
        .expect("Max-Age must be numeric");

    assert!(max_age > 0, "issued refresh cookie must have positive Max-Age");
}

fn assert_refresh_cookie_cleared(cookie: &str) {
    assert_eq!(
        cookie
            .split(';')
            .map(str::trim)
            .find_map(|attribute| attribute.strip_prefix("Max-Age="))
            .expect("cleared cookie must contain Max-Age"),
        "0"
    );

    assert!(cookie.starts_with(&format!("{REFRESH_COOKIE_NAME}=")));
    assert!(cookie.contains("Path=/"));
    assert!(cookie.contains("Secure"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(!cookie.contains("Domain="));
}

async fn json_body(response: Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("read JSON response body");

    serde_json::from_slice(&body).expect("decode JSON response body")
}

async fn login(
    pool: PgPool,
    email: &str,
    source_ip: IpAddr,
) -> (Value, String) {
    let response = test_router(pool)
        .oneshot(login_request(email, source_ip))
        .await
        .expect("run login request");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert_cors_contract(&response);
    assert_refresh_cookie_contract(set_cookie_header(&response));

    let refresh_token = refresh_cookie_value(&response);
    let body = json_body(response).await;

    (body, refresh_token)
}

#[tokio::test]
async fn refresh_rejects_json_only_credentials() {
    let pool = PgPool::connect_lazy("postgres://invalid")
        .expect("build lazy PostgreSQL pool");

    let response = test_router(pool)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/refresh")
                .header(header::ORIGIN, ALLOWED_ORIGIN)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "refresh_token": "json-only-refresh-token"
                    })
                    .to_string(),
                ))
                .expect("build JSON-only refresh request"),
        )
        .await
        .expect("run JSON-only refresh request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert!(response.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(
        json_body(response).await["code"],
        "INVALID_REFRESH_TOKEN"
    );
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn login_returns_access_credentials_only_and_sets_the_secure_refresh_cookie() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR
        .run(&pool)
        .await
        .expect("run migrations");
    reset(&pool).await;

    let email = format!("contract-login-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let (body, _refresh_token) = login(
        pool.clone(),
        &email,
        "192.0.2.50".parse().expect("valid test IP"),
    )
    .await;

    assert!(body["access_token"].as_str().is_some());
    assert_eq!(body["token_type"], "Bearer");
    assert!(body["expires_in"].as_u64().unwrap_or_default() > 0);
    assert!(body.get("refresh_token").is_none());
    assert!(body.get("refresh_expires_in").is_none());

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn refresh_uses_the_cookie_contract_and_rotates_the_refresh_credential() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR
        .run(&pool)
        .await
        .expect("run migrations");
    reset(&pool).await;

    let email = format!("contract-refresh-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let (login_body, old_refresh) = login(
        pool.clone(),
        &email,
        "192.0.2.51".parse().expect("valid test IP"),
    )
    .await;

    let response = test_router(pool.clone())
        .oneshot(refresh_request(&old_refresh))
        .await
        .expect("run refresh request");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert_cors_contract(&response);

    let new_refresh = refresh_cookie_value(&response);

    assert_ne!(new_refresh, old_refresh);
    assert_refresh_cookie_contract(set_cookie_header(&response));

    let body = json_body(response).await;

    assert_ne!(
        body["access_token"],
        login_body["access_token"]
    );
    assert_eq!(body["token_type"], "Bearer");
    assert!(body["expires_in"].as_u64().unwrap_or_default() > 0);
    assert!(body.get("refresh_token").is_none());
    assert!(body.get("refresh_expires_in").is_none());

    let replay = test_router(pool.clone())
        .oneshot(refresh_request(&old_refresh))
        .await
        .expect("run refresh replay request");

    assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        replay.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert_eq!(
        json_body(replay).await["code"],
        "INVALID_REFRESH_TOKEN"
    );

    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn logout_revokes_the_browser_session_and_clears_the_refresh_cookie() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR
        .run(&pool)
        .await
        .expect("run migrations");
    reset(&pool).await;

    let email = format!("contract-logout-{}@example.com", Uuid::new_v4());

    let account_id = seed_account(&pool, &email).await;

    let (_login_body, refresh_token) = login(
        pool.clone(),
        &email,
        "192.0.2.52".parse().expect("valid test IP"),
    )
    .await;

    let response = test_router(pool.clone())
        .oneshot(logout_request(Some(&refresh_token)))
        .await
        .expect("run logout request");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("no-store"))
    );
    assert_cors_contract(&response);
    assert_refresh_cookie_cleared(set_cookie_header(&response));

    let revoked_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) \
         FROM authentication_session \
         WHERE account_id = $1 \
           AND revoked_at IS NOT NULL",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("count revoked sessions");

    assert_eq!(revoked_count, 1);

    reset(&pool).await;
}