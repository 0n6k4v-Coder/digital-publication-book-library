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
    #[error("invalid request: {0}")]
    InvalidRequest(&'static str),
    #[error("validation error: {0}")]
    Validation(&'static str),
    #[error("invalid account id")]
    InvalidAccountId,
    #[error("account not found")]
    AccountNotFound,
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
        let is_unauthorized = matches!(&self, Self::Unauthorized);
        let problem = self.problem();

        let status =
            StatusCode::from_u16(problem.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let mut response = (status, Json(problem)).into_response();

        add_no_store(response.headers_mut());

        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );

        if is_unauthorized {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(r#"Bearer realm="admin-api""#),
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
