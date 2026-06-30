mod dto;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::ai::domain::{ChatMessage, MessageRole};

pub use dto::{
    ChatCompletionRequest, ChatCompletionResponse, ConversationSummary, MessageRecord,
};

/// A chat conversation thread.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single message in a conversation.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Message {
    #[must_use]
    pub fn to_chat_message(&self) -> ChatMessage {
        ChatMessage {
            role: self.role.clone(),
            content: self.content.clone(),
            tool_call_id: None,
            tool_name: None,
            tool_calls: None,
        }
    }
}
