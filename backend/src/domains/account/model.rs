use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub email: String,
    pub password: SecretString,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub email: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct CreatedAccount {
    pub id: Uuid,
    pub email: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

impl From<CreatedAccount> for AccountResponse {
    fn from(account: CreatedAccount) -> Self {
        Self {
            id: account.id,
            email: account.email,
            status: account.status,
            created_at: account.created_at,
            updated_at: account.updated_at,
            deleted_at: account.deleted_at,
        }
    }
}
