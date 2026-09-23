use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};
use sha2::{Digest, Sha256};

use crate::{app::state::AppState, shared::error::AppError};

use super::{model::AuthenticatedPrincipal, repository::AuthenticationRepository};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BearerAuthError {
    Missing,
    Invalid,
}

pub fn parse_bearer_token(headers: &HeaderMap) -> Result<&str, BearerAuthError> {
    let header_value = headers
        .get(header::AUTHORIZATION)
        .ok_or(BearerAuthError::Missing)?;
    let header_value = header_value
        .to_str()
        .map_err(|_| BearerAuthError::Invalid)?;

    let mut parts = header_value.split(' ').filter(|part| !part.is_empty());
    let scheme = parts.next().ok_or(BearerAuthError::Invalid)?;
    let token = parts.next().ok_or(BearerAuthError::Invalid)?;

    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(BearerAuthError::Invalid);
    }

    Ok(token)
}

pub fn sha256_token_verifier(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    let mut verifier = String::with_capacity(digest.len() * 2);

    for byte in digest {
        use std::fmt::Write;
        write!(&mut verifier, "{byte:02x}").expect("writing to String cannot fail");
    }

    verifier
}

pub async fn authenticate_request(
    parts: &mut Parts,
    state: &AppState,
) -> Result<AuthenticatedPrincipal, AppError> {
    let token = parse_bearer_token(&parts.headers).map_err(|error| match error {
        BearerAuthError::Missing => AppError::Unauthorized,
        BearerAuthError::Invalid => AppError::InvalidAuthentication,
    })?;

    let token_hash = sha256_token_verifier(token);

    AuthenticationRepository::new(state.pool.clone())
        .validate_access_token(&token_hash)
        .await
        .map_err(crate::shared::error::internal_error)?
        .ok_or(AppError::InvalidAuthentication)
}

impl FromRequestParts<AppState> for AuthenticatedPrincipal {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        authenticate_request(parts, state).await
    }
}
