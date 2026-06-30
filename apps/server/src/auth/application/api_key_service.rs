use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::application::crypto::{generate_api_key, hash_api_key};
use crate::auth::domain::ApiKey;
use crate::AppError;

/// Manages programmatic API key lifecycle.
pub struct ApiKeyService<'a> {
    pool: &'a PgPool,
}

impl<'a> ApiKeyService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new API key. The raw key is returned once and never stored.
    pub async fn create(&self, user_id: Uuid, name: &str) -> Result<(ApiKey, String), AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("API key name is required".into()));
        }

        let raw_key = generate_api_key();
        let key_hash = hash_api_key(&raw_key);
        let key_prefix = raw_key.chars().take(10).collect::<String>();
        let id = Uuid::new_v4();

        let api_key = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            INSERT INTO api_keys (id, user_id, name, key_prefix, key_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, key_prefix, last_used_at, expires_at, created_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(name)
        .bind(&key_prefix)
        .bind(&key_hash)
        .fetch_one(self.pool)
        .await?;

        Ok((api_key.into(), raw_key))
    }

    /// Lists API keys for a user (without raw key material).
    pub async fn list(&self, user_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, user_id, name, key_prefix, last_used_at, expires_at, created_at
            FROM api_keys
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Revokes an API key owned by the given user.
    pub async fn revoke(&self, user_id: Uuid, key_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM api_keys WHERE id = $1 AND user_id = $2")
            .bind(key_id)
            .bind(user_id)
            .execute(self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("API key not found".into()));
        }

        Ok(())
    }

    /// Validates an API key and returns the owning user ID.
    pub async fn validate(&self, raw_key: &str) -> Result<Uuid, AppError> {
        let key_hash = hash_api_key(raw_key);

        let user_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE key_hash = $1
              AND (expires_at IS NULL OR expires_at > NOW())
            RETURNING user_id
            "#,
        )
        .bind(key_hash)
        .fetch_optional(self.pool)
        .await?;

        user_id.ok_or_else(|| AppError::Unauthorized("Invalid API key".into()))
    }
}

#[derive(sqlx::FromRow)]
struct ApiKeyRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    key_prefix: String,
    last_used_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(row: ApiKeyRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            key_prefix: row.key_prefix,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}
