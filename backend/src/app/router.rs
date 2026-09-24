use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::{
    app::state::AppState,
    domains::{
        account::handler::{
            activate_account, change_email, change_password, create_account, deactivate_account,
            hard_delete_account, restore_account, soft_delete_account, update_account,
            view_account, view_accounts,
        },
        authentication::handler::{login, logout, refresh},
    },
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/admin/accounts", get(view_accounts).post(create_account))
        .route(
            "/admin/accounts/{id}",
            get(view_account)
                .patch(update_account)
                .delete(soft_delete_account),
        )
        .route("/admin/accounts/{id}/email", patch(change_email))
        .route("/admin/accounts/{id}/password", patch(change_password))
        .route("/admin/accounts/{id}/purge", delete(hard_delete_account))
        .route("/admin/accounts/{id}/deactivate", post(deactivate_account))
        .route("/admin/accounts/{id}/activate", post(activate_account))
        .route("/admin/accounts/{id}/restore", post(restore_account))
        .with_state(state)
}
