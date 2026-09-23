use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::CreatedAccount;

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
}

#[derive(Debug, sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    status: String,
    deleted_at: Option<OffsetDateTime>,
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
