use std::{env, error::Error, fs, sync::Arc};

use digital_publication_backend::{
    shared::validation::{hash_password, normalize_email, PasswordBlocklist, PasswordPolicy},
    MIGRATOR,
};
use secrecy::SecretString;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

const ACCOUNT_ADMIN_ROLE: &str = "account_admin";
const DATABASE_MAX_CONNECTIONS: u32 = 10;

type DynError = Box<dyn Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let database_url = required_env("DATABASE_URL")?;
    let admin_email = required_env("DEV_ADMIN_EMAIL")?;
    let password_file = required_env("DEV_ADMIN_PASSWORD_FILE")?;

    let password = fs::read_to_string(password_file)?
        .trim_end_matches(&['\r', '\n'][..])
        .to_owned();

    let email = normalize_email(&admin_email)?;

    let password_secret = SecretString::from(password);

    let development_blocklist =
        PasswordBlocklist::from_hashes("development", std::iter::empty::<[u8; 20]>());

    let password_policy = PasswordPolicy::new(Arc::new(development_blocklist));

    password_policy.validate(&password_secret)?;

    let password_hash = hash_password(password_secret)?;

    let pool = PgPoolOptions::new()
        .max_connections(DATABASE_MAX_CONNECTIONS)
        .connect(&database_url)
        .await?;

    MIGRATOR.run(&pool).await?;

    seed_development_admin(&pool, &email.canonical, &email.normalized, &password_hash).await?;

    println!("Development admin ready: {}", email.canonical);

    Ok(())
}

async fn seed_development_admin(
    pool: &PgPool,
    email: &str,
    email_normalized: &str,
    password_hash: &str,
) -> Result<(), DynError> {
    let mut tx = pool.begin().await?;

    let existing_account_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT a.id
        FROM account AS a
        INNER JOIN account_credentials AS ac
            ON ac.account_id = a.id
        WHERE ac.email_normalized = $1
        FOR UPDATE OF a, ac
        "#,
    )
    .bind(email_normalized)
    .fetch_optional(&mut *tx)
    .await?;

    let account_id = match existing_account_id {
        Some(account_id) => {
            sqlx::query(
                r#"
                UPDATE account
                SET
                    status = 'active',
                    deleted_at = NULL,
                    deleted_by = NULL,
                    updated_at = CURRENT_TIMESTAMP,
                    updated_by = NULL
                WHERE id = $1
                "#,
            )
            .bind(account_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                UPDATE account_credentials
                SET
                    email = $2,
                    email_normalized = $3,
                    password_hash = $4,
                    updated_at = CURRENT_TIMESTAMP
                WHERE account_id = $1
                "#,
            )
            .bind(account_id)
            .bind(email)
            .bind(email_normalized)
            .bind(password_hash)
            .execute(&mut *tx)
            .await?;

            account_id
        }

        None => {
            let account_id = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO account (
                    status,
                    deleted_at
                )
                VALUES (
                    'active',
                    NULL
                )
                RETURNING id
                "#,
            )
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO account_credentials (
                    account_id,
                    email,
                    email_normalized,
                    password_hash
                )
                VALUES (
                    $1,
                    $2,
                    $3,
                    $4
                )
                "#,
            )
            .bind(account_id)
            .bind(email)
            .bind(email_normalized)
            .bind(password_hash)
            .execute(&mut *tx)
            .await?;

            account_id
        }
    };

    let role_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM authorization_role
        WHERE name = $1
        "#,
    )
    .bind(ACCOUNT_ADMIN_ROLE)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO authorization_account_role (
            account_id,
            role_id,
            created_by
        )
        VALUES (
            $1,
            $2,
            NULL
        )
        ON CONFLICT (
            account_id,
            role_id
        ) DO NOTHING
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

fn required_env(name: &'static str) -> Result<String, DynError> {
    env::var(name).map_err(|_| format!("{name} is required").into())
}
