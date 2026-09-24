use std::net::SocketAddr;

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use secrecy::ExposeSecret;

use crate::{
    app::state::AppState,
    domains::authentication::{
        model::{
            AuthenticateAccountRequest, AuthenticatedPrincipal, AuthenticationResponse,
            AuthenticationTokens, RefreshAuthenticationRequest, ACCESS_TOKEN_EXPIRES_IN,
            REFRESH_TOKEN_EXPIRES_IN,
        },
        repository::AuthenticationRepository,
        service::AuthenticationService,
    },
    shared::{error::AppError, response::add_no_store},
};

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
    State(state): State<AppState>,
    request: Result<Json<RefreshAuthenticationRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = request.map_err(|_| crate::shared::error::invalid_json_response())?;

    let service = AuthenticationService::new(
        AuthenticationRepository::new(state.pool.clone()),
        state.password_hash_semaphore.clone(),
    );

    let tokens = service.refresh_authentication(request).await?;

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

fn authentication_response(tokens: AuthenticationTokens) -> Response {
    let body = AuthenticationResponse {
        access_token: tokens.access_token.expose_secret().to_owned(),
        token_type: "Bearer",
        expires_in: ACCESS_TOKEN_EXPIRES_IN,
        refresh_token: tokens.refresh_token.expose_secret().to_owned(),
        refresh_expires_in: REFRESH_TOKEN_EXPIRES_IN,
    };

    let mut response = (StatusCode::OK, Json(body)).into_response();
    add_no_store(response.headers_mut());
    response
}
