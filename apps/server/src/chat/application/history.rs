//! Read-only chat history queries.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::chat::domain::{ConversationSummary, MessageRecord};
use crate::AppError;

/// Lists conversations for a user within a project.
pub async fn list_conversations(
    db: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ConversationSummary>, AppError> {
    let rows = sqlx::query_as::<_, ConversationRow>(
        r"
        SELECT id, title, created_at, updated_at
        FROM chat_history
        WHERE project_id = $1 AND user_id = $2
        ORDER BY updated_at DESC
        LIMIT 50
        ",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Loads messages for a conversation after verifying ownership.
pub async fn list_messages(
    db: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<Vec<MessageRecord>, AppError> {
    let exists = sqlx::query_scalar::<_, bool>(
        r"
        SELECT EXISTS(
            SELECT 1 FROM chat_history
            WHERE id = $1 AND project_id = $2 AND user_id = $3
        )
        ",
    )
    .bind(conversation_id)
    .bind(project_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;

    if !exists {
        return Err(AppError::NotFound("Conversation not found".into()));
    }

    let rows = sqlx::query_as::<_, MessageRow>(
        r"
        SELECT id, role, content, tool_calls, created_at
        FROM chat_messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        ",
    )
    .bind(conversation_id)
    .fetch_all(db)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

#[derive(sqlx::FromRow)]
struct ConversationRow {
    id: Uuid,
    title: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ConversationRow> for ConversationSummary {
    fn from(row: ConversationRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    role: String,
    content: String,
    tool_calls: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

impl From<MessageRow> for MessageRecord {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            role: row.role,
            content: row.content,
            tool_calls: row.tool_calls,
            created_at: row.created_at,
        }
    }
}
