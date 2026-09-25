use std::{env, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    response::Response,
    Router,
};
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::{sync::Mutex, task::JoinSet};
use tower::ServiceExt;
use uuid::Uuid;

use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    domains::authentication::extractor::sha256_token_verifier,
    shared::validation::PasswordBlocklist,
};

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

async fn test_pool() -> Option<PgPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;

    Some(
        PgPool::connect(&database_url)
            .await
            .expect("connect to test database"),
    )
}

async fn seed_account(
    pool: &PgPool,
    email: &str,
    status: &str,
    deleted_at: Option<OffsetDateTime>,
    updated_by: Option<Uuid>,
    deleted_by: Option<Uuid>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH inserted_account AS (
            INSERT INTO account (
                status,
                deleted_at,
                created_by,
                updated_by,
                deleted_by
            )
            VALUES ($1, $2, $3, $3, $4)
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
            $5,
            $5,
            $6
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .bind(updated_by)
    .bind(deleted_by)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .fetch_one(pool)
    .await
    .expect("seed account")
}

async fn assign_role(pool: &PgPool, account_id: Uuid, role_name: &str) {
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
    .expect("assign authorization role");
}

async fn seed_access_token(pool: &PgPool, account_id: Uuid) -> String {
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
        "test-soft-delete-target-{}-{}-{}",
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

    token
}

async fn seed_principal(pool: &PgPool, role_name: &str) -> (Uuid, String) {
    let account_id = seed_account(
        pool,
        &format!("principal-{}@example.com", Uuid::new_v4()),
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
        "test-soft-delete-principal-{}-{}-{}",
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
    .expect("seed principal access token");

    assign_role(pool, account_id, role_name).await;

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
    let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

    build_router(AppState::new(
        pool,
        Arc::new(blocklist),
        std::num::NonZeroUsize::new(2).unwrap(),
    ))
}

fn delete_request(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("DELETE").uri(uri);

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    builder.body(Body::empty()).expect("build delete request")
}

async fn assert_problem(response: Response, expected_status: StatusCode) -> Value {
    assert_eq!(response.status(), expected_status);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let body = to_bytes(response.into_body(), 32 * 1024)
        .await
        .expect("read problem response");

    serde_json::from_slice(&body).expect("decode problem response")
}

#[derive(Debug, sqlx::FromRow)]
struct AccountSnapshot {
    created_at: OffsetDateTime,
    created_by: Option<Uuid>,
    updated_at: OffsetDateTime,
    updated_by: Option<Uuid>,
    status: String,
    deleted_at: Option<OffsetDateTime>,
    deleted_by: Option<Uuid>,
    email: String,
    password_hash: String,
    credentials_updated_at: OffsetDateTime,
}

async fn snapshot(pool: &PgPool, account_id: Uuid) -> AccountSnapshot {
    sqlx::query_as::<_, AccountSnapshot>(
        r#"
        SELECT
            a.created_at,
            a.created_by,
            a.updated_at,
            a.updated_by,
            a.status,
            a.deleted_at,
            a.deleted_by,
            ac.email,
            ac.password_hash,
            ac.updated_at AS credentials_updated_at
        FROM account AS a
        INNER JOIN account_credentials AS ac
            ON ac.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("fetch account snapshot")
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn soft_deletes_account_and_invalidates_existing_authentication() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;

    let target_id = seed_account(
        &pool,
        &format!("target-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        None,
    )
    .await;

    assign_role(&pool, target_id, "account_admin").await;

    let target_token = seed_access_token(&pool, target_id).await;
    let before = snapshot(&pool, target_id).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{target_id}"),
            Some(&admin_token),
        ))
        .await
        .expect("send soft-delete request");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "no-store"
    );
    assert!(response.headers().get(header::CONTENT_TYPE).is_none());

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("read empty soft-delete response");

    assert!(body.is_empty());

    let after = snapshot(&pool, target_id).await;

    assert_eq!(after.created_at, before.created_at);
    assert_eq!(after.created_by, before.created_by);
    assert_eq!(after.email, before.email);
    assert_eq!(after.password_hash, before.password_hash);
    assert_eq!(after.credentials_updated_at, before.credentials_updated_at);
    assert_eq!(after.status, "inactive");
    assert!(after.deleted_at.is_some());
    assert_eq!(after.deleted_by, Some(admin_id));
    assert_eq!(after.updated_by, Some(admin_id));
    assert!(after.updated_at > before.updated_at);

    let role_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authorization_account_role WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("check retained authorization role");

    assert_eq!(role_count, 1);

    let response = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/admin/accounts/{target_id}"))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {admin_token}"),
                )
                .body(Body::empty())
                .expect("build deleted-account view request"),
        )
        .await
        .expect("send deleted-account view request");

    assert_problem(response, StatusCode::NOT_FOUND).await;

    let response = app(pool.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/admin/accounts")
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {target_token}"),
                )
                .body(Body::empty())
                .expect("build post-delete authentication request"),
        )
        .await
        .expect("send post-delete authentication request");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    let inactive_target_id = seed_account(
        &pool,
        &format!("inactive-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
        Some(admin_id),
        None,
    )
    .await;

    assign_role(&pool, inactive_target_id, "account_admin").await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{inactive_target_id}"),
            Some(&admin_token),
        ))
        .await
        .expect("send inactive soft-delete request");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let inactive_state = snapshot(&pool, inactive_target_id).await;

    assert_eq!(inactive_state.status, "inactive");
    assert!(inactive_state.deleted_at.is_some());
    assert_eq!(inactive_state.deleted_by, Some(admin_id));

    cleanup_accounts(
        &pool,
        &[inactive_target_id, target_id, admin_id],
    )
    .await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_unauthenticated_unauthorized_invalid_and_conflicting_requests() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;
    let (viewer_id, viewer_token) = seed_principal(&pool, "account_viewer").await;

    let target_id = seed_account(
        &pool,
        &format!("target-errors-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        None,
    )
    .await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{target_id}"),
            None,
        ))
        .await
        .expect("send missing-authentication request");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{target_id}"),
            Some("invalid-soft-delete-token"),
        ))
        .await
        .expect("send invalid-authentication request");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{target_id}"),
            Some(&viewer_token),
        ))
        .await
        .expect("send unauthorized-principal request");

    let body = assert_problem(response, StatusCode::FORBIDDEN).await;
    assert_eq!(body["code"], "FORBIDDEN");

    let target_before = snapshot(&pool, target_id).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            "/admin/accounts/not-a-uuid",
            Some(&admin_token),
        ))
        .await
        .expect("send invalid-account-id request");

    let body = assert_problem(response, StatusCode::BAD_REQUEST).await;
    assert_eq!(body["code"], "INVALID_ACCOUNT_ID");

    let missing_id = Uuid::new_v4();

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{missing_id}"),
            Some(&admin_token),
        ))
        .await
        .expect("send missing-account request");

    let body = assert_problem(response, StatusCode::NOT_FOUND).await;
    assert_eq!(body["code"], "ACCOUNT_NOT_FOUND");

    let deleted_id = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(OffsetDateTime::now_utc()),
        Some(admin_id),
        Some(admin_id),
    )
    .await;

    let deleted_before = snapshot(&pool, deleted_id).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{deleted_id}"),
            Some(&admin_token),
        ))
        .await
        .expect("send already-deleted request");

    let body = assert_problem(response, StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "ACCOUNT_ALREADY_DELETED");

    let deleted_after = snapshot(&pool, deleted_id).await;

    assert_eq!(deleted_after.status, deleted_before.status);
    assert_eq!(deleted_after.deleted_at, deleted_before.deleted_at);
    assert_eq!(deleted_after.deleted_by, deleted_before.deleted_by);
    assert_eq!(deleted_after.updated_at, deleted_before.updated_at);
    assert_eq!(deleted_after.updated_by, deleted_before.updated_by);

    let last_admin_before = snapshot(&pool, admin_id).await;

    let response = app(pool.clone())
        .oneshot(delete_request(
            &format!("/admin/accounts/{admin_id}"),
            Some(&admin_token),
        ))
        .await
        .expect("send last-administrator request");

    let body = assert_problem(response, StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "LAST_ACTIVE_ADMINISTRATOR");

    let last_admin_after = snapshot(&pool, admin_id).await;

    assert_eq!(last_admin_after.status, last_admin_before.status);
    assert_eq!(last_admin_after.deleted_at, last_admin_before.deleted_at);
    assert_eq!(last_admin_after.deleted_by, last_admin_before.deleted_by);
    assert_eq!(last_admin_after.updated_at, last_admin_before.updated_at);
    assert_eq!(last_admin_after.updated_by, last_admin_before.updated_by);

    let target_after = snapshot(&pool, target_id).await;

    assert_eq!(target_after.status, target_before.status);
    assert_eq!(target_after.deleted_at, target_before.deleted_at);

    cleanup_accounts(
        &pool,
        &[deleted_id, target_id, admin_id, viewer_id],
    )
    .await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn concurrent_soft_deletes_preserve_last_administrator_invariant() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_a_id, admin_a_token) = seed_principal(&pool, "account_admin").await;
    let (admin_b_id, admin_b_token) = seed_principal(&pool, "account_admin").await;

    let base_app = app(pool.clone());
    let mut requests = JoinSet::new();

    for (account_id, token) in [(admin_a_id, admin_a_token), (admin_b_id, admin_b_token)] {
        let request_app = base_app.clone();

        requests.spawn(async move {
            let response = request_app
                .oneshot(delete_request(
                    &format!("/admin/accounts/{account_id}"),
                    Some(&token),
                ))
                .await
                .expect("send concurrent soft-delete request");

            let status = response.status();
            let body = to_bytes(response.into_body(), 32 * 1024)
                .await
                .expect("read concurrent response body");

            (status, body)
        });
    }

    let mut statuses = Vec::new();
    let mut conflict_code = None;

    while let Some(result) = requests.join_next().await {
        let (status, body) = result.expect("join concurrent soft-delete request");
        statuses.push(status);

        if status == StatusCode::CONFLICT {
            let payload: Value =
                serde_json::from_slice(&body).expect("decode concurrent problem response");
            conflict_code = payload["code"].as_str().map(str::to_owned);
        }
    }

    statuses.sort_unstable();

    assert_eq!(
        statuses,
        vec![StatusCode::NO_CONTENT, StatusCode::CONFLICT],
        "exactly one concurrent soft-delete must succeed"
    );
    assert_eq!(
        conflict_code.as_deref(),
        Some("LAST_ACTIVE_ADMINISTRATOR")
    );

    let states = sqlx::query_as::<_, (Uuid, String, Option<OffsetDateTime>)>(
        r#"
        SELECT id, status, deleted_at
        FROM account
        WHERE id = ANY($1)
        ORDER BY id
        "#,
    )
    .bind([admin_a_id, admin_b_id])
    .fetch_all(&pool)
    .await
    .expect("read final administrator states");

    assert_eq!(states.len(), 2);
    assert_eq!(
        states
            .iter()
            .filter(|(_, status, deleted_at)| status == "active" && deleted_at.is_none())
            .count(),
        1
    );
    assert_eq!(
        states
            .iter()
            .filter(|(_, status, deleted_at)| status == "inactive" && deleted_at.is_some())
            .count(),
        1
    );

    cleanup_accounts(&pool, &[admin_a_id, admin_b_id]).await;
}