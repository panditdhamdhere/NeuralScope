//! Chat completion use case — orchestrates AI responses and persists history.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::ai::application::{observability_system_prompt, AiOrchestrator, ToolRegistry};
use crate::ai::domain::{ChatMessage, LlmError, LlmProvider};
use crate::chat::domain::{ChatCompletionRequest, ChatCompletionResponse};
use crate::AppError;

/// Handles project-scoped AI chat completions.
pub struct ChatService<'a> {
    db: &'a PgPool,
    provider: Arc<dyn LlmProvider>,
}

impl<'a> ChatService<'a> {
    #[must_use]
    pub fn new(db: &'a PgPool, provider: Arc<dyn LlmProvider>) -> Self {
        Self { db, provider }
    }

    /// Runs the AI orchestrator and stores the conversation exchange.
    pub async fn complete(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError> {
        if request.message.trim().is_empty() {
            return Err(AppError::Validation("Message cannot be empty".into()));
        }

        let conversation_id = match request.conversation_id {
            Some(id) => {
                self.ensure_conversation(id, project_id, user_id).await?;
                id
            }
            None => self.create_conversation(project_id, user_id, &request.message).await?,
        };

        let history = self.load_history(conversation_id).await?;
        let tools = ToolRegistry::for_project(project_id, self.db.clone());
        let orchestrator = AiOrchestrator::new(self.provider.clone());

        let mut messages = vec![ChatMessage::system(observability_system_prompt(
            project_id,
        ))];
        messages.extend(history);
        messages.push(ChatMessage::user(&request.message));

        let result = orchestrator
            .run(messages, &tools)
            .await
            .map_err(map_llm_error)?;

        self.save_message(conversation_id, "user", &request.message, None)
            .await?;
        self.save_message(
            conversation_id,
            "assistant",
            &result.content,
            Some(serde_json::json!({ "tool_calls_made": result.tool_calls_made })),
        )
        .await?;

        self.touch_conversation(conversation_id).await?;

        Ok(ChatCompletionResponse {
            conversation_id,
            content: result.content,
            tool_calls_made: result.tool_calls_made,
            provider: self.provider.name().to_string(),
        })
    }

    async fn create_conversation(
        &self,
        project_id: Uuid,
        user_id: Uuid,
        first_message: &str,
    ) -> Result<Uuid, AppError> {
        let title = truncate_title(first_message);

        let row = sqlx::query_scalar::<_, Uuid>(
            r"
            INSERT INTO chat_history (project_id, user_id, title)
            VALUES ($1, $2, $3)
            RETURNING id
            ",
        )
        .bind(project_id)
        .bind(user_id)
        .bind(title)
        .fetch_one(self.db)
        .await?;

        Ok(row)
    }

    async fn ensure_conversation(
        &self,
        conversation_id: Uuid,
        project_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AppError> {
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
        .fetch_one(self.db)
        .await?;

        if exists {
            Ok(())
        } else {
            Err(AppError::NotFound("Conversation not found".into()))
        }
    }

    async fn load_history(&self, conversation_id: Uuid) -> Result<Vec<ChatMessage>, AppError> {
        let rows = sqlx::query_as::<_, HistoryRow>(
            r"
            SELECT role, content
            FROM chat_messages
            WHERE conversation_id = $1
            ORDER BY created_at ASC
            ",
        )
        .bind(conversation_id)
        .fetch_all(self.db)
        .await?;

        Ok(rows.into_iter().map(|row| row.into_chat_message()).collect())
    }

    async fn save_message(
        &self,
        conversation_id: Uuid,
        role: &str,
        content: &str,
        tool_calls: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            INSERT INTO chat_messages (conversation_id, role, content, tool_calls)
            VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(tool_calls)
        .execute(self.db)
        .await?;

        Ok(())
    }

    async fn touch_conversation(&self, conversation_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE chat_history
            SET updated_at = NOW()
            WHERE id = $1
            ",
        )
        .bind(conversation_id)
        .execute(self.db)
        .await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct HistoryRow {
    role: String,
    content: String,
}

impl HistoryRow {
    fn into_chat_message(self) -> ChatMessage {
        let role = match self.role.as_str() {
            "assistant" => crate::ai::domain::MessageRole::Assistant,
            "system" => crate::ai::domain::MessageRole::System,
            "tool" => crate::ai::domain::MessageRole::Tool,
            _ => crate::ai::domain::MessageRole::User,
        };

        ChatMessage {
            role,
            content: self.content,
            tool_call_id: None,
            tool_name: None,
            tool_calls: None,
        }
    }
}

fn truncate_title(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.len() <= 80 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..77])
    }
}

fn map_llm_error(error: LlmError) -> AppError {
    match error {
        LlmError::NotConfigured(message) => AppError::Internal(format!("AI not configured: {message}")),
        LlmError::RateLimited => AppError::External("AI provider rate limit exceeded".into()),
        LlmError::RequestFailed(message) | LlmError::InvalidResponse(message) => {
            AppError::AiProvider(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_title_shortens_long_messages() {
        let long = "a".repeat(100);
        let title = truncate_title(&long);
        assert!(title.len() <= 80);
        assert!(title.ends_with("..."));
    }
}
