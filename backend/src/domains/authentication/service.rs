use std::{net::IpAddr, sync::Arc};

use argon2::password_hash::{phc::PasswordHash, PasswordVerifier};
use argon2::Argon2;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::shared::{
    error::AppError,
    validation::{normalize_email, EmailValidationError},
};

use super::{
    extractor::sha256_token_verifier,
    model::{
        AuthenticateAccountRequest, AuthenticatedPrincipal, AuthenticationTokens,
        RefreshAuthenticationRequest,
    },
    repository::AuthenticationRepository,
};

const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$Zml4ZWQtZHVtbXktc2FsdA$Zw12+M9Cuwr7ijD/UzcXJuflc7TcqCGB+fdZlYeqFzY";

pub struct AuthenticationService {
    repository: AuthenticationRepository,
    password_hash_semaphore: Arc<Semaphore>,
}

impl AuthenticationService {
    pub fn new(
        repository: AuthenticationRepository,
        password_hash_semaphore: Arc<Semaphore>,
    ) -> Self {
        Self {
            repository,
            password_hash_semaphore,
        }
    }

    pub async fn authenticate_account(
        &self,
        source_ip: IpAddr,
        request: AuthenticateAccountRequest,
    ) -> Result<AuthenticationTokens, AppError> {
        let email = normalize_email(&request.email).map_err(map_login_email_error)?;
        let email_key = sha256_token_verifier(&email.normalized);
        let source_ip_key = sha256_token_verifier(&source_ip.to_string());

        let attempt_id = self
            .repository
            .reserve_login_attempt(&email_key, &source_ip_key)
            .await
            .map_err(crate::shared::error::internal_error)?
            .ok_or(AppError::AuthenticationRateLimited)?;

        let account = self
            .repository
            .find_account_authentication_data(&email.normalized)
            .await
            .map_err(crate::shared::error::internal_error)?;

        let (password_hash, account_id, account_is_active, account_is_deleted) = match account {
            Some(account) => (
                account.password_hash,
                Some(account.account_id),
                account.is_active,
                account.is_deleted,
            ),
            None => (DUMMY_PASSWORD_HASH.to_owned(), None, false, false),
        };

        let password_valid = self.verify_password(request.password, password_hash).await;

        if !password_valid || !account_is_active || account_is_deleted {
            self.repository
                .mark_login_attempt_failed(attempt_id)
                .await
                .map_err(crate::shared::error::internal_error)?;
            return Err(AppError::InvalidCredentials);
        }

        let account_id = account_id.ok_or(AppError::InvalidCredentials)?;
        let access_token = generate_opaque_token();
        let refresh_token = generate_opaque_token();
        let access_token_hash = sha256_token_verifier(access_token.expose_secret());
        let refresh_token_hash = sha256_token_verifier(refresh_token.expose_secret());

        let issued = self
            .repository
            .issue_tokens(
                attempt_id,
                &email_key,
                account_id,
                &access_token_hash,
                &refresh_token_hash,
            )
            .await
            .map_err(crate::shared::error::internal_error)?;

        if !issued {
            return Err(AppError::InvalidCredentials);
        }

        Ok(AuthenticationTokens {
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh_authentication(
        &self,
        request: RefreshAuthenticationRequest,
    ) -> Result<AuthenticationTokens, AppError> {
        let access_token = generate_opaque_token();
        let refresh_token = generate_opaque_token();
        let refresh_token_hash = sha256_token_verifier(request.refresh_token.expose_secret());
        let access_token_hash = sha256_token_verifier(access_token.expose_secret());
        let replacement_refresh_token_hash = sha256_token_verifier(refresh_token.expose_secret());

        let refreshed = self
            .repository
            .refresh_tokens(
                &refresh_token_hash,
                &access_token_hash,
                &replacement_refresh_token_hash,
            )
            .await
            .map_err(crate::shared::error::internal_error)?;

        if !refreshed {
            return Err(AppError::InvalidRefreshToken);
        }

        Ok(AuthenticationTokens {
            access_token,
            refresh_token,
        })
    }

    pub async fn revoke_authentication(
        &self,
        principal: AuthenticatedPrincipal,
    ) -> Result<(), AppError> {
        let revoked = self
            .repository
            .revoke_session(principal.account_id, principal.session_id)
            .await
            .map_err(crate::shared::error::internal_error)?;

        if !revoked {
            return Err(AppError::Unauthorized);
        }

        Ok(())
    }

    async fn verify_password(&self, password: SecretString, password_hash: String) -> bool {
        let permit = match self.password_hash_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => return false,
        };

        let result = tokio::task::spawn_blocking(move || {
            let parsed_hash = match PasswordHash::new(&password_hash) {
                Ok(parsed_hash) => parsed_hash,
                Err(_) => return false,
            };

            Argon2::default()
                .verify_password(password.expose_secret().as_bytes(), &parsed_hash)
                .is_ok()
        })
        .await;

        drop(permit);

        result.unwrap_or(false)
    }
}

pub fn generate_opaque_token() -> SecretString {
    SecretString::from(format!(
        "{}{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    ))
}

fn map_login_email_error(error: EmailValidationError) -> AppError {
    match error {
        EmailValidationError::InvalidFormat
        | EmailValidationError::TooLong
        | EmailValidationError::InvalidDomain => {
            AppError::InvalidRequest("The request contains an invalid email address.")
        }
    }
}
