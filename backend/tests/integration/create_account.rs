use std::env;

use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

static TEST_DATABASE_LOCK: Mutex<()> = Mutex::const_new(());

async fn test_pool() -> Option<PgPool> {
    let database_url = env::var("TEST_DATABASE_URL").ok()?;

    Some(
        PgPool::connect(&database_url)
            .await
            .expect("connect to TEST_DATABASE_URL"),
    )
}

async fn cleanup_account(pool: &PgPool, account_id: Uuid) {
    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("cleanup account");
}

#[tokio::test]
#[ignore = "requires PostgreSQL configured through TEST_DATABASE_URL and applied migrations"]
async fn every_account_must_have_exactly_one_credential_set() {
    let _database_guard = TEST_DATABASE_LOCK.lock().await;
    let Some(pool) = test_pool().await else {
        return;
    };

    sqlx::migrate!().run(&pool).await.expect("run migrations");

    let mut create_transaction =
        pool.begin().await.expect("begin create transaction");

    let account_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO account (status, deleted_at) \
         VALUES ('active', NULL) RETURNING id",
    )
    .fetch_one(&mut *create_transaction)
    .await
    .expect("create account");

    sqlx::query(
        r#"
        INSERT INTO account_credentials (
            account_id,
            email,
            email_normalized,
            password_hash
        )
        VALUES ($1, $2, $2, $3)
        "#,
    )
    .bind(account_id)
    .bind(format!("credential-{account_id}@example.com"))
    .bind("$argon2id$v=19$placeholder$placeholder")
    .execute(&mut *create_transaction)
    .await
    .expect("create credentials");

    create_transaction
        .commit()
        .await
        .expect("commit complete account credential set");

    let mut delete_transaction =
        pool.begin().await.expect("begin delete transaction");

    sqlx::query(
        "DELETE FROM account_credentials WHERE account_id = $1",
    )
    .bind(account_id)
    .execute(&mut *delete_transaction)
    .await
    .expect("delete credentials inside transaction");

    assert!(delete_transaction.commit().await.is_err());

    let credential_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account_credentials \
         WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("count restored credentials");

    assert_eq!(credential_count, 1);

    cleanup_account(&pool, account_id).await;

    let mut insert_transaction =
        pool.begin().await.expect("begin insert transaction");

    let second_account_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO account (status, deleted_at) \
         VALUES ('active', NULL) RETURNING id",
    )
    .fetch_one(&mut *insert_transaction)
    .await
    .expect("insert account without credentials");

    assert!(insert_transaction.commit().await.is_err());

    let remaining = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account WHERE id = $1",
    )
    .bind(second_account_id)
    .fetch_one(&pool)
    .await
    .expect("check rolled back account");

    assert_eq!(remaining, 0);
}