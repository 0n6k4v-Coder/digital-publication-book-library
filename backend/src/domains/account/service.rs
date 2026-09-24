use std::sync::Arc;

use secrecy::SecretString;
use thiserror::Error;
use tokio::sync::Semaphore;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

use crate::shared::{
    error::AppError,
    validation::{
        hash_password, normalize_email, EmailValidationError, PasswordPolicy,
        PasswordValidationError,
    },
};

use super::{
    model::{
        CreateAccountRequest, CreatedAccount, ListAccountsQuery, ListAccountsQueryValidationError,
        ListedAccounts, UpdateAccountRequest, ViewedAccount,
    },
    repository::{
        AccountRepository, ActivateAccountRepositoryError, CreateAccountRepositoryError,
        DeactivateAccountRepositoryError, ListAccountsRepositoryError,
        SoftDeleteAccountRepositoryError, UpdateAccountRepositoryError, ViewAccountRepositoryError,
    },
};

pub struct AccountService {
    repository: AccountRepository,
    password_policy: PasswordPolicy,
    password_hash_semaphore: Arc<Semaphore>,
}

impl AccountService {
    pub fn new(
        repository: AccountRepository,
        password_policy: PasswordPolicy,
        password_hash_concurrency: Arc<Semaphore>,
    ) -> Self {
        Self {
            repository,
            password_policy,
            password_hash_semaphore: password_hash_concurrency,
        }
    }

    pub async fn create_account(
        &self,
        actor_id: Uuid,
        request: CreateAccountRequest,
    ) -> Result<CreatedAccount, AppError> {
        let email = normalize_email(&request.email).map_err(map_email_validation)?;

        self.password_policy
            .validate(&request.password)
            .map_err(map_password_validation)?;

        let password_hash = self.hash_password(request.password).await?;

        self.repository
            .create(
                actor_id,
                &email.canonical,
                &email.normalized,
                &password_hash,
            )
            .await
            .map_err(|error| match error {
                CreateAccountRepositoryError::EmailAlreadyInUse => AppError::EmailAlreadyInUse,
                CreateAccountRepositoryError::Database(error) => {
                    crate::shared::error::internal_error(error)
                }
            })
    }

    pub async fn list_accounts(
        &self,
        request: ListAccountsQuery,
    ) -> Result<ListedAccounts, AppError> {
        request
            .validate()
            .map_err(map_list_accounts_query_validation)?;

        self.repository
            .list(
                request.page,
                request.page_size,
                request.status.as_deref(),
                request.include_deleted,
            )
            .await
            .map_err(|error| match error {
                ListAccountsRepositoryError::Database(error) => {
                    crate::shared::error::internal_error(error)
                }
            })
    }

    pub async fn view_account(&self, account_id: Uuid) -> Result<ViewedAccount, AppError> {
        self.repository
            .find_by_id(account_id)
            .await
            .map_err(|error| match error {
                ViewAccountRepositoryError::Database(error) => {
                    crate::shared::error::internal_error(error)
                }
            })?
            .ok_or(AppError::AccountNotFound)
    }

    pub async fn update_account(
        &self,
        actor_id: Uuid,
        account_id: Uuid,
        request: UpdateAccountRequest,
    ) -> Result<ViewedAccount, AppError> {
        let display_name = match request.display_name {
            Some(None) => None,
            Some(Some(value)) => {
                Some(normalize_display_name(&value).map_err(map_display_name_validation)?)
            }
            None => {
                return Err(AppError::Validation(
                    "The PATCH document must contain at least one supported field.",
                ))
            }
        };

        self.repository
            .update_display_name(account_id, actor_id, display_name.as_deref())
            .await
            .map_err(|error| match error {
                UpdateAccountRepositoryError::Database(error) => {
                    crate::shared::error::internal_error(error)
                }
            })?
            .ok_or(AppError::AccountNotFound)
    }

    pub async fn deactivate_account(
        &self,
        actor_id: Uuid,
        account_id: Uuid,
    ) -> Result<ViewedAccount, AppError> {
        self.repository
            .deactivate(account_id, actor_id)
            .await
            .map_err(map_deactivate_account_repository_error)
    }

    pub async fn soft_delete_account(
        &self,
        actor_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), AppError> {
        self.repository
            .soft_delete(account_id, actor_id)
            .await
            .map_err(map_soft_delete_account_repository_error)
    }

    pub async fn activate_account(
        &self,
        actor_id: Uuid,
        account_id: Uuid,
    ) -> Result<ViewedAccount, AppError> {
        self.repository
            .activate(account_id, actor_id)
            .await
            .map_err(map_activate_account_repository_error)
    }

    async fn hash_password(&self, password: SecretString) -> Result<String, AppError> {
        let permit = self
            .password_hash_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::Internal)?;

        let result = tokio::task::spawn_blocking(move || hash_password(password)).await;

        drop(permit);

        match result {
            Ok(Ok(password_hash)) => Ok(password_hash),
            Ok(Err(_)) | Err(_) => Err(AppError::Internal),
        }
    }
}

