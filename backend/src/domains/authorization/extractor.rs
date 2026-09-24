use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::{
    app::state::AppState,
    domains::authentication::{extractor::authenticate_request, model::AuthenticatedPrincipal},
    shared::error::AppError,
};

use super::{
    repository::AuthorizationRepository,
    service::{
        authorize, ACCOUNT_ACTIVATE_PERMISSION, ACCOUNT_CREATE_PERMISSION,
        ACCOUNT_DEACTIVATE_PERMISSION, ACCOUNT_DELETE_PERMISSION, ACCOUNT_PURGE_PERMISSION,
        ACCOUNT_RESTORE_PERMISSION, ACCOUNT_UPDATE_PERMISSION,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountCreate {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountCreate {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountCreate {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_CREATE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountUpdate {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountUpdate {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountUpdate {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_UPDATE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountDeactivate {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountDeactivate {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountDeactivate {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_DEACTIVATE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountDelete {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountDelete {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountDelete {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_DELETE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountPurge {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountPurge {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountPurge {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_PURGE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountActivate {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountActivate {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountActivate {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_ACTIVATE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizedAccountRestore {
    principal: AuthenticatedPrincipal,
}

impl AuthorizedAccountRestore {
    pub fn account_id(self) -> uuid::Uuid {
        self.principal.account_id
    }
}

impl FromRequestParts<AppState> for AuthorizedAccountRestore {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let principal = authenticate_request(parts, state).await?;

        authorize(
            &AuthorizationRepository::new(state.pool.clone()),
            &principal,
            ACCOUNT_RESTORE_PERMISSION,
        )
        .await?;

        Ok(Self { principal })
    }
}
