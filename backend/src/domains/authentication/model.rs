use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

pub const ACCESS_TOKEN_EXPIRES_IN: u64 = 3_600;
pub const REFRESH_TOKEN_EXPIRES_IN: u64 = 2_592_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    pub account_id: Uuid,
    pub session_id: Uuid,
    pub authenticated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateAccountRequest {
    pub email: String,
    pub password: SecretString,
}

#[derive(Debug, Deserialize)]
pub struct RefreshAuthenticationRequest {
    pub refresh_token: SecretString,
}

#[derive(Debug)]
pub struct AuthenticationTokens {
    pub access_token: SecretString,
    pub refresh_token: SecretString,
}

#[derive(Debug, Serialize)]
pub struct AuthenticationResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub refresh_token: String,
    pub refresh_expires_in: u64,
}
