use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::Semaphore;

use crate::shared::validation::{PasswordBlocklist, PasswordPolicy};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub password_policy: PasswordPolicy,
    pub password_hash_semaphore: Arc<Semaphore>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        blocklist: Arc<PasswordBlocklist>,
        password_hash_concurrency: std::num::NonZeroUsize,
    ) -> Self {
        Self {
            pool,
            password_policy: PasswordPolicy::new(blocklist),
            password_hash_semaphore: Arc::new(Semaphore::new(password_hash_concurrency.get())),
        }
    }
}
