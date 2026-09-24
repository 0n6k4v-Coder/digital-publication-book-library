use sqlx::PgPool;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::{
    RoleAssignmentState, RoleAssignmentValidationError, RoleRevocationState,
    RoleRevocationValidationError,
};

const ACCOUNT_ADMIN_ROLE: &str = "account_admin";
const ROLE_ASSIGNMENT_PRIMARY_KEY_CONSTRAINT: &str = "authorization_account_role_pkey";

pub struct AuthorizationRepository {
    pool: PgPool,
}

impl AuthorizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn has_permission(
        &self,
        account_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM authorization_account_role AS ar
                INNER JOIN authorization_role AS r
                    ON r.id = ar.role_id
                INNER JOIN authorization_role_permission AS rp
                    ON rp.role_id = ar.role_id
                INNER JOIN authorization_permission AS p
                    ON p.id = rp.permission_id
                WHERE ar.account_id = $1
                  AND p.name = $2
                  AND r.disabled_at IS NULL
            )
            "#,
        )
        .bind(account_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn assign_role(
        &self,
        account_id: Uuid,
        role_name: &str,
        actor_id: Uuid,
    ) -> Result<(), RoleAssignmentRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RoleAssignmentRepositoryError::Database)?;

        let Some(role) = sqlx::query_as::<_, AuthorizationRoleRow>(
            r#"
            SELECT
                id,
                name,
                disabled_at
            FROM authorization_role
            WHERE name = $1
            FOR SHARE
            "#,
        )
        .bind(role_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RoleAssignmentRepositoryError::Database)?
        else {
            return Err(RoleAssignmentRepositoryError::Validation(
                RoleAssignmentValidationError::RoleNotFound,
            ));
        };

        if role.name == ACCOUNT_ADMIN_ROLE {
            lock_administrator_invariant(&mut tx)
                .await
                .map_err(RoleAssignmentRepositoryError::Database)?;
        }

        let account_exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM account
                WHERE id = $1
            )
            "#,
        )
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(RoleAssignmentRepositoryError::Database)?;

        let assignment_exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM authorization_account_role
                WHERE account_id = $1
                  AND role_id = $2
            )
            "#,
        )
        .bind(account_id)
        .bind(role.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(RoleAssignmentRepositoryError::Database)?;

        RoleAssignmentState {
            role_exists: true,
            role_enabled: role.disabled_at.is_none(),
            account_exists,
            assignment_exists,
        }
        .validate()
        .map_err(RoleAssignmentRepositoryError::Validation)?;

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
        .bind(role.id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if let Some(database_error) = error.as_database_error() {
                if database_error.constraint() == Some(ROLE_ASSIGNMENT_PRIMARY_KEY_CONSTRAINT) {
                    return RoleAssignmentRepositoryError::Validation(
                        RoleAssignmentValidationError::RoleAlreadyAssigned,
                    );
                }
            }

            RoleAssignmentRepositoryError::Database(error)
        })?;

        tx.commit()
            .await
            .map_err(RoleAssignmentRepositoryError::Database)?;

        Ok(())
    }

    pub async fn revoke_role(
        &self,
        account_id: Uuid,
        role_name: &str,
        actor_id: Uuid,
    ) -> Result<(), RoleRevocationRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RoleRevocationRepositoryError::Database)?;

        let Some(initial_assignment) = sqlx::query_as::<_, RoleAssignmentRow>(
            r#"
            SELECT
                ar.role_id,
                r.name
            FROM authorization_account_role AS ar
            INNER JOIN authorization_role AS r
                ON r.id = ar.role_id
            WHERE ar.account_id = $1
              AND r.name = $2
            "#,
        )
        .bind(account_id)
        .bind(role_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RoleRevocationRepositoryError::Database)?
        else {
            return Err(RoleRevocationRepositoryError::Validation(
                RoleRevocationValidationError::RoleAssignmentNotFound,
            ));
        };

        let is_account_admin = initial_assignment.name == ACCOUNT_ADMIN_ROLE;

        if is_account_admin {
            lock_administrator_invariant(&mut tx)
                .await
                .map_err(RoleRevocationRepositoryError::Database)?;
        }

        let Some(assignment) = sqlx::query_as::<_, RoleAssignmentRow>(
            r#"
            SELECT
                ar.role_id,
                r.name
            FROM authorization_account_role AS ar
            INNER JOIN authorization_role AS r
                ON r.id = ar.role_id
            WHERE ar.account_id = $1
              AND r.name = $2
            FOR UPDATE OF ar
            "#,
        )
        .bind(account_id)
        .bind(role_name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RoleRevocationRepositoryError::Database)?
        else {
            return Err(RoleRevocationRepositoryError::Validation(
                RoleRevocationValidationError::RoleAssignmentNotFound,
            ));
        };

        let mut target_is_active = false;
        let mut target_is_deleted = false;
        let mut active_administrator_count = 0_i64;

        if is_account_admin {
            let Some(target) = sqlx::query_as::<_, AccountLifecycleRow>(
                r#"
                SELECT
                    status,
                    deleted_at
                FROM account
                WHERE id = $1
                FOR UPDATE
                "#,
            )
            .bind(account_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(RoleRevocationRepositoryError::Database)?
            else {
                return Err(RoleRevocationRepositoryError::Validation(
                    RoleRevocationValidationError::RoleAssignmentNotFound,
                ));
            };

            target_is_active = target.status == "active";
            target_is_deleted = target.deleted_at.is_some();

            active_administrator_count = sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(DISTINCT ar.account_id)
                FROM authorization_account_role AS ar
                INNER JOIN authorization_role AS r
                    ON r.id = ar.role_id
                INNER JOIN account AS a
                    ON a.id = ar.account_id
                WHERE r.name = $1
                  AND r.disabled_at IS NULL
                  AND a.status = 'active'
                  AND a.deleted_at IS NULL
                "#,
            )
            .bind(ACCOUNT_ADMIN_ROLE)
            .fetch_one(&mut *tx)
            .await
            .map_err(RoleRevocationRepositoryError::Database)?;
        }

        RoleRevocationState {
            assignment_exists: true,
            is_account_admin: assignment.name == ACCOUNT_ADMIN_ROLE,
            target_is_active,
            target_is_deleted,
            active_administrator_count,
        }
        .validate()
        .map_err(RoleRevocationRepositoryError::Validation)?;

        sqlx::query(
            r#"
            INSERT INTO authorization_role_revocation_audit (
                account_id,
                role_id,
                revoked_by
            )
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(account_id)
        .bind(assignment.role_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(RoleRevocationRepositoryError::Database)?;

        let deleted = sqlx::query(
            r#"
            DELETE FROM authorization_account_role
            WHERE account_id = $1
              AND role_id = $2
            "#,
        )
        .bind(account_id)
        .bind(assignment.role_id)
        .execute(&mut *tx)
        .await
        .map_err(RoleRevocationRepositoryError::Database)?;

        if deleted.rows_affected() != 1 {
            return Err(RoleRevocationRepositoryError::Validation(
                RoleRevocationValidationError::RoleAssignmentNotFound,
            ));
        }

        tx.commit()
            .await
            .map_err(RoleRevocationRepositoryError::Database)?;

        Ok(())
    }
}

async fn lock_administrator_invariant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i16>(
        r#"
        SELECT id
        FROM account_administrator_invariant_lock
        WHERE id = 1
        FOR UPDATE
        "#,
    )
    .fetch_one(&mut **tx)
    .await
    .map(|_| ())
}

#[derive(Debug, sqlx::FromRow)]
struct AuthorizationRoleRow {
    id: Uuid,
    name: String,
    disabled_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct RoleAssignmentRow {
    role_id: Uuid,
    name: String,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountLifecycleRow {
    status: String,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Error)]
pub enum RoleAssignmentRepositoryError {
    #[error("{0}")]
    Validation(#[from] RoleAssignmentValidationError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Error)]
pub enum RoleRevocationRepositoryError {
    #[error("{0}")]
    Validation(#[from] RoleRevocationValidationError),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}