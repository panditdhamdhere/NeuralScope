use axum::{
    routing::get,
    Router,
};

use crate::api::state::AppState;

use super::handlers;

/// Trace routes under `/api/v1/projects/:project_id/traces`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/traces",
            get(handlers::list_traces).post(handlers::ingest_trace),
        )
        .route(
            "/projects/{project_id}/traces/{trace_id}",
            get(handlers::get_trace),
        )
}
