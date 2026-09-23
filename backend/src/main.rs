use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tracing::info;

use digital_publication_backend::{
    app::{config::Config, router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter("info").init();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    let blocklist = Arc::new(PasswordBlocklist::from_file(
        &config.password_blocklist_path,
    )?);
    info!(version = %blocklist.version(), "password blocklist loaded");

    let state = AppState::new(pool, blocklist, config.password_hash_concurrency);

    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;

    info!(address = %listener.local_addr()?, "backend listening");
    axum::serve(listener, router).await?;

    Ok(())
}
