use axum::{
    extract::{
        rejection::{JsonRejection, QueryRejection},
        Path, Query, State,
    },
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    app::state::AppState,
    domains::{
        authentication::model::AuthenticatedPrincipal,
        authorization::{
            repository::AuthorizationRepository,
            service::{authorize, ACCOUNT_VIEW_DELETED_PERMISSION, ACCOUNT_VIEW_PERMISSION},
        },
    },
    shared::error::AppError,
    shared::response::add_no_store,
};

use super::{
    model::{AccountListResponse, AccountResponse, CreateAccountRequest, ListAccountsQuery},
    repository::AccountRepository,
    service::AccountService,
};

pub async fn create_account(
    authorized: crate::domains::authorization::extractor::AuthorizedAccountCreate,
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

pub async fn view_account(
    Path(account_id): Path<String>,
    principal: AuthenticatedPrincipal,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let account_id = Uuid::parse_str(&account_id).map_err(|_| AppError::InvalidAccountId)?;

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
