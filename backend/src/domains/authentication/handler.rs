use std::net::SocketAddr;

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, Json, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use secrecy::{ExposeSecret, SecretString};
use time::OffsetDateTime;

use crate::{
    app::state::AppState,
    domains::authentication::{
        model::{
            AuthenticateAccountRequest, AuthenticationResponse, AuthenticationTokens,
            ACCESS_TOKEN_EXPIRES_IN,
        },
        repository::AuthenticationRepository,
        service::AuthenticationService,
    },
    shared::{error::AppError, response::add_no_store},
};

const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";
const REFRESH_COOKIE_PREFIX: &str = "__Host-refresh_token=";
const REFRESH_COOKIE_CLEAR: &str =
    "__Host-refresh_token=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=Strict";

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
    let refresh_token = extract_refresh_token(&headers).ok_or(AppError::InvalidRefreshToken)?;

    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    let tokens = service.refresh_authentication(refresh_token).await?;

    Ok(authentication_response(tokens))
}

pub async fn logout(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let refresh_token = extract_refresh_token(&headers);

    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    service.logout_authentication(refresh_token).await?;

    let mut response = StatusCode::NO_CONTENT.into_response();
    add_no_store(response.headers_mut());
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static(REFRESH_COOKIE_CLEAR),
    );

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

fn refresh_cookie_max_age_seconds(refresh_expires_at: OffsetDateTime) -> u64 {
    (refresh_expires_at - OffsetDateTime::now_utc())
        .whole_seconds()
        .max(0) as u64
}

fn authentication_response(tokens: AuthenticationTokens) -> Response {
    let max_age = refresh_cookie_max_age_seconds(tokens.refresh_expires_at);

    let refresh_cookie = format!(
        "{REFRESH_COOKIE_NAME}={}; Max-Age={max_age}; \
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
