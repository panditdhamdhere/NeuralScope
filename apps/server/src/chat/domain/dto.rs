use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Request body for a chat completion.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionRequest {
    pub message: String,
    pub conversation_id: Option<Uuid>,
}

/// Response from a chat completion.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCompletionResponse {
    pub conversation_id: Uuid,
    pub content: String,
    pub tool_calls_made: u32,
    pub provider: String,
}

/// Summary of a conversation thread.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A persisted chat message.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRecord {
    pub id: Uuid,
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Value>,
    pub created_at: DateTime<Utc>,
}
