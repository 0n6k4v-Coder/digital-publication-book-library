use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::model::AuthenticatedPrincipal;

pub struct AuthenticationRepository {
    pool: PgPool,
}

impl AuthenticationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
