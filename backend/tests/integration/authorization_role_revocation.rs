use std::{env, sync::Arc};

use sqlx::{AssertSqlSafe, PgPool};
use time::OffsetDateTime;
use tokio::{sync::Mutex, task::JoinSet};
use uuid::Uuid;

use digital_publication_backend::domains::{
    authentication::model::AuthenticatedPrincipal,
    authorization::{
        repository::AuthorizationRepository,
        service::{revoke_role, RoleManagementError},
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
    sqlx::migrate!()
        .run(pool)
        .await
        .expect("run migrations");
}

async fn seed_account(pool: &PgPool, status: &str, deleted_at: Option<OffsetDateTime>) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO account (
            status,
            deleted_at
        )
        VALUES ($1, $2)
        RETURNING id
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

async fn assignment_exists(pool: &PgPool, account_id: Uuid, role_name: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM authorization_account_role AS ar
            INNER JOIN authorization_role AS r
                ON r.id = ar.role_id
            WHERE ar.account_id = $1
              AND r.name = $2
        )
        "#,
    )
    .bind(account_id)
    .bind(role_name)
    .fetch_one(pool)
    .await
    .expect("check assignment")
}

async fn audit_count(pool: &PgPool, account_id: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM authorization_role_revocation_audit
        WHERE account_id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .expect("count audit records")
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn revokes_role_and_writes_exactly_one_audit_record() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;
    assign_role_direct(&pool, target_id, "account_viewer", Some(actor_id)).await;

    let viewer_role_id = role_id(&pool, "account_viewer").await;

    let repository = AuthorizationRepository::new(pool.clone());

    revoke_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await
    .expect("revoke role");

    assert!(!assignment_exists(&pool, target_id, "account_viewer").await);
    assert_eq!(audit_count(&pool, target_id).await, 1);

    let audit = sqlx::query_as::<_, (Uuid, Uuid, Uuid, OffsetDateTime)>(
        r#"
        SELECT
            account_id,
            role_id,
            revoked_by,
            revoked_at
        FROM authorization_role_revocation_audit
        WHERE account_id = $1
        "#,
    )
    .bind(target_id)
    .fetch_one(&pool)
    .await
    .expect("fetch audit");

    assert_eq!(audit.0, target_id);
    assert_eq!(audit.1, viewer_role_id);
    assert_eq!(audit.2, actor_id);
    assert!(audit.3 <= OffsetDateTime::now_utc());

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_role_revocation_without_permission() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_viewer", None).await;
    assign_role_direct(&pool, target_id, "account_viewer", Some(actor_id)).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = revoke_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::Forbidden));
    assert!(assignment_exists(&pool, target_id, "account_viewer").await);
    assert_eq!(audit_count(&pool, target_id).await, 0);

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_missing_role_assignment() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = revoke_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    assert_eq!(result, Err(RoleManagementError::RoleAssignmentNotFound));
    assert_eq!(audit_count(&pool, target_id).await, 0);

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn rejects_revoking_the_last_active_administrator() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let admin_id = seed_account(&pool, "active", None).await;
    assign_role_direct(&pool, admin_id, "account_admin", None).await;

    let repository = AuthorizationRepository::new(pool.clone());

    let result = revoke_role(
        &repository,
        &principal(admin_id),
        admin_id,
        "account_admin",
    )
    .await;

    let assignment_remains = assignment_exists(&pool, admin_id, "account_admin").await;
    let audit_records = audit_count(&pool, admin_id).await;

    cleanup_accounts(&pool, &[admin_id]).await;

    assert_eq!(
        result,
        Err(RoleManagementError::LastActiveAdministrator)
    );
    assert!(assignment_remains);
    assert_eq!(audit_records, 0);
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn allows_revoking_admin_role_from_inactive_or_soft_deleted_account() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let inactive_id = seed_account(&pool, "inactive", None).await;
    let deleted_id =
        seed_account(&pool, "inactive", Some(OffsetDateTime::now_utc())).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;
    assign_role_direct(&pool, inactive_id, "account_admin", Some(actor_id)).await;
    assign_role_direct(&pool, deleted_id, "account_admin", Some(actor_id)).await;

    let repository = AuthorizationRepository::new(pool.clone());

    revoke_role(
        &repository,
        &principal(actor_id),
        inactive_id,
        "account_admin",
    )
    .await
    .expect("revoke inactive administrator");

    revoke_role(
        &repository,
        &principal(actor_id),
        deleted_id,
        "account_admin",
    )
    .await
    .expect("revoke soft-deleted administrator");

    assert!(!assignment_exists(&pool, inactive_id, "account_admin").await);
    assert!(!assignment_exists(&pool, deleted_id, "account_admin").await);
    assert_eq!(audit_count(&pool, inactive_id).await, 1);
    assert_eq!(audit_count(&pool, deleted_id).await, 1);

    cleanup_accounts(&pool, &[actor_id, inactive_id, deleted_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn concurrent_admin_revocations_preserve_last_active_administrator() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let admin_a_id = seed_account(&pool, "active", None).await;
    let admin_b_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, admin_a_id, "account_admin", None).await;
    assign_role_direct(&pool, admin_b_id, "account_admin", Some(admin_a_id)).await;

    let mut requests = JoinSet::new();

    for (actor_id, target_id) in [(admin_a_id, admin_b_id), (admin_b_id, admin_a_id)] {
        let request_pool = pool.clone();

        requests.spawn(async move {
            let repository = AuthorizationRepository::new(request_pool);

            repository
                .revoke_role(target_id, "account_admin", actor_id)
                .await
                .map_err(|error| match error {
                    digital_publication_backend::domains::authorization::repository::RoleRevocationRepositoryError::Validation(error) => {
                        match error {
                            digital_publication_backend::domains::authorization::model::RoleRevocationValidationError::RoleAssignmentNotFound => {
                                RoleManagementError::RoleAssignmentNotFound
                            }
                            digital_publication_backend::domains::authorization::model::RoleRevocationValidationError::LastActiveAdministrator => {
                                RoleManagementError::LastActiveAdministrator
                            }
                        }
                    }
                    digital_publication_backend::domains::authorization::repository::RoleRevocationRepositoryError::Database(_) => {
                        RoleManagementError::Internal
                    }
                })
        });
    }

    let mut results = Vec::new();

    while let Some(result) = requests.join_next().await {
        results.push(result.expect("join revocation task"));
    }

    results.sort_by_key(|result| match result {
        Ok(()) => 0_u8,
        Err(RoleManagementError::LastActiveAdministrator) => 1_u8,
        Err(_) => 2_u8,
    });

    let remaining_active_administrators = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(DISTINCT ar.account_id)
        FROM authorization_account_role AS ar
        INNER JOIN authorization_role AS r
            ON r.id = ar.role_id
        INNER JOIN account AS a
            ON a.id = ar.account_id
        WHERE r.name = 'account_admin'
          AND r.disabled_at IS NULL
          AND a.status = 'active'
          AND a.deleted_at IS NULL
          AND a.id = ANY($1)
        "#,
    )
    .bind([admin_a_id, admin_b_id])
    .fetch_one(&pool)
    .await
    .expect("count remaining administrators");

    let total_audit_records =
        audit_count(&pool, admin_a_id).await + audit_count(&pool, admin_b_id).await;

    cleanup_accounts(&pool, &[admin_a_id, admin_b_id]).await;

    assert_eq!(
        results,
        vec![
            Ok(()),
            Err(RoleManagementError::LastActiveAdministrator)
        ]
    );
    assert_eq!(remaining_active_administrators, 1);
    assert_eq!(total_audit_records, 1);
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn audit_failure_rolls_back_role_revocation() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;
    assign_role_direct(&pool, target_id, "account_viewer", Some(actor_id)).await;

    let suffix = Uuid::new_v4().simple().to_string();
    let function_name = format!("test_fail_role_revocation_audit_{suffix}");
    let trigger_name = format!("test_fail_role_revocation_audit_trigger_{suffix}");

    sqlx::query(AssertSqlSafe(format!(
        r#"
        CREATE FUNCTION {function_name}()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $$
        BEGIN
            RAISE EXCEPTION 'test audit failure';
        END;
        $$
        "#
    )))
    .execute(&pool)
    .await
    .expect("create audit failure function");

    sqlx::query(AssertSqlSafe(format!(
        r#"
        CREATE TRIGGER {trigger_name}
        BEFORE INSERT ON authorization_role_revocation_audit
        FOR EACH ROW
        EXECUTE FUNCTION {function_name}()
        "#
    )))
    .execute(&pool)
    .await
    .expect("create audit failure trigger");

    let repository = AuthorizationRepository::new(pool.clone());

    let result = revoke_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    sqlx::query(AssertSqlSafe(format!(
        "DROP TRIGGER {trigger_name} ON authorization_role_revocation_audit"
    )))
    .execute(&pool)
    .await
    .expect("drop audit failure trigger");

    sqlx::query(AssertSqlSafe(format!(
        "DROP FUNCTION {function_name}()"
    )))
    .execute(&pool)
    .await
    .expect("drop audit failure function");

    assert_eq!(result, Err(RoleManagementError::Internal));
    assert!(assignment_exists(&pool, target_id, "account_viewer").await);
    assert_eq!(audit_count(&pool, target_id).await, 0);

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn role_assignment_delete_failure_rolls_back_audit_insert() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;

    let Some(pool) = test_pool().await else {
        return;
    };

    run_migrations(&pool).await;

    let actor_id = seed_account(&pool, "active", None).await;
    let target_id = seed_account(&pool, "active", None).await;

    assign_role_direct(&pool, actor_id, "account_admin", None).await;
    assign_role_direct(&pool, target_id, "account_viewer", Some(actor_id)).await;

    let suffix = Arc::new(Uuid::new_v4().simple().to_string());
    let function_name = format!("test_fail_role_assignment_delete_{}", suffix.as_str());
    let trigger_name = format!(
        "test_fail_role_assignment_delete_trigger_{}",
        suffix.as_str()
    );

    sqlx::query(AssertSqlSafe(format!(
        r#"
        CREATE FUNCTION {function_name}()
        RETURNS trigger
        LANGUAGE plpgsql
        AS $$
        BEGIN
            RAISE EXCEPTION 'test role assignment delete failure';
        END;
        $$
        "#
    )))
    .execute(&pool)
    .await
    .expect("create assignment delete failure function");

    sqlx::query(AssertSqlSafe(format!(
        r#"
        CREATE TRIGGER {trigger_name}
        BEFORE DELETE ON authorization_account_role
        FOR EACH ROW
        EXECUTE FUNCTION {function_name}()
        "#
    )))
    .execute(&pool)
    .await
    .expect("create assignment delete failure trigger");

    let repository = AuthorizationRepository::new(pool.clone());

    let result = revoke_role(
        &repository,
        &principal(actor_id),
        target_id,
        "account_viewer",
    )
    .await;

    sqlx::query(AssertSqlSafe(format!(
        "DROP TRIGGER {trigger_name} ON authorization_account_role"
    )))
    .execute(&pool)
    .await
    .expect("drop assignment delete failure trigger");

    sqlx::query(AssertSqlSafe(format!(
        "DROP FUNCTION {function_name}()"
    )))
    .execute(&pool)
    .await
    .expect("drop assignment delete failure function");

    assert_eq!(result, Err(RoleManagementError::Internal));
    assert!(assignment_exists(&pool, target_id, "account_viewer").await);
    assert_eq!(audit_count(&pool, target_id).await, 0);

    cleanup_accounts(&pool, &[actor_id, target_id]).await;
}