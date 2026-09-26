use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use reqwest::{Client, Response};
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

const TEST_PASSWORD: &str = "an extremely secure password";
const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";
const AUTHENTICATION_SESSION_EXPIRES_IN: u64 = 86_400;
const REFRESH_TOKEN_POLICY_EXPIRES_IN: u64 = 2_592_000;

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

    let password_hash = hash_password(SecretString::from(TEST_PASSWORD.to_owned())).unwrap();

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

fn test_router(pool: PgPool) -> Router {
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(app: Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

    let address = listener.local_addr().unwrap();

    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    (address, task)
}

fn refresh_cookie_value(response: &Response) -> String {
    let header = response
        .headers()
        .get("set-cookie")
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

fn refresh_cookie_max_age_seconds(response: &Response) -> u64 {
    let header = response
        .headers()
        .get("set-cookie")
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

#[tokio::test]
#[ignore = "requires PostgreSQL 18 and a built backend"]
async fn login_end_to_end_returns_access_credentials_and_session_bounded_refresh_cookie() {
    let _lock = TEST_DATABASE_LOCK.lock().await;

    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();

    reset(&pool).await;

    let email = format!("e2e-login-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let (address, server) = start_server(test_router(pool.clone())).await;

    let response = Client::new()
        .post(format!("http://{address}/auth/login"))
        .header("content-type", "application/json")
        .json(&json!({
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");

    let refresh_cookie = refresh_cookie_value(&response);
    let refresh_cookie_max_age = refresh_cookie_max_age_seconds(&response);

    assert_eq!(refresh_cookie.len(), 96);
    assert!(refresh_cookie_max_age > 0);
    assert!(refresh_cookie_max_age <= AUTHENTICATION_SESSION_EXPIRES_IN);

    let set_cookie = response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(set_cookie.contains("Path=/"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(!set_cookie.contains("Domain="));

    let body: Value = response.json().await.unwrap();

    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
    assert!(body.get("refresh_token").is_none());
    assert!(body.get("refresh_expires_in").is_none());
    assert!(!body.to_string().contains(&refresh_cookie));

    let access_token = body["access_token"].as_str().unwrap();

    assert_eq!(access_token.len(), 96);

    let access_hash = sqlx::query_scalar::<_, String>(
        "SELECT token_hash \
         FROM authentication_access_token \
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let refresh_hash = sqlx::query_scalar::<_, String>(
        "SELECT token_hash \
         FROM authentication_refresh_token \
         LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(access_hash, sha256_token_verifier(access_token));
    assert_eq!(refresh_hash, sha256_token_verifier(&refresh_cookie));

    let (refresh_expires_at, session_expires_at) =
        sqlx::query_as::<_, (OffsetDateTime, OffsetDateTime)>(
            r#"
            SELECT
                r.expires_at,
                s.expires_at
            FROM authentication_refresh_token AS r
            INNER JOIN authentication_session AS s
                ON s.id = r.session_id
            WHERE r.token_hash = $1
            "#,
        )
        .bind(&refresh_hash)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(refresh_expires_at > OffsetDateTime::now_utc());
    assert!(refresh_expires_at <= session_expires_at);

    let policy_ceiling =
        OffsetDateTime::now_utc() + Duration::seconds(REFRESH_TOKEN_POLICY_EXPIRES_IN as i64);

    assert!(refresh_expires_at <= policy_ceiling);

    assert_ne!(access_hash, access_token);
    assert_ne!(refresh_hash, refresh_cookie);

    server.abort();
    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 and a built backend"]
async fn failed_login_is_generic_and_rate_limited() {
    let _lock = TEST_DATABASE_LOCK.lock().await;

    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();

    reset(&pool).await;

    let email = format!("e2e-rate-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let email_key = sha256_token_verifier(&normalize_email(&email).unwrap().normalized);

    for index in 0..10 {
        sqlx::query(
            "INSERT INTO authentication_login_attempt \
             (email_key, source_ip_key, failed, email_counted, source_ip_counted) \
             VALUES ($1, $2, TRUE, TRUE, TRUE)",
        )
        .bind(&email_key)
        .bind(format!("ip-{index}"))
        .execute(&pool)
        .await
        .unwrap();
    }

    let (address, server) = start_server(test_router(pool.clone())).await;

    let response = Client::new()
        .post(format!("http://{address}/auth/login"))
        .header("content-type", "application/json")
        .json(&json!({
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        reqwest::StatusCode::TOO_MANY_REQUESTS
    );

    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(response.headers().get("www-authenticate").is_none());

    let body: Value = response.json().await.unwrap();

    assert_eq!(body["code"], "AUTHENTICATION_RATE_LIMITED");

    server.abort();
    reset(&pool).await;
}