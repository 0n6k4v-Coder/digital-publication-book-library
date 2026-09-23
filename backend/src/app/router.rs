use axum::{routing::get, Router};

use crate::{
    app::state::AppState,
    domains::account::handler::{create_account, view_accounts},
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/admin/accounts", get(view_accounts).post(create_account))
        .with_state(state)
}
