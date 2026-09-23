use sqlx::PgPool;
use uuid::Uuid;

pub struct AuthorizationRepository {
    pool: PgPool,
}

impl AuthorizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn has_permission(
        &self,
        account_id: Uuid,
        permission: &str,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM authorization_account_role AS ar
                INNER JOIN authorization_role_permission AS rp
                    ON rp.role_id = ar.role_id
                INNER JOIN authorization_permission AS p
                    ON p.id = rp.permission_id
                WHERE ar.account_id = $1
                  AND p.name = $2
            )
            "#,
        )
        .bind(account_id)
        .bind(permission)
        .fetch_one(&self.pool)
        .await
    }
}
