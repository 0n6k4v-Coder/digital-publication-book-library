use std::{
    env,
    net::{IpAddr, SocketAddr},
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
        repository::AuthenticationRepository,
    },
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
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
    PgPool::connect(
        &env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set"),
    )
    .await
    .unwrap()
}

async fn reset(pool: &PgPool) {
    sqlx::query("DELETE FROM authentication_login_attempt")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("DELETE FROM authorization_account_role")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("DELETE FROM authentication_session")
        .execute(pool)
        .await
        .unwrap();

    sqlx::query("DELETE FROM account")
        .execute(pool)
        .await
        .unwrap();
}

async fn seed_account(pool: &PgPool, email: &str) -> Uuid {
    let email = normalize_email(email).unwrap();

    let password_hash =
        hash_password(SecretString::from(TEST_PASSWORD.to_owned())).unwrap();

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
    .unwrap()
}

fn login_request(email: &str, source_ip: IpAddr) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(
            Body::from(
                json!({
                    "email": email,
                    "password": TEST_PASSWORD
                })
                .to_string(),
            ),
        )
        .unwrap();

    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(source_ip, 40000)));

    request
}

fn refresh_request(refresh_token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/refresh")
        .header(
            header::COOKIE,
            format!("{REFRESH_COOKIE_NAME}={refresh_token}"),
        )
        .body(Body::empty())
        .unwrap()
}

fn logout_request(refresh_token: Option<&str>) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/logout")
        .body(Body::empty())
        .unwrap();

    if let Some(refresh_token) = refresh_token {
        request.headers_mut().insert(
            header::COOKIE,
            format!("{REFRESH_COOKIE_NAME}={refresh_token}")
                .parse()
                .unwrap(),
        );
    }

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

fn assert_refresh_cookie_deleted(response: &axum::response::Response) {
    let header = response
        .headers()
        .get(header::SET_COOKIE)
        .expect("logout response must clear the refresh cookie")
        .to_str()
        .expect("logout cookie must be valid ASCII");

    assert!(header.starts_with("__Host-refresh_token="));
    assert!(header.contains("Max-Age=0"));
    assert!(header.contains("Path=/"));
    assert!(header.contains("Secure"));
    assert!(header.contains("HttpOnly"));
    assert!(header.contains("SameSite=Strict"));
    assert!(!header.contains("Domain="));
}

async fn json_body(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .unwrap();

    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn logout_without_refresh_cookie_is_idempotent() {
    let pool = sqlx::PgPool::connect_lazy("postgres://invalid").unwrap();

    let response = test_router(pool)
        .oneshot(logout_request(None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());

    assert_refresh_cookie_deleted(&response);
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 configured through TEST_DATABASE_URL and applied migrations"]
async fn logout_revokes_only_the_current_session_and_invalidates_its_access_token() {
    let _lock = TEST_DATABASE_LOCK.lock().await;
    let pool = database().await;

    digital_publication_backend::MIGRATOR.run(&pool).await.unwrap();
    reset(&pool).await;

    let email = format!("logout-{}@example.com", Uuid::new_v4());

    let account_id = seed_account(&pool, &email).await;

    let first_login = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            "192.0.2.40".parse().unwrap(),
        ))
        .await
        .unwrap();

    let first_refresh = refresh_cookie_value(&first_login);
    let first_body = json_body(first_login).await;
    let first_access = first_body["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let second_login = test_router(pool.clone())
        .oneshot(login_request(
            &email,
            "192.0.2.41".parse().unwrap(),
        ))
        .await
        .unwrap();

    let second_body = json_body(second_login).await;
    let second_access = second_body["access_token"]
        .as_str()
        .unwrap()
        .to_owned();

    let logout = test_router(pool.clone())
        .oneshot(logout_request(Some(&first_refresh)))
        .await
        .unwrap();

    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    assert_eq!(logout.headers()[header::CACHE_CONTROL], "no-store");
    assert!(logout.headers().get(header::WWW_AUTHENTICATE).is_none());
    assert_refresh_cookie_deleted(&logout);

    let revoked_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) \
         FROM authentication_session \
         WHERE account_id = $1 \
           AND revoked_at IS NOT NULL",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(revoked_count, 1);

    let first_refresh_response = test_router(pool.clone())
        .oneshot(refresh_request(&first_refresh))
        .await
        .unwrap();

    assert_eq!(
        first_refresh_response.status(),
        StatusCode::UNAUTHORIZED
    );
    assert!(first_refresh_response
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .is_none());
    assert_eq!(
        json_body(first_refresh_response).await["code"],
        "INVALID_REFRESH_TOKEN"
    );

    let authentication_repository = AuthenticationRepository::new(pool.clone());

    assert!(
        authentication_repository
            .validate_access_token(&sha256_token_verifier(&first_access))
            .await
            .unwrap()
            .is_none()
    );

    assert!(
        authentication_repository
            .validate_access_token(&sha256_token_verifier(&second_access))
            .await
            .unwrap()
            .is_some()
    );

    reset(&pool).await;
}