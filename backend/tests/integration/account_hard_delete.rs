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
use tokio::{
    sync::Mutex,
    task::JoinSet,
};
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
            .expect("connect to TEST_DATABASE_URL"),
    )
}

async fn seed_account(
    pool: &PgPool,
    _email: &str,
    status: &str,
    deleted_at: Option<OffsetDateTime>,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (
            status,
            deleted_at,
            created_by,
            updated_by
        )
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .bind(created_by)
    .bind(updated_by)
    .fetch_one(pool)
    .await
    .expect("seed account")
}

async fn seed_credentials(pool: &PgPool, account_id: Uuid, email: &str) {
    sqlx::query(
        r#"
        INSERT INTO account_credentials (
            account_id,
            email,
            email_normalized,
            password_hash
        )
        VALUES ($1, $2, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(email)
    .bind("$argon2id$v=19$m=19456,t=2,p=1$test$test")
    .execute(pool)
    .await
    .expect("seed account credentials");
}

async fn assign_role_as(
    pool: &PgPool,
    account_id: Uuid,
    role_name: &str,
    created_by: Uuid,
) {
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
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("assign authorization role");
}

async fn seed_access_and_refresh_tokens(pool: &PgPool, account_id: Uuid) -> String {
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

    let access_token = format!(
        "test-purge-access-{}-{}",
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
    .bind(sha256_token_verifier(&access_token))
    .execute(pool)
    .await
    .expect("seed access token");

    let refresh_token = format!(
        "test-purge-refresh-{}-{}",
        Uuid::new_v4(),
        Uuid::new_v4()
    );

    sqlx::query(
        r#"
        INSERT INTO authentication_refresh_token (
            session_id,
            token_hash,
            expires_at
        )
        VALUES ($1, $2, CURRENT_TIMESTAMP + INTERVAL '30 days')
        "#,
    )
    .bind(session_id)
    .bind(sha256_token_verifier(&refresh_token))
    .execute(pool)
    .await
    .expect("seed refresh token");

    access_token
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

    let email = format!("principal-{account_id}@example.com");
    seed_credentials(pool, account_id, &email).await;

    let token = seed_access_and_refresh_tokens(pool, account_id).await;
    assign_role_as(pool, account_id, role_name, account_id).await;

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

fn purge_request(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("DELETE").uri(uri);

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    builder.body(Body::empty()).expect("build purge request")
}

fn view_request(uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build view request")
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

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn hard_deletes_account_and_cascades_all_required_state() {
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
        Some(admin_id),
    )
    .await;
    seed_credentials(
        &pool,
        target_id,
        &format!("target-{}@example.com", target_id),
    )
    .await;
    let target_token = seed_access_and_refresh_tokens(&pool, target_id).await;
    assign_role_as(&pool, target_id, "account_admin", admin_id).await;

    let other_admin_id = seed_account(
        &pool,
        &format!("other-admin-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;
    seed_credentials(
        &pool,
        other_admin_id,
        &format!("other-admin-{}@example.com", other_admin_id),
    )
    .await;
    assign_role_as(&pool, other_admin_id, "account_admin", admin_id).await;

    let survivor_id = seed_account(
        &pool,
        &format!("survivor-{}@example.com", Uuid::new_v4()),
        "active",
        None,
        Some(target_id),
        Some(target_id),
    )
    .await;
    seed_credentials(
        &pool,
        survivor_id,
        &format!("survivor-{}@example.com", survivor_id),
    )
    .await;
    assign_role_as(&pool, survivor_id, "account_viewer", target_id).await;

    let role_definition_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authorization_role",
    )
    .fetch_one(&pool)
    .await
    .expect("count role definitions");

    let permission_definition_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authorization_permission",
    )
    .fetch_one(&pool)
    .await
    .expect("count permission definitions");

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{target_id}/purge"),
            Some(&admin_token),
        ))
        .await
        .expect("send purge request");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("read purge response");
    assert!(body.is_empty());

    let account_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account WHERE id = $1)",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("check account existence");
    assert!(!account_exists);

    let credentials_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account_credentials WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count target credentials");
    assert_eq!(credentials_count, 0);

    let session_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authentication_session WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count target sessions");
    assert_eq!(session_count, 0);

    let access_token_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM authentication_access_token AS at
        INNER JOIN authentication_session AS s
            ON s.id = at.session_id
        WHERE s.account_id = $1
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count target access tokens");
    assert_eq!(access_token_count, 0);

    let refresh_token_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM authentication_refresh_token AS rt
        INNER JOIN authentication_session AS s
            ON s.id = rt.session_id
        WHERE s.account_id = $1
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count target refresh tokens");
    assert_eq!(refresh_token_count, 0);

    let role_assignment_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authorization_account_role WHERE account_id = $1",
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count target role assignments");
    assert_eq!(role_assignment_count, 0);

    let surviving_creator = sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT created_by
        FROM authorization_account_role
        WHERE account_id = $1
          AND role_id = (
              SELECT id
              FROM authorization_role
              WHERE name = 'account_viewer'
          )
        "#,
    )
    .bind(survivor_id)
    .fetch_optional(&pool)
    .await
    .expect("fetch surviving role creator");

    assert_eq!(surviving_creator.flatten(), None);

    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authorization_role")
            .fetch_one(&pool)
            .await
            .expect("count remaining role definitions"),
        role_definition_count
    );

    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM authorization_permission")
            .fetch_one(&pool)
            .await
            .expect("count remaining permission definitions"),
        permission_definition_count
    );

    let response = app(pool.clone())
        .oneshot(view_request(
            &format!("/admin/accounts/{survivor_id}"),
            &target_token,
        ))
        .await
        .expect("send request with deleted account token");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    cleanup_accounts(
        &pool,
        &[target_id, survivor_id, other_admin_id, admin_id],
    )
    .await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_authentication_authorization_and_invalid_targets() {
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

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{admin_id}/purge"),
            None,
        ))
        .await
        .expect("send unauthenticated request");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{admin_id}/purge"),
            Some("invalid-purge-token"),
        ))
        .await
        .expect("send invalid-token request");

    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer realm="admin-api", error="invalid_token""#
    );
    assert_problem(response, StatusCode::UNAUTHORIZED).await;

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{admin_id}/purge"),
            Some(&viewer_token),
        ))
        .await
        .expect("send unauthorized request");

    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
    let body = assert_problem(response, StatusCode::FORBIDDEN).await;
    assert_eq!(body["code"], "FORBIDDEN");

    let response = app(pool.clone())
        .oneshot(purge_request(
            "/admin/accounts/not-a-uuid/purge",
            Some(&admin_token),
        ))
        .await
        .expect("send invalid-account-id request");

    let body = assert_problem(response, StatusCode::BAD_REQUEST).await;
    assert_eq!(body["code"], "INVALID_ACCOUNT_ID");

    let missing_id = Uuid::new_v4();

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{missing_id}/purge"),
            Some(&admin_token),
        ))
        .await
        .expect("send missing-account request");

    let body = assert_problem(response, StatusCode::NOT_FOUND).await;
    assert_eq!(body["code"], "ACCOUNT_NOT_FOUND");

    let exists_after_errors = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account WHERE id = $1)",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .expect("check administrator still exists");

    assert!(exists_after_errors);

    cleanup_accounts(&pool, &[admin_id, viewer_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_hard_delete_of_the_last_active_administrator() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{admin_id}/purge"),
            Some(&admin_token),
        ))
        .await
        .expect("send last-administrator purge request");

    let body = assert_problem(response, StatusCode::CONFLICT).await;
    assert_eq!(body["code"], "LAST_ACTIVE_ADMINISTRATOR");

    let account_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account WHERE id = $1)",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .expect("check administrator existence");
    assert!(account_exists);

    let role_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM authorization_account_role WHERE account_id = $1",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .expect("count administrator role assignments");
    assert_eq!(role_count, 1);

    let response = app(pool.clone())
        .oneshot(view_request(
            &format!("/admin/accounts/{admin_id}"),
            &admin_token,
        ))
        .await
        .expect("verify authentication state survived rollback");

    assert_eq!(response.status(), StatusCode::OK);

    cleanup_accounts(&pool, &[admin_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn concurrent_purges_preserve_the_last_administrator_invariant() {
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

    let mut requests = JoinSet::new();

    for (account_id, token) in [
        (admin_a_id, admin_a_token),
        (admin_b_id, admin_b_token),
    ] {
        let request_app = app(pool.clone());

        requests.spawn(async move {
            let response = request_app
                .oneshot(purge_request(
                    &format!("/admin/accounts/{account_id}/purge"),
                    Some(&token),
                ))
                .await
                .expect("send concurrent purge request");

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
        let (status, body) = result.expect("join concurrent purge request");
        statuses.push(status);

        if status == StatusCode::CONFLICT {
            let payload: Value =
                serde_json::from_slice(&body).expect("decode conflict response");
            conflict_code = payload["code"].as_str().map(str::to_owned);
        }
    }

    statuses.sort_unstable();

    assert_eq!(
        statuses,
        vec![StatusCode::NO_CONTENT, StatusCode::CONFLICT],
        "exactly one concurrent purge must succeed"
    );
    assert_eq!(
        conflict_code.as_deref(),
        Some("LAST_ACTIVE_ADMINISTRATOR")
    );

    let remaining_active_admins = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM account AS a
        INNER JOIN authorization_account_role AS ar
            ON ar.account_id = a.id
        INNER JOIN authorization_role AS r
            ON r.id = ar.role_id
        WHERE a.id = ANY($1)
          AND a.status = 'active'
          AND a.deleted_at IS NULL
          AND r.name = 'account_admin'
          AND r.disabled_at IS NULL
        "#,
    )
    .bind([admin_a_id, admin_b_id])
    .fetch_one(&pool)
    .await
    .expect("count remaining active administrators");

    assert_eq!(remaining_active_admins, 1);

    cleanup_accounts(&pool, &[admin_a_id, admin_b_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn hard_delete_allows_inactive_and_soft_deleted_targets() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("run migrations");

    let (admin_id, admin_token) = seed_principal(&pool, "account_admin").await;

    let inactive_id = seed_account(
        &pool,
        &format!("inactive-{}@example.com", Uuid::new_v4()),
        "inactive",
        None,
        Some(admin_id),
        Some(admin_id),
    )
    .await;
    seed_credentials(
        &pool,
        inactive_id,
        &format!("inactive-{}@example.com", inactive_id),
    )
    .await;
    assign_role_as(&pool, inactive_id, "account_admin", admin_id).await;

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{inactive_id}/purge"),
            Some(&admin_token),
        ))
        .await
        .expect("purge inactive account");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let soft_deleted_id = seed_account(
        &pool,
        &format!("deleted-{}@example.com", Uuid::new_v4()),
        "inactive",
        Some(OffsetDateTime::now_utc()),
        Some(admin_id),
        Some(admin_id),
    )
    .await;
    seed_credentials(
        &pool,
        soft_deleted_id,
        &format!("deleted-{}@example.com", soft_deleted_id),
    )
    .await;
    assign_role_as(&pool, soft_deleted_id, "account_admin", admin_id).await;

    let response = app(pool.clone())
        .oneshot(purge_request(
            &format!("/admin/accounts/{soft_deleted_id}/purge"),
            Some(&admin_token),
        ))
        .await
        .expect("purge soft-deleted account");
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    cleanup_accounts(
        &pool,
        &[inactive_id, soft_deleted_id, admin_id],
    )
    .await;
}