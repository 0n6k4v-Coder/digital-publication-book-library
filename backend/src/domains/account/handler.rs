use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    app::state::AppState,
    shared::{
        auth::AuthenticatedAdmin,
        error::AppError,
        response::add_no_store,
    },
};

use super::{
    model::{AccountResponse, CreateAccountRequest},
    repository::AccountRepository,
    service::AccountService,
};

pub async fn create_account(
    auth: AuthenticatedAdmin,
    State(state): State<AppState>,
    request: Result<Json<CreateAccountRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) =
        request.map_err(|_| crate::shared::error::invalid_json_response())?;

    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service
        .create_account(auth.account_id, request)
        .await?;

    let account_id = account.id;
    let response_body = AccountResponse::from(account);

    let mut response =
        (StatusCode::CREATED, Json(response_body)).into_response();

    add_no_store(response.headers_mut());

    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/admin/accounts/{account_id}"))
            .map_err(|_| AppError::Internal)?,
    );

    Ok(response)
}