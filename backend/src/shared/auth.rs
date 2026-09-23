use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use super::error::AppError;

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedAdmin {
    pub account_id: Uuid,
}

impl AuthenticatedAdmin {
    /// Creates an authenticated administrator identity after an
    /// upstream authentication and authorization layer has
    /// successfully verified the caller.
    pub fn from_verified_account(account_id: Uuid) -> Self {
        Self { account_id }
    }
}

impl<S> FromRequestParts<S> for AuthenticatedAdmin
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedAdmin>()
            .copied()
            .ok_or(AppError::Unauthorized)
    }
}
