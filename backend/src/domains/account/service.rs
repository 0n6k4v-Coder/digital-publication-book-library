use std::sync::Arc;

use secrecy::SecretString;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::shared::{
    error::AppError,
    validation::{
        hash_password, normalize_email, EmailValidationError, PasswordPolicy,
        PasswordValidationError,
    },
};

use super::{
    model::{CreateAccountRequest, CreatedAccount},
    repository::{AccountRepository, CreateAccountRepositoryError},
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
        password_hash_semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            repository,
            password_policy,
            password_hash_semaphore,
        }
    }

    pub async fn create_account(
        &self,
        actor_id: Uuid,
        request: CreateAccountRequest,
    ) -> Result<CreatedAccount, AppError> {
        let email =
            normalize_email(&request.email).map_err(map_email_validation)?;

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
                CreateAccountRepositoryError::EmailAlreadyInUse => {
                    AppError::EmailAlreadyInUse
                }
                CreateAccountRepositoryError::Database(error) => {
                    crate::shared::error::internal_error(error)
                }
            })
    }

    async fn hash_password(
        &self,
        password: SecretString,
    ) -> Result<String, AppError> {
        let permit = self
            .password_hash_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::Internal)?;

        let result =
            tokio::task::spawn_blocking(move || hash_password(password)).await;

        drop(permit);

        match result {
            Ok(Ok(password_hash)) => Ok(password_hash),
            Ok(Err(_)) | Err(_) => Err(AppError::Internal),
        }
    }
}

fn map_email_validation(error: EmailValidationError) -> AppError {
    match error {
        EmailValidationError::InvalidFormat
        | EmailValidationError::InvalidDomain => {
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
            AppError::Validation(
                "Password must contain at least 15 characters.",
            )
        }
        PasswordValidationError::Blocklisted => AppError::Validation(
            "Password is commonly used or compromised and cannot be used.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_mapping_does_not_expose_input_values() {
        let error =
            map_password_validation(PasswordValidationError::Blocklisted);

        assert!(matches!(
            error,
            AppError::Validation(
                "Password is commonly used or compromised and cannot be used."
            )
        ));
    }
}