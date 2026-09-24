use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

use super::response::add_no_store;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid authentication")]
    InvalidAuthentication,
    #[error("forbidden")]
    Forbidden,
    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
    #[error("validation error: {0}")]
    Validation(&'static str),
    #[error("unsupported media type")]
    UnsupportedMediaType,
    #[error("invalid account id")]
    InvalidAccountId,
    #[error("account not found")]
    AccountNotFound,
    #[error("account already inactive")]
    AccountAlreadyInactive,
    #[error("last active administrator")]
    LastActiveAdministrator,
    #[error("account already active")]
    AccountAlreadyActive,
    #[error("account is soft deleted")]
    AccountSoftDeleted,
    #[error("account already deleted")]
    AccountAlreadyDeleted,
    #[error("account is not soft deleted")]
    AccountNotDeleted,
    #[error("email already in use")]
    EmailAlreadyInUse,
    #[error("internal server error")]
    Internal,
}

impl AppError {
    fn problem(&self) -> ProblemDetails {
        let (status, code, title, detail, type_uri) = match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required",
                "Authentication is required to access this resource.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/unauthorized",
            ),
            Self::InvalidAuthentication => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required",
                "The supplied authentication credentials are invalid or no longer valid.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/unauthorized",
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Forbidden",
                "The authenticated principal is not authorized to perform this operation.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/forbidden",
            ),
            Self::InvalidRequest(detail) => (
                StatusCode::BAD_REQUEST,
                "INVALID_REQUEST",
                "Invalid request",
                *detail,
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/invalid-request",
            ),
            Self::Validation(detail) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "Validation failed",
                *detail,
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/validation-error",
            ),
            Self::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "UNSUPPORTED_MEDIA_TYPE",
                "Unsupported media type",
                "The request Content-Type must be application/merge-patch+json.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/unsupported-media-type",
            ),
            Self::InvalidAccountId => (
                StatusCode::BAD_REQUEST,
                "INVALID_ACCOUNT_ID",
                "Invalid account ID",
                "The account ID must be a valid UUID.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/invalid-account-id",
            ),
            Self::AccountNotFound => (
                StatusCode::NOT_FOUND,
                "ACCOUNT_NOT_FOUND",
                "Account not found",
                "The requested account was not found.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-not-found",
            ),
            Self::AccountAlreadyInactive => (
                StatusCode::CONFLICT,
                "ACCOUNT_ALREADY_INACTIVE",
                "Account already inactive",
                "The requested account is already inactive.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-already-inactive",
            ),
            Self::LastActiveAdministrator => (
                StatusCode::CONFLICT,
                "LAST_ACTIVE_ADMINISTRATOR",
                "Last active administrator",
                "The last active administrator account cannot be deactivated.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/last-active-administrator",
            ),
            Self::AccountAlreadyActive => (
                StatusCode::CONFLICT,
                "ACCOUNT_ALREADY_ACTIVE",
                "Account already active",
                "The requested account is already active.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-already-active",
            ),
            Self::AccountSoftDeleted => (
                StatusCode::CONFLICT,
                "ACCOUNT_SOFT_DELETED",
                "Account is soft deleted",
                "The requested account is soft-deleted and cannot be activated.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-soft-deleted",
            ),
            Self::AccountAlreadyDeleted => (
                StatusCode::CONFLICT,
                "ACCOUNT_ALREADY_DELETED",
                "Account already deleted",
                "The requested account is already soft-deleted.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-already-deleted",
            ),
            Self::AccountNotDeleted => (
                StatusCode::CONFLICT,
                "ACCOUNT_NOT_DELETED",
                "Account not deleted",
                "The requested account is not soft-deleted.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/account-not-deleted",
            ),
            Self::EmailAlreadyInUse => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_IN_USE",
                "Email already in use",
                "The supplied email address is already associated with an account.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/email-already-in-use",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Internal server error",
                "The server could not complete the request.",
                "https://github.com/0n6k4v-Coder/digital-publication-book-library/problems/internal-server-error",
            ),
        };

        ProblemDetails {
            r#type: type_uri,
            title,
            status: status.as_u16(),
            detail,
            code,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let auth_challenge = match &self {
            AppError::Unauthorized => Some(r#"Bearer realm="admin-api""#),
            AppError::InvalidAuthentication => {
                Some(r#"Bearer realm="admin-api", error="invalid_token""#)
            }
            _ => None,
        };

        let problem = self.problem();
        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut response = (status, Json(problem)).into_response();

        add_no_store(response.headers_mut());

        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );

        if let Some(challenge) = auth_challenge {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(challenge),
            );
        }

        response
    }
}

#[derive(Debug, Serialize)]
pub struct ProblemDetails {
    pub r#type: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub detail: &'static str,
    pub code: &'static str,
}

pub fn internal_error<E>(error: E) -> AppError
where
    E: std::fmt::Display,
{
    error!(error = %error, "account request failed unexpectedly");
    AppError::Internal
}

pub fn invalid_json_response() -> AppError {
    AppError::InvalidRequest("The request body is malformed or has an invalid content type.")
}
