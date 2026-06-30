use axum::{routing::get, routing::post, Router};

use crate::api::state::AppState;

use super::handlers;

/// Architecture routes under `/api/v1/projects/:project_id/architecture`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/architecture/graph",
            get(handlers::get_architecture_graph),
        )
        .route(
            "/projects/{project_id}/architecture/regenerate",
            post(handlers::regenerate_architecture_graph),
        )
}
