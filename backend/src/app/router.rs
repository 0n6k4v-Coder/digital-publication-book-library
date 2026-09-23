use axum::{routing::post, Router};

use crate::{app::state::AppState, domains::account::handler::create_account};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/admin/accounts", post(create_account))
        .with_state(state)
}
