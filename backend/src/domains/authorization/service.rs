use thiserror::Error;
use uuid::Uuid;

use crate::{
    domains::authentication::model::AuthenticatedPrincipal,
    shared::error::{internal_error, AppError},
};

use super::{
    model::{RoleAssignmentValidationError, RoleRevocationValidationError},
    repository::{
        AuthorizationRepository, RoleAssignmentRepositoryError, RoleRevocationRepositoryError,
    },
};

pub const ACCOUNT_VIEW_PERMISSION: &str = "account:view";
pub const ACCOUNT_VIEW_DELETED_PERMISSION: &str = "account:view_deleted";
pub const ACCOUNT_CREATE_PERMISSION: &str = "account:create";
pub const ACCOUNT_UPDATE_PERMISSION: &str = "account:update";
pub const ACCOUNT_DEACTIVATE_PERMISSION: &str = "account:deactivate";
pub const ACCOUNT_DELETE_PERMISSION: &str = "account:delete";
pub const ACCOUNT_PURGE_PERMISSION: &str = "account:purge";
pub const ACCOUNT_ACTIVATE_PERMISSION: &str = "account:activate";
pub const ACCOUNT_RESTORE_PERMISSION: &str = "account:restore";
pub const ACCOUNT_CHANGE_EMAIL_PERMISSION: &str = "account:change_email";
pub const ACCOUNT_CHANGE_PASSWORD_PERMISSION: &str = "account:change_password";

pub const AUTHORIZATION_ROLE_ASSIGN_PERMISSION: &str = "authorization:role_assign";
pub const AUTHORIZATION_ROLE_REVOKE_PERMISSION: &str = "authorization:role_revoke";

pub async fn authorize(
    repository: &AuthorizationRepository,
    principal: &AuthenticatedPrincipal,
    required_permission: &str,
) -> Result<(), AppError> {
    let allowed = repository
        .has_permission(principal.account_id, required_permission)
        .await
        .map_err(internal_error)?;

    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub async fn assign_role(
    repository: &AuthorizationRepository,
    principal: &AuthenticatedPrincipal,
    account_id: Uuid,
    role_name: &str,
) -> Result<(), RoleManagementError> {
    authorize(repository, principal, AUTHORIZATION_ROLE_ASSIGN_PERMISSION)
        .await
        .map_err(map_authorization_error)?;

    repository
        .assign_role(account_id, role_name, principal.account_id)
        .await
        .map_err(map_assignment_repository_error)
}

pub async fn revoke_role(
    repository: &AuthorizationRepository,
    principal: &AuthenticatedPrincipal,
    account_id: Uuid,
    role_name: &str,
) -> Result<(), RoleManagementError> {
    authorize(repository, principal, AUTHORIZATION_ROLE_REVOKE_PERMISSION)
        .await
        .map_err(map_authorization_error)?;

    repository
        .revoke_role(account_id, role_name, principal.account_id)
        .await
        .map_err(map_revocation_repository_error)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoleManagementError {
    #[error("forbidden")]
    Forbidden,

    #[error("role not found")]
    RoleNotFound,

    #[error("role is disabled")]
    RoleDisabled,

    #[error("account not found")]
    AccountNotFound,

    #[error("role is already assigned")]
    RoleAlreadyAssigned,

    #[error("role assignment not found")]
    RoleAssignmentNotFound,

    #[error("last active administrator")]
    LastActiveAdministrator,

    #[error("internal authorization error")]
    Internal,
}

fn map_authorization_error(error: AppError) -> RoleManagementError {
    match error {
        AppError::Forbidden => RoleManagementError::Forbidden,
        _ => RoleManagementError::Internal,
    }
}

fn map_assignment_repository_error(error: RoleAssignmentRepositoryError) -> RoleManagementError {
    match error {
        RoleAssignmentRepositoryError::Validation(error) => match error {
            RoleAssignmentValidationError::RoleNotFound => RoleManagementError::RoleNotFound,
            RoleAssignmentValidationError::RoleDisabled => RoleManagementError::RoleDisabled,
            RoleAssignmentValidationError::AccountNotFound => RoleManagementError::AccountNotFound,
            RoleAssignmentValidationError::RoleAlreadyAssigned => {
                RoleManagementError::RoleAlreadyAssigned
            }
        },
        RoleAssignmentRepositoryError::Database(error) => {
            let _ = internal_error(error);
            RoleManagementError::Internal
        }
    }
}

fn map_revocation_repository_error(error: RoleRevocationRepositoryError) -> RoleManagementError {
    match error {
        RoleRevocationRepositoryError::Validation(error) => match error {
            RoleRevocationValidationError::RoleAssignmentNotFound => {
                RoleManagementError::RoleAssignmentNotFound
            }
            RoleRevocationValidationError::LastActiveAdministrator => {
                RoleManagementError::LastActiveAdministrator
            }
        },
        RoleRevocationRepositoryError::Database(error) => {
            let _ = internal_error(error);
            RoleManagementError::Internal
        }
    }
}
