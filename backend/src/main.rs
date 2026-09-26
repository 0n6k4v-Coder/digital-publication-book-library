use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum_server::tls_rustls::RustlsConfig;
use sqlx::postgres::PgPoolOptions;
use tokio::time::sleep;
use tracing::{info, warn};

use digital_publication_backend::{
    app::{
        config::{Config, PasswordBlocklistRefreshConfig},
        router::build_router,
        security::apply_security,
        state::AppState,
    },
    shared::validation::{PasswordBlocklist, PASSWORD_BLOCKLIST_MAX_AGE},
};

const REFRESH_RETRY_DELAY: Duration = Duration::from_secs(5 * 60);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter("info").init();

    match std::env::args().nth(1).as_deref() {
        Some("--refresh-password-blocklist") => {
            refresh_password_blocklist_once().await?;
            return Ok(());
        }
        Some(other) => {
            return Err(format!("unsupported command-line argument: {other}").into());
        }
        None => {}
    }

    install_rustls_provider()?;

    let config = Config::from_env()?;

    let blocklist = Arc::new(PasswordBlocklist::load_active(
        &config.password_blocklist_path,
    )?);

    info!(
        version = %blocklist.version(),
        "validated password blocklist snapshot loaded"
    );

    spawn_password_blocklist_refresh(
        config.password_blocklist_refresh_config(),
        blocklist.clone(),
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    let state = AppState::new(pool, blocklist, config.password_hash_concurrency);

    let router = apply_security(build_router(state), config.cors_allowed_origins.clone());

    let tls_config =
        RustlsConfig::from_pem_file(&config.tls_cert_path, &config.tls_key_path).await?;

    info!(
        address = %config.bind_addr,
        "backend listening with HTTPS"
    );

    axum_server::bind_rustls(config.bind_addr, tls_config)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn refresh_password_blocklist_once() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = PasswordBlocklistRefreshConfig::from_env()?;

    let version = PasswordBlocklist::refresh_to_active(
        &config.password_blocklist_path,
        &config.password_project_blocklist_path,
        config.password_blocklist_refresh_concurrency,
    )
    .await?;

    info!(
        version = %version,
        "password blocklist snapshot refreshed and activated"
    );

    Ok(())
}

fn spawn_password_blocklist_refresh(
    config: PasswordBlocklistRefreshConfig,
    blocklist: Arc<PasswordBlocklist>,
) {
    tokio::spawn(async move {
        loop {
            let delay = blocklist.refresh_delay(PASSWORD_BLOCKLIST_MAX_AGE);
            sleep(delay).await;

            if !blocklist.refresh_due(PASSWORD_BLOCKLIST_MAX_AGE) {
                continue;
            }

            match PasswordBlocklist::refresh_to_active(
                &config.password_blocklist_path,
                &config.password_project_blocklist_path,
                config.password_blocklist_refresh_concurrency,
            )
            .await
            {
                Ok(version) => {
                    if blocklist.reload_active().is_ok() {
                        info!(
                            version = %version,
                            "password blocklist snapshot refreshed and activated"
                        );
                    } else {
                        warn!(
                            "password blocklist refresh completed but the new validated \
                             snapshot could not be loaded; retaining the last known good snapshot"
                        );
                        sleep(REFRESH_RETRY_DELAY).await;
                    }
                }
                Err(_) => {
                    warn!(
                        "password blocklist refresh failed; retaining the last known good snapshot"
                    );
                    sleep(REFRESH_RETRY_DELAY).await;
                }
            }
        }
    });
}

fn install_rustls_provider() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match rustls::crypto::ring::default_provider().install_default() {
        Ok(()) | Err(_) => Ok(()),
    }
}
