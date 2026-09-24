use secrecy::SecretString;
use serde::{Deserialize, Deserializer, Serialize};
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
    #[serde(default, deserialize_with = "deserialize_nullable_patch")]
    pub display_name: Option<Option<String>>,
}

fn deserialize_nullable_patch<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(deserializer)?))
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

#[derive(Debug, PartialEq, Eq)]
pub struct DeactivationState {
    pub is_active: bool,
    pub is_deleted: bool,
    pub is_administrator: bool,
    pub active_administrator_count: usize,
}

impl DeactivationState {
    pub fn validate(self) -> Result<(), DeactivationValidationError> {
        if self.is_deleted {
            return Err(DeactivationValidationError::AccountNotFound);
        }

        if !self.is_active {
            return Err(DeactivationValidationError::AccountAlreadyInactive);
        }

        if self.is_administrator && self.active_administrator_count <= 1 {
            return Err(DeactivationValidationError::LastActiveAdministrator);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeactivationValidationError {
    AccountNotFound,
    AccountAlreadyInactive,
    LastActiveAdministrator,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SoftDeleteState {
    pub is_active: bool,
    pub is_deleted: bool,
    pub is_administrator: bool,
    pub active_administrator_count: usize,
}

impl SoftDeleteState {
    pub fn validate(self) -> Result<(), SoftDeleteValidationError> {
        if self.is_deleted {
            return Err(SoftDeleteValidationError::AccountAlreadyDeleted);
        }

        if self.is_active && self.is_administrator && self.active_administrator_count <= 1 {
            return Err(SoftDeleteValidationError::LastActiveAdministrator);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SoftDeleteValidationError {
    AccountAlreadyDeleted,
    LastActiveAdministrator,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HardDeleteState {
    pub is_active: bool,
    pub is_deleted: bool,
    pub is_administrator: bool,
    pub active_administrator_count: usize,
}

impl HardDeleteState {
    pub fn validate(self) -> Result<(), HardDeleteValidationError> {
        if self.is_active
            && !self.is_deleted
            && self.is_administrator
            && self.active_administrator_count <= 1
        {
            return Err(HardDeleteValidationError::LastActiveAdministrator);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum HardDeleteValidationError {
    LastActiveAdministrator,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ActivationState {
    pub is_active: bool,
    pub is_deleted: bool,
}

impl ActivationState {
    pub fn validate(self) -> Result<(), ActivationValidationError> {
        if self.is_deleted {
            return Err(ActivationValidationError::AccountSoftDeleted);
        }

        if self.is_active {
            return Err(ActivationValidationError::AccountAlreadyActive);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActivationValidationError {
    AccountSoftDeleted,
    AccountAlreadyActive,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RestoreState {
    pub is_deleted: bool,
}

impl RestoreState {
    pub fn validate(self) -> Result<(), RestoreValidationError> {
        if !self.is_deleted {
            return Err(RestoreValidationError::AccountNotDeleted);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RestoreValidationError {
    AccountNotDeleted,
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
