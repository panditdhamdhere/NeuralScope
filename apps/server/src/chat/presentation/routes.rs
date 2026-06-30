use axum::{routing::get, routing::post, Router};

use crate::api::state::AppState;

use super::handlers;

/// Chat routes under `/api/v1/projects/:project_id/chat`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/chat/completions",
            post(handlers::chat_completion),
        )
        .route(
            "/projects/{project_id}/chat/conversations",
            get(handlers::get_conversations),
        )
        .route(
            "/projects/{project_id}/chat/conversations/{conversation_id}/messages",
            get(handlers::get_messages),
        )
}
