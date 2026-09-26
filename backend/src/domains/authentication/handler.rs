use std::net::SocketAddr;

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, Json, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    app::state::AppState,
    domains::authentication::{
        model::{
            AuthenticateAccountRequest, AuthenticatedPrincipal, AuthenticationResponse,
            AuthenticationTokens, ACCESS_TOKEN_EXPIRES_IN,
        },
        repository::AuthenticationRepository,
        service::AuthenticationService,
    },
    shared::{error::AppError, response::add_no_store},
};

const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";
const REFRESH_COOKIE_PREFIX: &str = "__Host-refresh_token=";
const REFRESH_COOKIE_MAX_AGE_SECONDS: u64 = 86_400;

pub async fn login(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    request: Result<Json<AuthenticateAccountRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = request.map_err(|_| crate::shared::error::invalid_json_response())?;

    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    let tokens = service
        .authenticate_account(remote_addr.ip(), request)
        .await?;

    Ok(authentication_response(tokens))
}

pub async fn refresh(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let refresh_token =
        extract_refresh_token(&headers).ok_or(AppError::InvalidRefreshToken)?;

    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    let tokens = service.refresh_authentication(refresh_token).await?;

    Ok(authentication_response(tokens))
}

pub async fn logout(
    principal: AuthenticatedPrincipal,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    service.revoke_authentication(principal).await?;

    let mut response = StatusCode::NO_CONTENT.into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

fn extract_refresh_token(headers: &HeaderMap) -> Option<SecretString> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .map(str::trim)
        .find_map(|pair| {
            pair.strip_prefix(REFRESH_COOKIE_PREFIX)
                .filter(|value| !value.is_empty())
                .map(|value| SecretString::from(value.to_owned()))
        })
}

fn authentication_response(tokens: AuthenticationTokens) -> Response {
    let refresh_cookie = format!(
        "{REFRESH_COOKIE_NAME}={}; Max-Age={REFRESH_COOKIE_MAX_AGE_SECONDS}; \
         Path=/; Secure; HttpOnly; SameSite=Strict",
        tokens.refresh_token.expose_secret()
    );

    let body = AuthenticationResponse {
        access_token: tokens.access_token.expose_secret().to_owned(),
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRES_IN,
    };

    let mut response = (StatusCode::OK, Json(body)).into_response();
    add_no_store(response.headers_mut());

    let cookie_header: HeaderValue = refresh_cookie
        .parse()
        .expect("generated refresh token must produce a valid Set-Cookie header");

    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie_header);

    response
}

#[cfg(test)]
mod tests {
    use axum::http::{header, HeaderMap, HeaderValue};
    use secrecy::ExposeSecret;

    use super::extract_refresh_token;

    #[test]
    fn extract_refresh_token_reads_the_browser_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static(
                "theme=dark; __Host-refresh_token=refresh-token-value; other=value",
            ),
        );

        let token = extract_refresh_token(&headers).expect("refresh cookie should be present");

        assert_eq!(token.expose_secret(), "refresh-token-value");
    }

    #[test]
    fn extract_refresh_token_returns_none_when_cookie_is_missing() {
        let headers = HeaderMap::new();

        assert!(extract_refresh_token(&headers).is_none());
    }
}