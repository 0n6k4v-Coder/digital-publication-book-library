use axum::{
    body::to_bytes,
    extract::{
        rejection::{JsonRejection, QueryRejection},
        FromRequestParts, Path, Query, Request, State,
    },
    http::{header, request::Parts, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    app::state::AppState,
    domains::{
        authentication::model::AuthenticatedPrincipal,
        authorization::{
            extractor::{
                AuthorizedAccountActivate, AuthorizedAccountCreate, AuthorizedAccountDeactivate,
                AuthorizedAccountDelete, AuthorizedAccountUpdate,
            },
            repository::AuthorizationRepository,
            service::{authorize, ACCOUNT_VIEW_DELETED_PERMISSION, ACCOUNT_VIEW_PERMISSION},
        },
    },
    shared::{error::AppError, response::add_no_store},
};

use super::{
    model::{
        AccountListResponse, AccountResponse, CreateAccountRequest, ListAccountsQuery,
        UpdateAccountRequest,
    },
    repository::AccountRepository,
    service::AccountService,
};

const MAX_UPDATE_ACCOUNT_BODY_BYTES: usize = 2 * 1024 * 1024;

pub async fn create_account(
    authorized: AuthorizedAccountCreate,
    State(state): State<AppState>,
    request: Result<Json<CreateAccountRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = request.map_err(|_| crate::shared::error::invalid_json_response())?;

    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service
        .create_account(authorized.account_id(), request)
        .await?;

    let account_id = account.id;
    let response_body = AccountResponse::from(account);

    let mut response = (StatusCode::CREATED, Json(response_body)).into_response();

    add_no_store(response.headers_mut());

    response.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(&format!("/admin/accounts/{account_id}"))
            .map_err(|_| AppError::Internal)?,
    );

    Ok(response)
}

pub async fn view_accounts(
    principal: AuthenticatedPrincipal,
    State(state): State<AppState>,
    query: Result<Query<ListAccountsQuery>, QueryRejection>,
) -> Result<Response, AppError> {
    let Query(query) = query
        .map_err(|_| AppError::InvalidRequest("The query parameters are malformed or invalid."))?;

    let authorization_repository = AuthorizationRepository::new(state.pool.clone());

    authorize(
        &authorization_repository,
        &principal,
        ACCOUNT_VIEW_PERMISSION,
    )
    .await?;

    if query.include_deleted {
        authorize(
            &authorization_repository,
            &principal,
            ACCOUNT_VIEW_DELETED_PERMISSION,
        )
        .await?;
    }

    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let accounts = service.list_accounts(query).await?;
    let response_body = AccountListResponse::from(accounts);

    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

#[derive(Debug, Clone, Copy)]
pub struct AccountId(Uuid);

impl<S> FromRequestParts<S> for AccountId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(account_id) = Path::<Uuid>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::InvalidAccountId)?;

        Ok(Self(account_id))
    }
}

pub async fn view_account(
    AccountId(account_id): AccountId,
    principal: AuthenticatedPrincipal,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    authorize(
        &AuthorizationRepository::new(state.pool.clone()),
        &principal,
        ACCOUNT_VIEW_PERMISSION,
    )
    .await?;

    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service.view_account(account_id).await?;
    let response_body = AccountResponse::from(account);

    let mut response = (StatusCode::OK, Json(response_body)).into_response();

    add_no_store(response.headers_mut());

    Ok(response)
}

pub async fn update_account(
    AccountId(account_id): AccountId,
    authorized: AuthorizedAccountUpdate,
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, AppError> {
    if !has_merge_patch_content_type(&request) {
        return Err(AppError::UnsupportedMediaType);
    }

    let body = to_bytes(request.into_body(), MAX_UPDATE_ACCOUNT_BODY_BYTES)
        .await
        .map_err(|_| {
            AppError::InvalidRequest("The request body is malformed or exceeds the allowed size.")
        })?;

    let value = serde_json::from_slice::<Value>(&body)
        .map_err(|_| AppError::InvalidRequest("The request body must contain valid JSON."))?;

    let patch = serde_json::from_value::<UpdateAccountRequest>(value).map_err(|_| {
        AppError::Validation("The PATCH document contains unsupported fields or invalid values.")
    })?;

    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service
        .update_account(authorized.account_id(), account_id, patch)
        .await?;

    let response_body = AccountResponse::from(account);

    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

pub async fn deactivate_account(
    AccountId(account_id): AccountId,
    authorized: AuthorizedAccountDeactivate,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service
        .deactivate_account(authorized.account_id(), account_id)
        .await?;

    let response_body = AccountResponse::from(account);

    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

pub async fn activate_account(
    AccountId(account_id): AccountId,
    authorized: AuthorizedAccountActivate,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    let account = service
        .activate_account(authorized.account_id(), account_id)
        .await?;

    let response_body = AccountResponse::from(account);

    let mut response = (StatusCode::OK, Json(response_body)).into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

pub async fn soft_delete_account(
    AccountId(account_id): AccountId,
    authorized: AuthorizedAccountDelete,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let service = AccountService::new(
        AccountRepository::new(state.pool.clone()),
        state.password_policy.clone(),
        state.password_hash_semaphore.clone(),
    );

    service
        .soft_delete_account(authorized.account_id(), account_id)
        .await?;

    let mut response = StatusCode::NO_CONTENT.into_response();
    add_no_store(response.headers_mut());

    Ok(response)
}

fn has_merge_patch_content_type(request: &Request) -> bool {
    request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| {
            value
                .trim()
                .eq_ignore_ascii_case("application/merge-patch+json")
        })
}