pub fn normalize_display_name(input: &str) -> Result<String, DisplayNameValidationError> {
    let trimmed = input.trim();
    let normalized = trimmed.nfc().collect::<String>();
    let length = normalized.chars().count();

    match length {
        0 => Err(DisplayNameValidationError::Empty),
        1..=100 => Ok(normalized),
        _ => Err(DisplayNameValidationError::TooLong),
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DisplayNameValidationError {
    #[error("display name is empty")]
    Empty,
    #[error("display name is too long")]
    TooLong,
}

fn map_deactivate_account_repository_error(error: DeactivateAccountRepositoryError) -> AppError {
    match error {
        DeactivateAccountRepositoryError::AccountNotFound => AppError::AccountNotFound,
        DeactivateAccountRepositoryError::AccountAlreadyInactive => {
            AppError::AccountAlreadyInactive
        }
        DeactivateAccountRepositoryError::LastActiveAdministrator => {
            AppError::LastActiveAdministrator
        }
        DeactivateAccountRepositoryError::Database(error) => {
            crate::shared::error::internal_error(error)
        }
    }
}

fn map_soft_delete_account_repository_error(error: SoftDeleteAccountRepositoryError) -> AppError {
    match error {
        SoftDeleteAccountRepositoryError::AccountNotFound => AppError::AccountNotFound,
        SoftDeleteAccountRepositoryError::AccountAlreadyDeleted => AppError::AccountAlreadyDeleted,
        SoftDeleteAccountRepositoryError::LastActiveAdministrator => {
            AppError::LastActiveAdministrator
        }
        SoftDeleteAccountRepositoryError::Database(error) => {
            crate::shared::error::internal_error(error)
        }
    }
}

fn map_activate_account_repository_error(error: ActivateAccountRepositoryError) -> AppError {
    match error {
        ActivateAccountRepositoryError::AccountNotFound => AppError::AccountNotFound,
        ActivateAccountRepositoryError::AccountAlreadyActive => AppError::AccountAlreadyActive,
        ActivateAccountRepositoryError::AccountSoftDeleted => AppError::AccountSoftDeleted,
        ActivateAccountRepositoryError::Database(error) => {
            crate::shared::error::internal_error(error)
        }
    }
}

fn map_email_validation(error: EmailValidationError) -> AppError {
    match error {
        EmailValidationError::InvalidFormat | EmailValidationError::InvalidDomain => {
            AppError::Validation("Email must be a valid modern addr-spec.")
        }
        EmailValidationError::TooLong => {
            AppError::Validation("Email must not exceed 254 characters.")
        }
    }
}

fn map_password_validation(error: PasswordValidationError) -> AppError {
    match error {
        PasswordValidationError::TooShort => {
            AppError::Validation("Password must contain at least 15 characters.")
        }
        PasswordValidationError::Blocklisted => {
            AppError::Validation("Password is commonly used or compromised and cannot be used.")
        }
    }
}

fn map_list_accounts_query_validation(error: ListAccountsQueryValidationError) -> AppError {
    match error {
        ListAccountsQueryValidationError::PageMustBePositive => {
            AppError::Validation("Page must be at least 1.")
        }
        ListAccountsQueryValidationError::PageSizeOutOfRange => {
            AppError::Validation("Page size must be between 1 and 100.")
        }
        ListAccountsQueryValidationError::InvalidStatus => {
            AppError::Validation("Status must be active or inactive.")
        }
    }
}

fn map_display_name_validation(error: DisplayNameValidationError) -> AppError {
    match error {
        DisplayNameValidationError::Empty | DisplayNameValidationError::TooLong => {
            AppError::Validation(
                "Display name must contain between 1 and 100 Unicode scalar values after trimming.",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_mapping_does_not_expose_input_values() {
        let error = map_password_validation(PasswordValidationError::Blocklisted);

        assert!(matches!(
            error,
            AppError::Validation("Password is commonly used or compromised and cannot be used.")
        ));
    }

    #[test]
    fn deactivation_repository_errors_map_to_account_errors() {
        assert!(matches!(
            map_deactivate_account_repository_error(
                DeactivateAccountRepositoryError::AccountAlreadyInactive
            ),
            AppError::AccountAlreadyInactive
        ));
        assert!(matches!(
            map_deactivate_account_repository_error(
                DeactivateAccountRepositoryError::LastActiveAdministrator
            ),
            AppError::LastActiveAdministrator
        ));
        assert!(matches!(
            map_deactivate_account_repository_error(
                DeactivateAccountRepositoryError::AccountNotFound
            ),
            AppError::AccountNotFound
        ));
    }

    #[test]
    fn soft_delete_repository_errors_map_to_account_errors() {
        assert!(matches!(
            map_soft_delete_account_repository_error(
                SoftDeleteAccountRepositoryError::AccountAlreadyDeleted
            ),
            AppError::AccountAlreadyDeleted
        ));
        assert!(matches!(
            map_soft_delete_account_repository_error(
                SoftDeleteAccountRepositoryError::LastActiveAdministrator
            ),
            AppError::LastActiveAdministrator
        ));
        assert!(matches!(
            map_soft_delete_account_repository_error(
                SoftDeleteAccountRepositoryError::AccountNotFound
            ),
            AppError::AccountNotFound
        ));
    }

    #[test]
    fn activation_repository_errors_map_to_account_errors() {
        assert!(matches!(
            map_activate_account_repository_error(
                ActivateAccountRepositoryError::AccountAlreadyActive
            ),
            AppError::AccountAlreadyActive
        ));
        assert!(matches!(
            map_activate_account_repository_error(
                ActivateAccountRepositoryError::AccountSoftDeleted
            ),
            AppError::AccountSoftDeleted
        ));
        assert!(matches!(
            map_activate_account_repository_error(ActivateAccountRepositoryError::AccountNotFound),
            AppError::AccountNotFound
        ));
    }
}
