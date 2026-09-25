pub mod app;
pub mod domains;
pub mod shared;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
