use axum::{routing::get, routing::post, Router};

use crate::api::state::AppState;

use super::handlers;

/// Network routes under `/api/v1/projects/:project_id/network`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/network/events",
            get(handlers::query_network_events).post(handlers::ingest_network_event),
        )
        .route(
            "/projects/{project_id}/network/events/batch",
            post(handlers::ingest_network_events_batch),
        )
        .route(
            "/projects/{project_id}/network/graph",
            get(handlers::get_network_graph),
        )
}
