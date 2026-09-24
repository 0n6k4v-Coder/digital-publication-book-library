use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub email: String,
    pub password: SecretString,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateAccountRequest {
    #[serde(default)]
    pub display_name: Option<Option<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub status: Option<String>,
    #[serde(default)]
    pub include_deleted: bool,
}

impl ListAccountsQuery {
    pub fn validate(&self) -> Result<(), ListAccountsQueryValidationError> {
        if self.page == 0 {
            return Err(ListAccountsQueryValidationError::PageMustBePositive);
        }

        if self.page_size == 0 || self.page_size > 100 {
            return Err(ListAccountsQueryValidationError::PageSizeOutOfRange);
        }

        if self
            .status
            .as_deref()
            .is_some_and(|status| status != "active" && status != "inactive")
        {
            return Err(ListAccountsQueryValidationError::InvalidStatus);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListAccountsQueryValidationError {
    PageMustBePositive,
    PageSizeOutOfRange,
    InvalidStatus,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
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

#[derive(Debug, Serialize)]
pub struct AccountListResponse {
    pub items: Vec<AccountResponse>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
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

#[derive(Debug)]
pub struct ViewedAccount {
    pub id: Uuid,
    pub email: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct ListedAccount {
    pub id: Uuid,
    pub email: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct ListedAccounts {
    pub items: Vec<ListedAccount>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
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

impl From<ViewedAccount> for AccountResponse {
    fn from(account: ViewedAccount) -> Self {
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

impl From<ListedAccount> for AccountResponse {
    fn from(account: ListedAccount) -> Self {
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

impl From<ListedAccounts> for AccountListResponse {
    fn from(accounts: ListedAccounts) -> Self {
        Self {
            items: accounts
                .items
                .into_iter()
                .map(AccountResponse::from)
                .collect(),
            page: accounts.page,
            page_size: accounts.page_size,
            total: accounts.total,
        }
    }
}
