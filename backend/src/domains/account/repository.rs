use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::{
    ActivationState, ActivationValidationError, CreatedAccount, DeactivationState,
    DeactivationValidationError, HardDeleteState, HardDeleteValidationError, ListedAccount,
    ListedAccounts, RestoreState, RestoreValidationError, SoftDeleteState,
    SoftDeleteValidationError, ViewedAccount,
};

const EMAIL_UNIQUE_CONSTRAINT: &str = "account_credentials_email_normalized_key";
const ACCOUNT_ADMIN_ROLE: &str = "account_admin";

pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        actor_id: Uuid,
        email: &str,
        email_normalized: &str,
        password_hash: &str,
    ) -> Result<CreatedAccount, CreateAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(CreateAccountRepositoryError::Database)?;

        let account = sqlx::query_as::<_, AccountRow>(
            r#"
            INSERT INTO account (created_by, updated_by, status, deleted_at)
            VALUES ($1, $1, 'active', NULL)
            RETURNING id, created_at, updated_at, status, deleted_at
            "#,
        )
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(CreateAccountRepositoryError::Database)?;

        sqlx::query(
            r#"
            INSERT INTO account_credentials (
                account_id,
                email,
                email_normalized,
                password_hash
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(account.id)
        .bind(email)
        .bind(email_normalized)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if let Some(database_error) = error.as_database_error() {
                if database_error.constraint() == Some(EMAIL_UNIQUE_CONSTRAINT) {
                    return CreateAccountRepositoryError::EmailAlreadyInUse;
                }
            }

            CreateAccountRepositoryError::Database(error)
        })?;

        tx.commit()
            .await
            .map_err(CreateAccountRepositoryError::Database)?;

        Ok(CreatedAccount {
            id: account.id,
            email: email.to_owned(),
            status: account.status,
            created_at: account.created_at,
            updated_at: account.updated_at,
            deleted_at: account.deleted_at,
        })
    }

    pub async fn change_email(
        &self,
        account_id: Uuid,
        email: &str,
        email_normalized: &str,
    ) -> Result<ViewedAccount, ChangeEmailRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(ChangeEmailRepositoryError::Database)?;

        let current = sqlx::query_as::<_, AccountCredentialTargetRow>(
            r#"
            SELECT
                a.id,
                a.status,
                a.created_at,
                a.updated_at,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
            FOR UPDATE OF a, ac
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ChangeEmailRepositoryError::Database)?;

        let Some(current) = current else {
            return Err(ChangeEmailRepositoryError::AccountNotFound);
        };

        if current.deleted_at.is_some() {
            return Err(ChangeEmailRepositoryError::AccountNotFound);
        }

        sqlx::query(
            r#"
            UPDATE account_credentials
            SET
                email = $2,
                email_normalized = $3,
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = $1
            "#,
        )
        .bind(account_id)
        .bind(email)
        .bind(email_normalized)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if let Some(database_error) = error.as_database_error() {
                if database_error.constraint() == Some(EMAIL_UNIQUE_CONSTRAINT) {
                    return ChangeEmailRepositoryError::EmailAlreadyInUse;
                }
            }

            ChangeEmailRepositoryError::Database(error)
        })?;

        tx.commit()
            .await
            .map_err(ChangeEmailRepositoryError::Database)?;

        Ok(ViewedAccount {
            id: current.id,
            email: email.to_owned(),
            status: current.status,
            created_at: current.created_at,
            updated_at: current.updated_at,
            deleted_at: current.deleted_at,
        })
    }

    pub async fn change_password(
        &self,
        account_id: Uuid,
        password_hash: &str,
    ) -> Result<(), ChangePasswordRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(ChangePasswordRepositoryError::Database)?;

        let current = sqlx::query_as::<_, AccountCredentialTargetRow>(
            r#"
            SELECT
                a.id,
                a.status,
                a.created_at,
                a.updated_at,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
            FOR UPDATE OF a, ac
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ChangePasswordRepositoryError::Database)?;

        let Some(current) = current else {
            return Err(ChangePasswordRepositoryError::AccountNotFound);
        };

        if current.deleted_at.is_some() {
            return Err(ChangePasswordRepositoryError::AccountNotFound);
        }

        sqlx::query(
            r#"
            UPDATE account_credentials
            SET
                password_hash = $2,
                updated_at = CURRENT_TIMESTAMP
            WHERE account_id = $1
            "#,
        )
        .bind(account_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await
        .map_err(ChangePasswordRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(ChangePasswordRepositoryError::Database)?;

        Ok(())
    }

    pub async fn list(
        &self,
        page: u32,
        page_size: u32,
        status: Option<&str>,
        include_deleted: bool,
    ) -> Result<ListedAccounts, ListAccountsRepositoryError> {
        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE ($1::text IS NULL OR a.status = $1)
              AND ($2 OR a.deleted_at IS NULL)
            "#,
        )
        .bind(status)
        .bind(include_deleted)
        .fetch_one(&self.pool)
        .await
        .map_err(ListAccountsRepositoryError::Database)?;

        let limit = i64::from(page_size);
        let offset = (i64::from(page) - 1) * limit;

        let rows = sqlx::query_as::<_, ListedAccountRow>(
            r#"
            SELECT
                a.id,
                ac.email,
                a.status,
                a.created_at,
                a.updated_at,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE ($1::text IS NULL OR a.status = $1)
              AND ($2 OR a.deleted_at IS NULL)
            ORDER BY a.id ASC
            LIMIT $3
            OFFSET $4
            "#,
        )
        .bind(status)
        .bind(include_deleted)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(ListAccountsRepositoryError::Database)?;

        Ok(ListedAccounts {
            items: rows.into_iter().map(ListedAccount::from).collect(),
            page,
            page_size,
            total,
        })
    }

    pub async fn find_by_id(
        &self,
        account_id: Uuid,
    ) -> Result<Option<ViewedAccount>, ViewAccountRepositoryError> {
        let row = sqlx::query_as::<_, ViewedAccountRow>(
            r#"
            SELECT
                a.id,
                ac.email,
                a.status,
                a.created_at,
                a.updated_at,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
              AND a.deleted_at IS NULL
            "#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(ViewAccountRepositoryError::Database)?;

        Ok(row.map(ViewedAccount::from))
    }

    pub async fn update_display_name(
        &self,
        account_id: Uuid,
        actor_id: Uuid,
        display_name: Option<&str>,
    ) -> Result<Option<ViewedAccount>, UpdateAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(UpdateAccountRepositoryError::Database)?;

        let current = sqlx::query_as::<_, AccountUpdateTargetRow>(
            r#"
            SELECT
                a.id,
                ac.email,
                a.display_name,
                a.status,
                a.created_at,
                a.updated_at,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
              AND a.deleted_at IS NULL
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(UpdateAccountRepositoryError::Database)?;

        let Some(current) = current else {
            return Ok(None);
        };

        if current.display_name.as_deref() == display_name {
            tx.commit()
                .await
                .map_err(UpdateAccountRepositoryError::Database)?;

            return Ok(Some(current.into()));
        }

        let updated = sqlx::query_as::<_, AccountUpdatedRow>(
            r#"
            UPDATE account
            SET
                display_name = $2,
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $3
            WHERE id = $1
              AND deleted_at IS NULL
            RETURNING
                id,
                created_at,
                updated_at,
                status,
                deleted_at
            "#,
        )
        .bind(account_id)
        .bind(display_name)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(UpdateAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(UpdateAccountRepositoryError::Database)?;

        Ok(Some(ViewedAccount {
            id: updated.id,
            email: current.email,
            status: updated.status,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            deleted_at: updated.deleted_at,
        }))
    }

    pub async fn deactivate(
        &self,
        account_id: Uuid,
        actor_id: Uuid,
    ) -> Result<ViewedAccount, DeactivateAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(DeactivateAccountRepositoryError::Database)?;

        lock_administrator_invariant(&mut tx)
            .await
            .map_err(DeactivateAccountRepositoryError::Database)?;

        let rows = sqlx::query_as::<_, DeactivationAccountRow>(
            r#"
            SELECT
                a.id,
                ac.email,
                a.status,
                a.deleted_at,
                EXISTS (
                    SELECT 1
                    FROM authorization_account_role AS ar
                    INNER JOIN authorization_role AS r
                        ON r.id = ar.role_id
                    WHERE ar.account_id = a.id
                      AND r.name = $2
                      AND r.disabled_at IS NULL
                ) AS is_administrator
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
               OR (
                    a.status = 'active'
                    AND a.deleted_at IS NULL
                    AND EXISTS (
                        SELECT 1
                        FROM authorization_account_role AS ar_active
                        INNER JOIN authorization_role AS r_active
                            ON r_active.id = ar_active.role_id
                        WHERE ar_active.account_id = a.id
                          AND r_active.name = $2
                          AND r_active.disabled_at IS NULL
                    )
                )
            ORDER BY a.id ASC
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .bind(ACCOUNT_ADMIN_ROLE)
        .fetch_all(&mut *tx)
        .await
        .map_err(DeactivateAccountRepositoryError::Database)?;

        let Some(target) = rows.iter().find(|row| row.id == account_id) else {
            return Err(DeactivateAccountRepositoryError::AccountNotFound);
        };

        let active_administrator_count = rows
            .iter()
            .filter(|row| {
                row.status == "active" && row.deleted_at.is_none() && row.is_administrator
            })
            .count();

        let state = DeactivationState {
            is_active: target.status == "active",
            is_deleted: target.deleted_at.is_some(),
            is_administrator: target.is_administrator,
            active_administrator_count,
        };

        state.validate().map_err(|error| match error {
            DeactivationValidationError::AccountNotFound => {
                DeactivateAccountRepositoryError::AccountNotFound
            }
            DeactivationValidationError::AccountAlreadyInactive => {
                DeactivateAccountRepositoryError::AccountAlreadyInactive
            }
            DeactivationValidationError::LastActiveAdministrator => {
                DeactivateAccountRepositoryError::LastActiveAdministrator
            }
        })?;

        let target_email = target.email.clone();

        let updated = sqlx::query_as::<_, AccountUpdatedRow>(
            r#"
            UPDATE account
            SET
                status = 'inactive',
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $2
            WHERE id = $1
              AND status = 'active'
              AND deleted_at IS NULL
            RETURNING
                id,
                created_at,
                updated_at,
                status,
                deleted_at
            "#,
        )
        .bind(account_id)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(DeactivateAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(DeactivateAccountRepositoryError::Database)?;

        Ok(ViewedAccount {
            id: updated.id,
            email: target_email,
            status: updated.status,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            deleted_at: updated.deleted_at,
        })
    }

    pub async fn soft_delete(
        &self,
        account_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), SoftDeleteAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(SoftDeleteAccountRepositoryError::Database)?;

        lock_administrator_invariant(&mut tx)
            .await
            .map_err(SoftDeleteAccountRepositoryError::Database)?;

        let rows = sqlx::query_as::<_, SoftDeleteAccountRow>(
            r#"
            SELECT
                a.id,
                a.status,
                a.deleted_at,
                EXISTS (
                    SELECT 1
                    FROM authorization_account_role AS ar
                    INNER JOIN authorization_role AS r
                        ON r.id = ar.role_id
                    WHERE ar.account_id = a.id
                      AND r.name = $2
                      AND r.disabled_at IS NULL
                ) AS is_administrator
            FROM account AS a
            WHERE a.id = $1
               OR (
                    a.status = 'active'
                    AND a.deleted_at IS NULL
                    AND EXISTS (
                        SELECT 1
                        FROM authorization_account_role AS ar_active
                        INNER JOIN authorization_role AS r_active
                            ON r_active.id = ar_active.role_id
                        WHERE ar_active.account_id = a.id
                          AND r_active.name = $2
                          AND r_active.disabled_at IS NULL
                    )
                )
            ORDER BY a.id ASC
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .bind(ACCOUNT_ADMIN_ROLE)
        .fetch_all(&mut *tx)
        .await
        .map_err(SoftDeleteAccountRepositoryError::Database)?;

        let Some(target) = rows.iter().find(|row| row.id == account_id) else {
            return Err(SoftDeleteAccountRepositoryError::AccountNotFound);
        };

        let active_administrator_count = rows
            .iter()
            .filter(|row| {
                row.status == "active" && row.deleted_at.is_none() && row.is_administrator
            })
            .count();

        let state = SoftDeleteState {
            is_active: target.status == "active",
            is_deleted: target.deleted_at.is_some(),
            is_administrator: target.is_administrator,
            active_administrator_count,
        };

        state.validate().map_err(|error| match error {
            SoftDeleteValidationError::AccountAlreadyDeleted => {
                SoftDeleteAccountRepositoryError::AccountAlreadyDeleted
            }
            SoftDeleteValidationError::LastActiveAdministrator => {
                SoftDeleteAccountRepositoryError::LastActiveAdministrator
            }
        })?;

        sqlx::query(
            r#"
            UPDATE account
            SET
                status = 'inactive',
                deleted_at = CURRENT_TIMESTAMP,
                deleted_by = $2,
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $2
            WHERE id = $1
              AND deleted_at IS NULL
            "#,
        )
        .bind(account_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await
        .map_err(SoftDeleteAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(SoftDeleteAccountRepositoryError::Database)?;

        Ok(())
    }

    pub async fn hard_delete(
        &self,
        account_id: Uuid,
    ) -> Result<(), HardDeleteAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(HardDeleteAccountRepositoryError::Database)?;

        sqlx::query_scalar::<_, i16>(
            r#"
            SELECT id
            FROM account_administrator_invariant_lock
            WHERE id = 1
            FOR UPDATE
            "#,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(HardDeleteAccountRepositoryError::Database)?;

        let rows = sqlx::query_as::<_, HardDeleteAccountRow>(
            r#"
            SELECT
                a.id,
                a.status,
                a.deleted_at,
                EXISTS (
                    SELECT 1
                    FROM authorization_account_role AS ar
                    INNER JOIN authorization_role AS r
                        ON r.id = ar.role_id
                    WHERE ar.account_id = a.id
                      AND r.name = $2
                      AND r.disabled_at IS NULL
                ) AS is_administrator
            FROM account AS a
            WHERE a.id = $1
               OR (
                    a.status = 'active'
                    AND a.deleted_at IS NULL
                    AND EXISTS (
                        SELECT 1
                        FROM authorization_account_role AS ar_active
                        INNER JOIN authorization_role AS r_active
                            ON r_active.id = ar_active.role_id
                        WHERE ar_active.account_id = a.id
                          AND r_active.name = $2
                          AND r_active.disabled_at IS NULL
                    )
                )
            ORDER BY a.id ASC
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .bind(ACCOUNT_ADMIN_ROLE)
        .fetch_all(&mut *tx)
        .await
        .map_err(HardDeleteAccountRepositoryError::Database)?;

        let Some(target) = rows.iter().find(|row| row.id == account_id) else {
            return Err(HardDeleteAccountRepositoryError::AccountNotFound);
        };

        let active_administrator_count = rows
            .iter()
            .filter(|row| {
                row.status == "active" && row.deleted_at.is_none() && row.is_administrator
            })
            .count();

        let state = HardDeleteState {
            is_active: target.status == "active",
            is_deleted: target.deleted_at.is_some(),
            is_administrator: target.is_administrator,
            active_administrator_count,
        };

        state.validate().map_err(|error| match error {
            HardDeleteValidationError::LastActiveAdministrator => {
                HardDeleteAccountRepositoryError::LastActiveAdministrator
            }
        })?;

        sqlx::query(
            r#"
            DELETE FROM account
            WHERE id = $1
            "#,
        )
        .bind(account_id)
        .execute(&mut *tx)
        .await
        .map_err(HardDeleteAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(HardDeleteAccountRepositoryError::Database)?;

        Ok(())
    }

    pub async fn activate(
        &self,
        account_id: Uuid,
        actor_id: Uuid,
    ) -> Result<ViewedAccount, ActivateAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(ActivateAccountRepositoryError::Database)?;

        lock_administrator_invariant(&mut tx)
            .await
            .map_err(ActivateAccountRepositoryError::Database)?;

        let Some(target) = sqlx::query_as::<_, ActivationAccountRow>(
            r#"
            SELECT
                ac.email,
                a.status,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ActivateAccountRepositoryError::Database)?
        else {
            return Err(ActivateAccountRepositoryError::AccountNotFound);
        };

        ActivationState {
            is_active: target.status == "active",
            is_deleted: target.deleted_at.is_some(),
        }
        .validate()
        .map_err(|error| match error {
            ActivationValidationError::AccountSoftDeleted => {
                ActivateAccountRepositoryError::AccountSoftDeleted
            }
            ActivationValidationError::AccountAlreadyActive => {
                ActivateAccountRepositoryError::AccountAlreadyActive
            }
        })?;

        let updated = sqlx::query_as::<_, AccountUpdatedRow>(
            r#"
            UPDATE account
            SET
                status = 'active',
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $2
            WHERE id = $1
              AND status = 'inactive'
              AND deleted_at IS NULL
            RETURNING
                id,
                created_at,
                updated_at,
                status,
                deleted_at
            "#,
        )
        .bind(account_id)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(ActivateAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(ActivateAccountRepositoryError::Database)?;

        Ok(ViewedAccount {
            id: updated.id,
            email: target.email,
            status: updated.status,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            deleted_at: updated.deleted_at,
        })
    }

    pub async fn restore(
        &self,
        account_id: Uuid,
        actor_id: Uuid,
    ) -> Result<ViewedAccount, RestoreAccountRepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(RestoreAccountRepositoryError::Database)?;

        let Some(target) = sqlx::query_as::<_, ActivationAccountRow>(
            r#"
            SELECT
                ac.email,
                a.status,
                a.deleted_at
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE a.id = $1
            FOR UPDATE OF a
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(RestoreAccountRepositoryError::Database)?
        else {
            return Err(RestoreAccountRepositoryError::AccountNotFound);
        };

        RestoreState {
            is_deleted: target.deleted_at.is_some(),
        }
        .validate()
        .map_err(|error| match error {
            RestoreValidationError::AccountNotDeleted => {
                RestoreAccountRepositoryError::AccountNotDeleted
            }
        })?;

        let updated = sqlx::query_as::<_, AccountUpdatedRow>(
            r#"
            UPDATE account
            SET
                status = 'inactive',
                deleted_at = NULL,
                deleted_by = NULL,
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $2
            WHERE id = $1
              AND deleted_at IS NOT NULL
            RETURNING
                id,
                created_at,
                updated_at,
                status,
                deleted_at
            "#,
        )
        .bind(account_id)
        .bind(actor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(RestoreAccountRepositoryError::Database)?;

        tx.commit()
            .await
            .map_err(RestoreAccountRepositoryError::Database)?;

        Ok(ViewedAccount {
            id: updated.id,
            email: target.email,
            status: updated.status,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
            deleted_at: updated.deleted_at,
        })
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
struct AccountRow {
    id: Uuid,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    status: String,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct ListedAccountRow {
    id: Uuid,
    email: String,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct ViewedAccountRow {
    id: Uuid,
    email: String,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountUpdateTargetRow {
    id: Uuid,
    email: String,
    display_name: Option<String>,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountCredentialTargetRow {
    id: Uuid,
    status: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountUpdatedRow {
    id: Uuid,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    status: String,
    deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
struct DeactivationAccountRow {
    id: Uuid,
    email: String,
    status: String,
    deleted_at: Option<OffsetDateTime>,
    is_administrator: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct SoftDeleteAccountRow {
    id: Uuid,
    status: String,
    deleted_at: Option<OffsetDateTime>,
    is_administrator: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct HardDeleteAccountRow {
    id: Uuid,
    status: String,
    deleted_at: Option<OffsetDateTime>,
    is_administrator: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct ActivationAccountRow {
    email: String,
    status: String,
    deleted_at: Option<OffsetDateTime>,
}

impl From<ListedAccountRow> for ListedAccount {
    fn from(row: ListedAccountRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }
}

impl From<ViewedAccountRow> for ViewedAccount {
    fn from(row: ViewedAccountRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }
}

impl From<AccountUpdateTargetRow> for ViewedAccount {
    fn from(row: AccountUpdateTargetRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        }
    }
}

#[derive(Debug)]
pub enum CreateAccountRepositoryError {
    EmailAlreadyInUse,
    Database(sqlx::Error),
}

impl std::fmt::Display for CreateAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmailAlreadyInUse => f.write_str("email already in use"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CreateAccountRepositoryError {}

#[derive(Debug)]
pub enum ChangeEmailRepositoryError {
    AccountNotFound,
    EmailAlreadyInUse,
    Database(sqlx::Error),
}

impl std::fmt::Display for ChangeEmailRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::EmailAlreadyInUse => f.write_str("email already in use"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChangeEmailRepositoryError {}

#[derive(Debug)]
pub enum ChangePasswordRepositoryError {
    AccountNotFound,
    Database(sqlx::Error),
}

impl std::fmt::Display for ChangePasswordRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChangePasswordRepositoryError {}

#[derive(Debug)]
pub enum ListAccountsRepositoryError {
    Database(sqlx::Error),
}

impl std::fmt::Display for ListAccountsRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ListAccountsRepositoryError {}

#[derive(Debug)]
pub enum ViewAccountRepositoryError {
    Database(sqlx::Error),
}

impl std::fmt::Display for ViewAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ViewAccountRepositoryError {}

#[derive(Debug)]
pub enum UpdateAccountRepositoryError {
    Database(sqlx::Error),
}

impl std::fmt::Display for UpdateAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for UpdateAccountRepositoryError {}

#[derive(Debug)]
pub enum DeactivateAccountRepositoryError {
    AccountNotFound,
    AccountAlreadyInactive,
    LastActiveAdministrator,
    Database(sqlx::Error),
}

impl std::fmt::Display for DeactivateAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::AccountAlreadyInactive => f.write_str("account already inactive"),
            Self::LastActiveAdministrator => f.write_str("last active administrator"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DeactivateAccountRepositoryError {}

#[derive(Debug)]
pub enum SoftDeleteAccountRepositoryError {
    AccountNotFound,
    AccountAlreadyDeleted,
    LastActiveAdministrator,
    Database(sqlx::Error),
}

impl std::fmt::Display for SoftDeleteAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::AccountAlreadyDeleted => f.write_str("account already deleted"),
            Self::LastActiveAdministrator => f.write_str("last active administrator"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for SoftDeleteAccountRepositoryError {}

#[derive(Debug)]
pub enum HardDeleteAccountRepositoryError {
    AccountNotFound,
    LastActiveAdministrator,
    Database(sqlx::Error),
}

impl std::fmt::Display for HardDeleteAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::LastActiveAdministrator => f.write_str("last active administrator"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for HardDeleteAccountRepositoryError {}

#[derive(Debug)]
pub enum ActivateAccountRepositoryError {
    AccountNotFound,
    AccountAlreadyActive,
    AccountSoftDeleted,
    Database(sqlx::Error),
}

impl std::fmt::Display for ActivateAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::AccountAlreadyActive => f.write_str("account already active"),
            Self::AccountSoftDeleted => f.write_str("account is soft deleted"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ActivateAccountRepositoryError {}

#[derive(Debug)]
pub enum RestoreAccountRepositoryError {
    AccountNotFound,
    AccountNotDeleted,
    Database(sqlx::Error),
}

impl std::fmt::Display for RestoreAccountRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccountNotFound => f.write_str("account not found"),
            Self::AccountNotDeleted => f.write_str("account is not soft deleted"),
            Self::Database(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RestoreAccountRepositoryError {}