use crate::{
    domains::authentication::model::AuthenticatedPrincipal,
    shared::error::{internal_error, AppError},
};

use super::repository::AuthorizationRepository;

pub const ACCOUNT_VIEW_PERMISSION: &str = "account:view";
pub const ACCOUNT_VIEW_DELETED_PERMISSION: &str = "account:view_deleted";
pub const ACCOUNT_CREATE_PERMISSION: &str = "account:create";
pub const ACCOUNT_UPDATE_PERMISSION: &str = "account:update";
pub const ACCOUNT_DEACTIVATE_PERMISSION: &str = "account:deactivate";
pub const ACCOUNT_DELETE_PERMISSION: &str = "account:delete";
pub const ACCOUNT_PURGE_PERMISSION: &str = "account:purge";
pub const ACCOUNT_ACTIVATE_PERMISSION: &str = "account:activate";
pub const ACCOUNT_RESTORE_PERMISSION: &str = "account:restore";

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
