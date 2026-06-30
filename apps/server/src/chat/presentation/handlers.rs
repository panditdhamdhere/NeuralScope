use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::{ensure_project_member, ensure_project_writer};
use crate::auth::presentation::AuthUser;
use crate::chat::application::{list_conversations, list_messages, ChatService};
use crate::chat::domain::{
    ChatCompletionRequest, ChatCompletionResponse, ConversationSummary, MessageRecord,
};
use crate::AppError;

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub total: usize,
}

/// `POST /api/v1/projects/:project_id/chat/completions`
pub async fn chat_completion(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<ChatCompletionRequest>,
) -> Result<(StatusCode, Json<ChatCompletionResponse>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let provider = state
        .ai_provider
        .as_ref()
        .ok_or_else(|| {
            AppError::Internal(
                "AI provider not configured. Set GEMINI_API_KEY or configure AI_DEFAULT_PROVIDER."
                    .into(),
            )
        })?
        .clone();

    let response = ChatService::new(&state.db, provider)
        .complete(project_id, auth.user.id, body, state.vector.clone())
        .await?;

    Ok((StatusCode::OK, Json(response)))
}

/// `GET /api/v1/projects/:project_id/chat/conversations`
pub async fn get_conversations(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ListResponse<ConversationSummary>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let conversations = list_conversations(&state.db, project_id, auth.user.id).await?;
    let total = conversations.len();

    Ok(Json(ListResponse {
        data: conversations,
        meta: ListMeta { total },
    }))
}

/// `GET /api/v1/projects/:project_id/chat/conversations/:conversation_id/messages`
pub async fn get_messages(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((project_id, conversation_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ListResponse<MessageRecord>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let messages = list_messages(&state.db, project_id, auth.user.id, conversation_id).await?;
    let total = messages.len();

    Ok(Json(ListResponse {
        data: messages,
        meta: ListMeta { total },
    }))
}
