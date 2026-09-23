use std::{env, net::SocketAddr, num::NonZeroUsize, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub bind_addr: SocketAddr,
    pub password_blocklist_path: PathBuf,
    pub password_hash_concurrency: NonZeroUsize,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = required("DATABASE_URL")?;

        let bind_addr = optional("BIND_ADDR")
            .unwrap_or_else(|| "127.0.0.1:3000".to_owned())
            .parse()
            .map_err(|_| ConfigError::Invalid("BIND_ADDR"))?;

        let database_max_connections = optional("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|| "10".to_owned())
            .parse::<u32>()
            .map_err(|_| ConfigError::Invalid("DATABASE_MAX_CONNECTIONS"))?;

        if database_max_connections == 0 {
            return Err(ConfigError::Invalid("DATABASE_MAX_CONNECTIONS"));
        }

        let password_blocklist_path = PathBuf::from(required("PASSWORD_BLOCKLIST_PATH")?);

        let password_hash_concurrency = optional("PASSWORD_HASH_CONCURRENCY")
            .unwrap_or_else(|| "4".to_owned())
            .parse::<NonZeroUsize>()
            .map_err(|_| ConfigError::Invalid("PASSWORD_HASH_CONCURRENCY"))?;

        Ok(Self {
            database_url,
            database_max_connections,
            bind_addr,
            password_blocklist_path,
            password_hash_concurrency,
        })
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::Missing(name))
}

fn optional(name: &str) -> Option<String> {
    env::var(name).ok()
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),

    #[error("invalid value for environment variable {0}")]
    Invalid(&'static str),
}
