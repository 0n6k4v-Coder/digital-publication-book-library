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
        .expect("response must set the refresh cookie")
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
        .expect("response must set the refresh cookie")
        .to_str()
        .expect("refresh cookie must be valid ASCII");

    header
        .split(';')
        .map(str::trim)
        .find_map(|attribute| attribute.strip_prefix("Max-Age="))
        .and_then(|value| value.parse::<u64>().ok())
        .expect("refresh cookie must contain a valid Max-Age")
}

async fn login(client: &Client, address: SocketAddr, email: &str) -> (Value, String) {
    let response = client
        .post(format!("http://{address}/auth/login"))
        .header("content-type", "application/json")
        .json(&json!({
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap();

    let refresh_token = refresh_cookie_value(&response);
    let body = response.json().await.unwrap();

    (body, refresh_token)
}

#[tokio::test]
async fn refresh_requires_the_browser_managed_cookie() {
    let pool = PgPool::connect_lazy("postgres://invalid").unwrap();

    let (address, server) = start_server(test_router(pool)).await;

    let response = Client::new()
        .post(format!("http://{address}/auth/refresh"))
        .header("content-type", "application/json")
        .json(&json!({
            "refresh_token": "json-refresh-token-must-not-be-used"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(response.headers().get("www-authenticate").is_none());
    assert_eq!(response.headers()["cache-control"], "no-store");

    let body: Value = response.json().await.unwrap();

    assert_eq!(body["code"], "INVALID_REFRESH_TOKEN");

    server.abort();
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 and a built backend"]
async fn refresh_end_to_end_rotates_the_presented_token() {
    let _lock = TEST_DATABASE_LOCK.lock().await;

    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();

    reset(&pool).await;

    let email = format!("e2e-refresh-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let (address, server) = start_server(test_router(pool.clone())).await;

    let client = Client::new();

    let (login_body, old_refresh) = login(&client, address, &email).await;

    assert!(login_body.get("refresh_token").is_none());

    let response = client
        .post(format!("http://{address}/auth/refresh"))
        .header("cookie", format!("{REFRESH_COOKIE_NAME}={old_refresh}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");

    let replacement_refresh = refresh_cookie_value(&response);
    let replacement_cookie_max_age = refresh_cookie_max_age_seconds(&response);

    assert_ne!(replacement_refresh, old_refresh);
    assert!(replacement_cookie_max_age > 0);
    assert!(replacement_cookie_max_age <= AUTHENTICATION_SESSION_EXPIRES_IN);

    let body: Value = response.json().await.unwrap();

    assert!(body.get("refresh_token").is_none());
    assert!(body.get("refresh_expires_in").is_none());
    assert_ne!(body["access_token"], login_body["access_token"]);

    let replacement_hash = sha256_token_verifier(&replacement_refresh);

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
        .bind(&replacement_hash)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(refresh_expires_at > OffsetDateTime::now_utc());
    assert!(refresh_expires_at <= session_expires_at);

    let policy_ceiling =
        OffsetDateTime::now_utc() + Duration::seconds(REFRESH_TOKEN_POLICY_EXPIRES_IN as i64);

    assert!(refresh_expires_at <= policy_ceiling);

    server.abort();
    reset(&pool).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 and a built backend"]
async fn replayed_refresh_end_to_end_returns_401_without_a_bearer_challenge() {
    let _lock = TEST_DATABASE_LOCK.lock().await;

    let pool = database().await;

    sqlx::migrate!().run(&pool).await.unwrap();

    reset(&pool).await;

    let email = format!("e2e-refresh-replay-{}@example.com", Uuid::new_v4());

    seed_account(&pool, &email).await;

    let (address, server) = start_server(test_router(pool.clone())).await;

    let client = Client::new();

    let (_login_body, old_refresh) = login(&client, address, &email).await;

    let first = client
        .post(format!("http://{address}/auth/refresh"))
        .header("cookie", format!("{REFRESH_COOKIE_NAME}={old_refresh}"))
        .send()
        .await
        .unwrap();

    assert_eq!(first.status(), reqwest::StatusCode::OK);

    let replay = client
        .post(format!("http://{address}/auth/refresh"))
        .header("cookie", format!("{REFRESH_COOKIE_NAME}={old_refresh}"))
        .send()
        .await
        .unwrap();

    assert_eq!(replay.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(replay.headers().get("www-authenticate").is_none());
    assert_eq!(replay.headers()["cache-control"], "no-store");

    let body: Value = replay.json().await.unwrap();

    assert_eq!(body["code"], "INVALID_REFRESH_TOKEN");

    server.abort();
    reset(&pool).await;
}