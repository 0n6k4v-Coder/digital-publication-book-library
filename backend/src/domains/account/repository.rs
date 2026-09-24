use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::{CreatedAccount, ListedAccount, ListedAccounts, ViewedAccount};

const EMAIL_UNIQUE_CONSTRAINT: &str = "account_credentials_email_normalized_key";

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
struct AccountUpdatedRow {
    id: Uuid,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
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
