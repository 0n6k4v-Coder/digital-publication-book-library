use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    app::state::AppState,
    domains::account::handler::{
        activate_account, create_account, deactivate_account, soft_delete_account, update_account,
        view_account, view_accounts,
    },
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/admin/accounts", get(view_accounts).post(create_account))
        .route(
            "/admin/accounts/{id}",
            get(view_account)
                .patch(update_account)
                .delete(soft_delete_account),
        )
        .route("/admin/accounts/{id}/deactivate", post(deactivate_account))
        .route("/admin/accounts/{id}/activate", post(activate_account))
        .with_state(state)
}
