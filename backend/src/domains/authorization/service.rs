use crate::{
    domains::authentication::model::AuthenticatedPrincipal,
    shared::error::{internal_error, AppError},
};

use super::repository::AuthorizationRepository;

pub const ACCOUNT_VIEW_PERMISSION: &str = "account:view";
pub const ACCOUNT_VIEW_DELETED_PERMISSION: &str = "account:view_deleted";
pub const ACCOUNT_CREATE_PERMISSION: &str = "account:create";
pub const ACCOUNT_UPDATE_PERMISSION: &str = "account:update";

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
