use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::{hash_password, normalize_email, PasswordBlocklist},
};
use reqwest::Client;
use secrecy::SecretString;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

const TEST_PASSWORD: &str = "an extremely secure password";
const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";

async fn database() -> PgPool {
    PgPool::connect(
        &env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set"),
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

async fn seed_account(
    pool: &PgPool,
    email: &str,
) -> Uuid {
    let email = normalize_email(email).unwrap();

    let password_hash =
        hash_password(
            SecretString::from(TEST_PASSWORD.to_owned()),
        )
        .unwrap();

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
    let blocklist =
        PasswordBlocklist::from_hashes(
            "test",
            Vec::<[u8; 20]>::new(),
        );

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(
    app: Router,
) -> (
    SocketAddr,
    tokio::task::JoinHandle<()>,
) {
    let listener =
        TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();

    let address =
        listener.local_addr().unwrap();

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

fn refresh_cookie_value(
    response: &reqwest::Response,
) -> String {
    let header = response
        .headers()
        .get("set-cookie")
        .expect("login response must set the refresh cookie")
        .to_str()
        .expect("refresh cookie must be valid ASCII");

    let prefix =
        format!("{REFRESH_COOKIE_NAME}=");

    header
        .strip_prefix(&prefix)
        .expect("refresh cookie must use the required __Host- name")
        .split(';')
        .next()
        .expect("refresh cookie must contain a value")
        .to_owned()
}

#[tokio::test]
#[ignore = "requires PostgreSQL 18 and a built backend"]
async fn logout_end_to_end_revokes_the_session() {
    let _lock =
        TEST_DATABASE_LOCK.lock().await;

    let pool = database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();

    reset(&pool).await;

    let email = format!(
        "e2e-logout-{}@example.com",
        Uuid::new_v4()
    );

    seed_account(&pool, &email).await;

    let (address, server) =
        start_server(test_router(pool.clone()))
            .await;

    let client = Client::new();

    let login_response = client
        .post(format!(
            "http://{address}/auth/login"
        ))
        .header(
            "content-type",
            "application/json",
        )
        .json(&json!({
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap();

    let refresh_token =
        refresh_cookie_value(&login_response);

    let login: Value =
        login_response
            .json()
            .await
            .unwrap();

    assert!(
        login
            .get("refresh_token")
            .is_none()
    );

    let access_token =
        login["access_token"]
            .as_str()
            .unwrap();

    let logout = client
        .post(format!(
            "http://{address}/auth/logout"
        ))
        .header(
            "authorization",
            format!("Bearer {access_token}"),
        )
        .send()
        .await
        .unwrap();

    assert_eq!(
        logout.status(),
        reqwest::StatusCode::NO_CONTENT
    );

    assert_eq!(
        logout.headers()["cache-control"],
        "no-store"
    );

    let refresh = client
        .post(format!(
            "http://{address}/auth/refresh"
        ))
        .header(
            "content-type",
            "application/json",
        )
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        refresh.status(),
        reqwest::StatusCode::UNAUTHORIZED
    );

    server.abort();
    reset(&pool).await;
}