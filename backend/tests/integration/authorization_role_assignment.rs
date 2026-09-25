use std::env;

use sqlx::PgPool;
use time::OffsetDateTime;
use tokio::sync::Mutex;
use uuid::Uuid;

use digital_publication_backend::domains::{
    authentication::model::AuthenticatedPrincipal,
    authorization::{
        repository::AuthorizationRepository,
        service::{assign_role, RoleManagementError},
    },
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

async fn run_migrations(pool: &PgPool) {
    digital_publication_backend::MIGRATOR
        .run(pool)
        .await
        .expect("run migrations");
}

async fn seed_account(pool: &PgPool, status: &str, deleted_at: Option<OffsetDateTime>) -> Uuid {
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
            concat('authorization-assignment-', id::text, '@example.com'),
            concat('authorization-assignment-', id::text, '@example.com'),
            '$argon2id$v=19$m=19456,t=2,p=1$test$test'
        FROM inserted_account
        RETURNING account_id
        "#,
    )
    .bind(status)
    .bind(deleted_at)
    .fetch_one(pool)
    .await
    .expect("seed account")
}

async fn role_id(pool: &PgPool, role_name: &str) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM authorization_role
        WHERE name = $1
        "#,
    )
    .bind(role_name)
    .fetch_one(pool)
    .await
    .expect("find role")
}

async fn assign_role_direct(
    pool: &PgPool,
    account_id: Uuid,
    role_name: &str,
    created_by: Option<Uuid>,
) {
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
    .bind(role_id(pool, role_name).await)
    .bind(created_by)
    .execute(pool)
    .await
    .expect("seed role assignment");
}

fn principal(account_id: Uuid) -> AuthenticatedPrincipal {
    AuthenticatedPrincipal {
        account_id,
        session_id: Uuid::new_v4(),
        authenticated_at: OffsetDateTime::now_utc(),
    }
}

async fn cleanup_accounts(pool: &PgPool, account_ids: &[Uuid]) {
    if account_ids.is_empty() {
        return;
    }

    sqlx::query(
        r#"
        DELETE FROM authorization_role_revocation_audit
        WHERE account_id = ANY($1)
           OR revoked_by = ANY($1)
        "#,
    )
    .bind(account_ids)
    .execute(pool)
    .await
    .expect("cleanup revocation audit");

    sqlx::query("DELETE FROM authorization_account_role WHERE account_id = ANY($1)")
        .bind(account_ids)
        .execute(pool)
        .await
        .expect("cleanup role assignments");

    sqlx::query("DELETE FROM authentication_session WHERE account_id = ANY($1)")
        .bind(account_ids)
        .execute(pool)
        .await
        .expect("cleanup sessions");

    sqlx::query("DELETE FROM account WHERE id = ANY($1)")
        .bind(account_ids)
        .execute(pool)
        .await
        .expect("cleanup accounts");
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn assigns_role_and_records_authenticated_actor() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await
    .expect("assign role");

    let row = sqlx::query_as::<_, (Uuid, Uuid, OffsetDateTime)>(
        r#"
        SELECT
            ar.account_id,
            ar.created_by,
            ar.created_at
        FROM authorization_account_role AS ar
        INNER JOIN authorization_role AS r
            ON r.id = ar.role_id
        WHERE ar.account_id = $1
          AND r.name = 'account_viewer'
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("fetch assigned role");

    assert_eq!(row.0, target_id);
    assert_eq!(row.1, actor_id);
    assert!(row.2 <= OffsetDateTime::now_utc());

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_assignment_without_role_assign_permission() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_viewer", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::Forbidden));

    let assignment_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM authorization_account_role AS ar
        INNER JOIN authorization_role AS r
            ON r.id = ar.role_id
        WHERE ar.account_id = $1
          AND r.name = 'account_viewer'
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("count assignments");

    assert_eq!(assignment_count, 0);

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_missing_role() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        "role-that-does-not-exist",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::RoleNotFound));

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_disabled_role() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let role_name = format!("disabled-test-role-{}", Uuid::new_v4());

    let role_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO authorization_role (
            name,
            description,
            disabled_at
        )
        VALUES ($1, 'test role', CURRENT_TIMESTAMP)
        RETURNING id
        "#,
    )
    .bind(&role_name)
    .fetch_one(&pool)
    .await
    .expect("create disabled role");

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        &role_name,
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::RoleDisabled));

    sqlx::query("DELETE FROM authorization_role WHERE id = $1")
        .bind(role_id)
        .execute(&pool)
        .await
        .expect("cleanup disabled role");

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_missing_account() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let missing_account_id = Uuid::new_v4();

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        missing_account_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::AccountNotFound));

    cleanup_accounts(&pool, &[actor_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_duplicate_role_assignment() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;
    assign_role_direct(&pool, target_id, "account_viewer", Some(actor_id)).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::RoleAlreadyAssigned));

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn disabled_account_admin_role_does_not_authorize_role_assignment() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let admin_role_id = role_id(&pool, "account_admin").await;

    sqlx::query(
        r#"
        UPDATE authorization_role
        SET disabled_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(admin_role_id)
    .execute(&pool)
    .await
    .expect("disable admin role");

    let repository = AuthorizationRepository::new(pool.clone());

    let result = assign_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::Forbidden));

    sqlx::query(
        r#"
        UPDATE authorization_role
        SET disabled_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(admin_role_id)
    .execute(&pool)
    .await
    .expect("restore admin role");

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}