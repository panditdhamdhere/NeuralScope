use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::role::ProjectRole;

/// Authenticated user entity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub email_verified: bool,
    pub image: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Active session bound to a user.
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// API key metadata (raw key is only returned on creation).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Project workspace entity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_id: Uuid,
    pub role: ProjectRole,
    pub created_at: DateTime<Utc>,
}
