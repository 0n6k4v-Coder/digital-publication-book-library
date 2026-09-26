use std::{env, net::SocketAddr, num::NonZeroUsize, path::PathBuf};

use axum::http::{HeaderValue, Uri};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub bind_addr: SocketAddr,
    pub password_blocklist_path: PathBuf,
    pub password_project_blocklist_path: PathBuf,
    pub password_hash_concurrency: NonZeroUsize,
    pub password_blocklist_refresh_concurrency: NonZeroUsize,
    pub tls_cert_path: PathBuf,
    pub tls_key_path: PathBuf,
    pub cors_allowed_origins: Vec<HeaderValue>,
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
        let password_project_blocklist_path =
            PathBuf::from(required("PASSWORD_PROJECT_BLOCKLIST_PATH")?);

        let password_hash_concurrency = optional("PASSWORD_HASH_CONCURRENCY")
            .unwrap_or_else(|| "4".to_owned())
            .parse::<NonZeroUsize>()
            .map_err(|_| ConfigError::Invalid("PASSWORD_HASH_CONCURRENCY"))?;

        let password_blocklist_refresh_concurrency =
            optional("PASSWORD_BLOCKLIST_REFRESH_CONCURRENCY")
                .unwrap_or_else(|| "32".to_owned())
                .parse::<NonZeroUsize>()
                .map_err(|_| ConfigError::Invalid("PASSWORD_BLOCKLIST_REFRESH_CONCURRENCY"))?;

        let tls_cert_path = PathBuf::from(required("TLS_CERT_PATH")?);
        let tls_key_path = PathBuf::from(required("TLS_KEY_PATH")?);

        let cors_allowed_origins = parse_allowed_origins(&required("CORS_ALLOWED_ORIGINS")?)?;

        Ok(Self {
            database_url,
            database_max_connections,
            bind_addr,
            password_blocklist_path,
            password_project_blocklist_path,
            password_hash_concurrency,
            password_blocklist_refresh_concurrency,
            tls_cert_path,
            tls_key_path,
            cors_allowed_origins,
        })
    }

    pub fn password_blocklist_refresh_config(&self) -> PasswordBlocklistRefreshConfig {
        PasswordBlocklistRefreshConfig {
            password_blocklist_path: self.password_blocklist_path.clone(),
            password_project_blocklist_path: self.password_project_blocklist_path.clone(),
            password_blocklist_refresh_concurrency: self.password_blocklist_refresh_concurrency,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PasswordBlocklistRefreshConfig {
    pub password_blocklist_path: PathBuf,
    pub password_project_blocklist_path: PathBuf,
    pub password_blocklist_refresh_concurrency: NonZeroUsize,
}

impl PasswordBlocklistRefreshConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let password_blocklist_path = PathBuf::from(required("PASSWORD_BLOCKLIST_PATH")?);
        let password_project_blocklist_path =
            PathBuf::from(required("PASSWORD_PROJECT_BLOCKLIST_PATH")?);

        let password_blocklist_refresh_concurrency =
            optional("PASSWORD_BLOCKLIST_REFRESH_CONCURRENCY")
                .unwrap_or_else(|| "32".to_owned())
                .parse::<NonZeroUsize>()
                .map_err(|_| ConfigError::Invalid("PASSWORD_BLOCKLIST_REFRESH_CONCURRENCY"))?;

        Ok(Self {
            password_blocklist_path,
            password_project_blocklist_path,
            password_blocklist_refresh_concurrency,
        })
    }
}

pub fn parse_allowed_origins(value: &str) -> Result<Vec<HeaderValue>, ConfigError> {
    let origins = value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(|origin| {
            let uri = origin
                .parse::<Uri>()
                .map_err(|_| ConfigError::Invalid("CORS_ALLOWED_ORIGINS"))?;

            match uri.scheme_str() {
                Some("http") | Some("https") => {}
                _ => return Err(ConfigError::Invalid("CORS_ALLOWED_ORIGINS")),
            }

            let authority = uri
                .authority()
                .ok_or(ConfigError::Invalid("CORS_ALLOWED_ORIGINS"))?;

            if authority.as_str().is_empty() || authority.as_str().contains('@') {
                return Err(ConfigError::Invalid("CORS_ALLOWED_ORIGINS"));
            }

            if uri.path() != "/" || uri.query().is_some() || origin.ends_with('/') {
                return Err(ConfigError::Invalid("CORS_ALLOWED_ORIGINS"));
            }

            origin
                .parse::<HeaderValue>()
                .map_err(|_| ConfigError::Invalid("CORS_ALLOWED_ORIGINS"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if origins.is_empty() {
        return Err(ConfigError::Invalid("CORS_ALLOWED_ORIGINS"));
    }

    Ok(origins)
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

#[cfg(test)]
mod tests {
    use super::parse_allowed_origins;

    #[test]
    fn parses_explicit_http_and_https_origins() {
        let origins = parse_allowed_origins("http://127.0.0.1:5173,https://admin.example.com")
            .expect("origins should parse");

        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0].to_str().unwrap(), "http://127.0.0.1:5173");
        assert_eq!(origins[1].to_str().unwrap(), "https://admin.example.com");
    }

    #[test]
    fn rejects_wildcard_origins() {
        assert!(parse_allowed_origins("*").is_err());
    }

    #[test]
    fn rejects_origins_with_paths() {
        assert!(parse_allowed_origins("https://admin.example.com/login").is_err());
    }

    #[test]
    fn rejects_origins_with_queries() {
        assert!(parse_allowed_origins("https://admin.example.com?mode=test").is_err());
    }

    #[test]
    fn rejects_origins_with_trailing_slashes() {
        assert!(parse_allowed_origins("https://admin.example.com/").is_err());
    }

    #[test]
    fn rejects_origins_with_user_info() {
        assert!(parse_allowed_origins("https://user@example.com").is_err());
    }

    #[test]
    fn rejects_empty_origin_configuration() {
        assert!(parse_allowed_origins("").is_err());
        assert!(parse_allowed_origins(" , ").is_err());
    }
}
