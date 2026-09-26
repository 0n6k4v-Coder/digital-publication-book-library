use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::{
    AuthenticatedPrincipal, AUTHENTICATION_SESSION_EXPIRES_IN, REFRESH_TOKEN_POLICY_EXPIRES_IN,
};

const LOGIN_EMAIL_FAILURE_LIMIT: i64 = 10;
const LOGIN_SOURCE_IP_FAILURE_LIMIT: i64 = 50;

pub struct AuthenticationRepository {
    pool: PgPool,
}

impl AuthenticationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_account_authentication_data(
        &self,
        email_normalized: &str,
    ) -> Result<Option<AccountAuthenticationData>, sqlx::Error> {
        sqlx::query_as::<_, AccountAuthenticationData>(
            r#"
            SELECT
                a.id AS account_id,
                ac.password_hash,
                a.status = 'active' AS is_active,
                a.deleted_at IS NOT NULL AS is_deleted
            FROM account AS a
            INNER JOIN account_credentials AS ac
                ON ac.account_id = a.id
            WHERE ac.email_normalized = $1
            "#,
        )
        .bind(email_normalized)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn reserve_login_attempt(
        &self,
        email_key: &str,
        source_ip_key: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let mut lock_keys = [format!("email:{email_key}"), format!("ip:{source_ip_key}")];
        lock_keys.sort_unstable();

        for lock_key in lock_keys {
            let lock_key = advisory_lock_key(&lock_key);

            sqlx::query("SELECT pg_advisory_xact_lock($1)")
                .bind(lock_key)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query(
            "DELETE FROM authentication_login_attempt \
             WHERE attempted_at <= CURRENT_TIMESTAMP - INTERVAL '15 minutes'",
        )
        .execute(&mut *tx)
        .await?;

        let counts = sqlx::query_as::<_, LoginRateCounts>(
            r#"
            SELECT
                COUNT(*) FILTER (
                    WHERE email_key = $1
                      AND email_counted = TRUE
                ) AS email_count,
                COUNT(*) FILTER (
                    WHERE source_ip_key = $2
                      AND source_ip_counted = TRUE
                ) AS source_ip_count
            FROM authentication_login_attempt
            WHERE attempted_at > CURRENT_TIMESTAMP - INTERVAL '15 minutes'
              AND (email_key = $1 OR source_ip_key = $2)
            "#,
        )
        .bind(email_key)
        .bind(source_ip_key)
        .fetch_one(&mut *tx)
        .await?;

        if counts.email_count >= LOGIN_EMAIL_FAILURE_LIMIT
            || counts.source_ip_count >= LOGIN_SOURCE_IP_FAILURE_LIMIT
        {
            tx.rollback().await?;
            return Ok(None);
        }

        let attempt_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO authentication_login_attempt (
                email_key,
                source_ip_key,
                failed,
                email_counted,
                source_ip_counted
            )
            VALUES ($1, $2, FALSE, TRUE, TRUE)
            RETURNING id
            "#,
        )
        .bind(email_key)
        .bind(source_ip_key)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Some(attempt_id))
    }

    pub async fn mark_login_attempt_failed(&self, attempt_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE authentication_login_attempt
            SET
                failed = TRUE,
                attempted_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
        )
        .bind(attempt_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn issue_tokens(
        &self,
        attempt_id: Uuid,
        email_key: &str,
        account_id: Uuid,
        access_token_hash: &str,
        refresh_token_hash: &str,
    ) -> Result<Option<OffsetDateTime>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let account_state = sqlx::query_as::<_, AccountState>(
            r#"
            SELECT
                status = 'active' AS is_active,
                deleted_at IS NOT NULL AS is_deleted
            FROM account
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(account_state) = account_state else {
            mark_attempt_failed_in_transaction(&mut tx, attempt_id).await?;

            tx.commit().await?;
            return Ok(None);
        };

        if !account_state.is_active || account_state.is_deleted {
            mark_attempt_failed_in_transaction(&mut tx, attempt_id).await?;

            tx.commit().await?;
            return Ok(None);
        }

        sqlx::query(
            r#"
            UPDATE authentication_login_attempt
            SET email_counted = FALSE
            WHERE email_key = $1
              AND failed = TRUE
            "#,
        )
        .bind(email_key)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM authentication_login_attempt WHERE id = $1")
            .bind(attempt_id)
            .execute(&mut *tx)
            .await?;

        let session = sqlx::query_as::<_, AuthenticationSessionRow>(
            r#"
            INSERT INTO authentication_session (
                account_id,
                expires_at,
                last_authenticated_at
            )
            VALUES (
                $1,
                CURRENT_TIMESTAMP
                    + make_interval(secs => $2::double precision),
                CURRENT_TIMESTAMP
            )
            RETURNING id, expires_at
            "#,
        )
        .bind(account_id)
        .bind(AUTHENTICATION_SESSION_EXPIRES_IN as f64)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO authentication_access_token (
                session_id,
                token_hash,
                expires_at
            )
            VALUES (
                $1,
                $2,
                CURRENT_TIMESTAMP + INTERVAL '1 hour'
            )
            "#,
        )
        .bind(session.id)
        .bind(access_token_hash)
        .execute(&mut *tx)
        .await?;

        let refresh_expires_at = sqlx::query_scalar::<_, OffsetDateTime>(
            r#"
            INSERT INTO authentication_refresh_token (
                session_id,
                token_hash,
                expires_at
            )
            VALUES (
                $1,
                $2,
                LEAST(
                    $3,
                    CURRENT_TIMESTAMP
                        + make_interval(secs => $4::double precision)
                )
            )
            RETURNING expires_at
            "#,
        )
        .bind(session.id)
        .bind(refresh_token_hash)
        .bind(session.expires_at)
        .bind(REFRESH_TOKEN_POLICY_EXPIRES_IN as f64)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Some(refresh_expires_at))
    }

    pub async fn refresh_tokens(
        &self,
        presented_refresh_token_hash: &str,
        access_token_hash: &str,
        replacement_refresh_token_hash: &str,
    ) -> Result<Option<OffsetDateTime>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let session_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE authentication_refresh_token AS r
            SET used_at = CURRENT_TIMESTAMP
            FROM authentication_session AS s
            INNER JOIN account AS a
                ON a.id = s.account_id
            WHERE r.token_hash = $1
              AND r.session_id = s.id
              AND r.used_at IS NULL
              AND r.revoked_at IS NULL
              AND r.expires_at > CURRENT_TIMESTAMP
              AND s.revoked_at IS NULL
              AND s.expires_at > CURRENT_TIMESTAMP
              AND a.status = 'active'
              AND a.deleted_at IS NULL
            RETURNING r.session_id
            "#,
        )
        .bind(presented_refresh_token_hash)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(session_id) = session_id else {
            tx.rollback().await?;
            return Ok(None);
        };

        let session_expires_at = sqlx::query_scalar::<_, OffsetDateTime>(
            r#"
            SELECT expires_at
            FROM authentication_session
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO authentication_access_token (
                session_id,
                token_hash,
                expires_at
            )
            VALUES (
                $1,
                $2,
                CURRENT_TIMESTAMP + INTERVAL '1 hour'
            )
            "#,
        )
        .bind(session_id)
        .bind(access_token_hash)
        .execute(&mut *tx)
        .await?;

        let replacement_refresh_expires_at = sqlx::query_scalar::<_, OffsetDateTime>(
            r#"
            INSERT INTO authentication_refresh_token (
                session_id,
                token_hash,
                expires_at
            )
            VALUES (
                $1,
                $2,
                LEAST(
                    $3,
                    CURRENT_TIMESTAMP
                        + make_interval(secs => $4::double precision)
                )
            )
            RETURNING expires_at
            "#,
        )
        .bind(session_id)
        .bind(replacement_refresh_token_hash)
        .bind(session_expires_at)
        .bind(REFRESH_TOKEN_POLICY_EXPIRES_IN as f64)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Some(replacement_refresh_expires_at))
    }

    pub async fn revoke_session_by_refresh_token(
        &self,
        refresh_token_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE authentication_session AS s
            SET
                revoked_at = COALESCE(
                    s.revoked_at,
                    CURRENT_TIMESTAMP
                ),
                revocation_reason = COALESCE(
                    s.revocation_reason,
                    'logout'
                )
            FROM authentication_refresh_token AS r
            WHERE r.session_id = s.id
              AND r.token_hash = $1
            "#,
        )
        .bind(refresh_token_hash)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn revoke_session(
        &self,
        account_id: Uuid,
        session_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let revoked = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE authentication_session
            SET
                revoked_at = COALESCE(
                    revoked_at,
                    CURRENT_TIMESTAMP
                ),
                revocation_reason = COALESCE(
                    revocation_reason,
                    'logout'
                )
            WHERE id = $1
              AND account_id = $2
            RETURNING id
            "#,
        )
        .bind(session_id)
        .bind(account_id)
        .fetch_optional(&mut *tx)
        .await?
        .is_some();

        tx.commit().await?;

        Ok(revoked)
    }

    pub async fn validate_access_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<AuthenticatedPrincipal>, sqlx::Error> {
        sqlx::query_as::<_, AccessTokenRow>(
            r#"
            SELECT
                s.account_id,
                s.id AS session_id,
                s.last_authenticated_at AS authenticated_at
            FROM authentication_access_token AS t
            INNER JOIN authentication_session AS s
                ON s.id = t.session_id
            INNER JOIN account AS a
                ON a.id = s.account_id
            WHERE t.token_hash = $1
              AND t.expires_at > CURRENT_TIMESTAMP
              AND s.revoked_at IS NULL
              AND s.expires_at > CURRENT_TIMESTAMP
              AND a.status = 'active'
              AND a.deleted_at IS NULL
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(AuthenticatedPrincipal::from))
    }
}

async fn mark_attempt_failed_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    attempt_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE authentication_login_attempt
        SET
            failed = TRUE,
            attempted_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#,
    )
    .bind(attempt_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub struct AccountAuthenticationData {
    pub account_id: Uuid,
    pub password_hash: String,
    pub is_active: bool,
    pub is_deleted: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct AccountState {
    is_active: bool,
    is_deleted: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct AuthenticationSessionRow {
    id: Uuid,
    expires_at: OffsetDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct LoginRateCounts {
    email_count: i64,
    source_ip_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct AccessTokenRow {
    account_id: Uuid,
    session_id: Uuid,
    authenticated_at: OffsetDateTime,
}

impl From<AccessTokenRow> for AuthenticatedPrincipal {
    fn from(row: AccessTokenRow) -> Self {
        Self {
            account_id: row.account_id,
            session_id: row.session_id,
            authenticated_at: row.authenticated_at,
        }
    }
}

fn advisory_lock_key(value: &str) -> i64 {
    let digest = Sha256::digest(value.as_bytes());

    i64::from_be_bytes(
        digest[..8]
            .try_into()
            .expect("SHA-256 digest is at least 8 bytes"),
    )
}
