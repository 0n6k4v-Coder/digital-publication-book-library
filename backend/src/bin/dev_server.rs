use std::{env, error::Error, net::SocketAddr, num::NonZeroUsize, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use digital_publication_backend::{
    app::{router::build_router, state::AppState},
    shared::validation::PasswordBlocklist,
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;

type DynError = Box<dyn Error + Send + Sync>;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:3000";
const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 10;
const PASSWORD_HASH_CONCURRENCY: usize = 4;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    install_rustls_provider()?;

    let database_url = required_env("DATABASE_URL")?;

    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned())
        .parse::<SocketAddr>()?;

    let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_DATABASE_MAX_CONNECTIONS);

    let tls_cert_path = required_env("TLS_CERT_PATH")?;
    let tls_key_path = required_env("TLS_KEY_PATH")?;

    let pool = PgPoolOptions::new()
        .max_connections(database_max_connections)
        .connect(&database_url)
        .await?;

    // Development-only password blocklist.
    //
    // The production server intentionally requires the validated
    // external password-blocklist snapshot. The dev server uses the
    // same PasswordPolicy implementation with an in-memory blocklist
    // so the local Docker stack does not need the full external snapshot.
    let blocklist = Arc::new(PasswordBlocklist::from_hashes(
        "development",
        std::iter::empty::<[u8; 20]>(),
    ));

    let state = AppState::new(
        pool,
        blocklist,
        NonZeroUsize::new(PASSWORD_HASH_CONCURRENCY).expect("non-zero"),
    );

    let router = build_router(state);

    let tls_config = RustlsConfig::from_pem_file(&tls_cert_path, &tls_key_path).await?;

    info!(
        address = %bind_addr,
        "development backend listening with HTTPS"
    );

    axum_server::bind_rustls(bind_addr, tls_config)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

fn required_env(name: &'static str) -> Result<String, DynError> {
    env::var(name).map_err(|_| format!("{name} is required").into())
}

fn install_rustls_provider() -> Result<(), DynError> {
    match rustls::crypto::ring::default_provider().install_default() {
        Ok(()) | Err(_) => Ok(()),
    }
}
