use axum::{
    routing::{get, post},
    Router,
};

use crate::api::state::AppState;

use super::handlers;

/// Log routes under `/api/v1/projects/:project_id/logs`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/logs",
            get(handlers::search_logs).post(handlers::ingest_log),
        )
        .route(
            "/projects/{project_id}/logs/batch",
            post(handlers::ingest_logs_batch),
        )
}
