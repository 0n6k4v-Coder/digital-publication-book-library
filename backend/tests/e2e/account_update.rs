use std::{env, net::SocketAddr, sync::Arc};

use axum::Router;
use reqwest::{Client, Response};
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::{net::TcpListener, sync::Mutex};
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

async fn connect_database() -> PgPool {
    let database_url =
        env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

    PgPool::connect(&database_url)
        .await
        .expect("connect to test database")
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    display_name: Option<&str>,
    status: &str,
    deleted_at: Option<OffsetDateTime>,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (
                status,
                display_name,
                deleted_at,
                created_by,
                updated_by
            )
            VALUES ($1, $2, $3, $4, $5)
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
            $6,
            $6,
            $7
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(status)
    .bind(display_name)
    .bind(deleted_at)
    .bind(created_by)
    .bind(updated_by)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .fetch_one(pool)
    .await
    .expect("seed account")
}

async fn seed_principal(pool: &PgPool, role_name: &str) -> (Uuid, String) {
    let account_id = seed_account(
        pool,
        &format!("e2e-principal-{}@example.com", Uuid::new_v4()),
        None,
        "active",
        None,
        None,
        None,
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
        "e2e-update-{}-{}-{}",
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

fn test_router(pool: PgPool) -> Router {
    let blocklist =
        PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

async fn start_server(app: Router) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind e2e server");

    let address = listener
        .local_addr()
        .expect("read e2e server address");

    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve e2e application");
    });

    (address, task)
}

async fn assert_problem(response: Response, expected_status: reqwest::StatusCode) {
    assert_eq!(response.status(), expected_status);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/problem+json")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let _: Value = response
        .json()
        .await
        .expect("decode problem response");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn update_account_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) =
        seed_principal(&pool, "account_admin").await;

    let target_id = seed_account(
        &pool,
        &format!("e2e-target-{}@example.com", Uuid::new_v4()),
        Some("Old Name"),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let client = Client::new();

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth(&admin_token)
        .header("content-type", "application/merge-patch+json")
        .body(r#"{"display_name":"  Cafe\u0301  "}"#)
        .send()
        .await
        .expect("send update request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );

    let response_body: Value = response
        .json()
        .await
        .expect("decode account response");

    assert_eq!(response_body["id"], target_id.to_string());
    assert!(response_body.get("password").is_none());
    assert!(response_body.get("password_hash").is_none());

    let stored_name = sqlx::query_scalar::<_, Option<String>>(
        "SELECT display_name FROM account WHERE id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read updated display name");

    assert_eq!(stored_name.as_deref(), Some("Café"));

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth(&admin_token)
        .header("content-type", "application/merge-patch+json")
        .body(r#"{"display_name":null}"#)
        .send()
        .await
        .expect("send clear-display-name request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let stored_name = sqlx::query_scalar::<_, Option<String>>(
        "SELECT display_name FROM account WHERE id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("read cleared display name");

    assert!(stored_name.is_none());

    server.abort();
    cleanup_accounts(&pool, &[target_id, admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn update_account_authentication_and_authorization_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) =
        seed_principal(&pool, "account_admin").await;
    let (viewer_id, viewer_token) =
        seed_principal(&pool, "account_viewer").await;

    let target_id = seed_account(
        &pool,
        &format!("e2e-security-target-{}@example.com", Uuid::new_v4()),
        Some("Initial"),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let client = Client::new();

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .header("content-type", "application/merge-patch+json")
        .body(r#"{"display_name":"Changed"}"#)
        .send()
        .await
        .expect("send unauthenticated request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api""#)
    );

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth(&viewer_token)
        .header("content-type", "application/merge-patch+json")
        .body(
            r#"{"display_name":"Changed","roles":["account_admin"],"permissions":["account:update"]}"#,
        )
        .send()
        .await
        .expect("send unauthorized request");

    assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
    assert!(response.headers().get("www-authenticate").is_none());
    assert_problem(response, reqwest::StatusCode::FORBIDDEN).await;

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth("invalid-e2e-update-token")
        .header("content-type", "application/merge-patch+json")
        .body(r#"{"display_name":"Changed"}"#)
        .send()
        .await
        .expect("send invalid-token request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        response
            .headers()
            .get("www-authenticate")
            .and_then(|value| value.to_str().ok()),
        Some(r#"Bearer realm="admin-api", error="invalid_token""#)
    );

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth(&admin_token)
        .header("content-type", "application/json")
        .body(r#"{"display_name":"Changed"}"#)
        .send()
        .await
        .expect("send wrong-content-type request");

    assert_problem(
        response,
        reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE,
    )
    .await;

    server.abort();
    cleanup_accounts(
        &pool,
        &[target_id, admin_id, viewer_id],
    )
    .await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL and a built backend"]
async fn update_account_validation_end_to_end() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let pool = connect_database().await;

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) =
        seed_principal(&pool, "account_admin").await;

    let target_id = seed_account(
        &pool,
        &format!(
            "e2e-validation-target-{}@example.com",
            Uuid::new_v4()
        ),
        Some("Initial"),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let (address, server) =
        start_server(test_router(pool.clone())).await;

    let client = Client::new();

    for body in [
        r#"{}"#,
        r#"{"unknown":"field"}"#,
        r#"{"display_name":""}"#,
        r#"{"display_name":"   "}"#,
        r#"{"display_name":123}"#,
    ] {
        let response = client
            .patch(format!(
                "http://{address}/admin/accounts/{target_id}"
            ))
            .bearer_auth(&admin_token)
            .header(
                "content-type",
                "application/merge-patch+json",
            )
            .body(body)
            .send()
            .await
            .expect("send validation request");

        assert_problem(
            response,
            reqwest::StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    }

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{target_id}"
        ))
        .bearer_auth(&admin_token)
        .header(
            "content-type",
            "application/merge-patch+json",
        )
        .body(r#"{"display_name":"Changed""#)
        .send()
        .await
        .expect("send malformed-json request");

    assert_problem(
        response,
        reqwest::StatusCode::BAD_REQUEST,
    )
    .await;

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/not-a-uuid"
        ))
        .bearer_auth(&admin_token)
        .header(
            "content-type",
            "application/merge-patch+json",
        )
        .body(r#"{"display_name":"Changed"}"#)
        .send()
        .await
        .expect("send invalid-id request");

    assert_problem(
        response,
        reqwest::StatusCode::BAD_REQUEST,
    )
    .await;

    let deleted_id = seed_account(
        &pool,
        &format!(
            "e2e-deleted-{}@example.com",
            Uuid::new_v4()
        ),
        Some("Deleted"),
        "inactive",
        Some(OffsetDateTime::now_utc()),
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let response = client
        .patch(format!(
            "http://{address}/admin/accounts/{deleted_id}"
        ))
        .bearer_auth(&admin_token)
        .header(
            "content-type",
            "application/merge-patch+json",
        )
        .body(r#"{"display_name":"Changed"}"#)
        .send()
        .await
        .expect("send deleted-account request");

    assert_problem(
        response,
        reqwest::StatusCode::NOT_FOUND,
    )
    .await;

    server.abort();
    cleanup_accounts(
        &pool,
        &[target_id, deleted_id, admin_id],
    )
    .await;
}