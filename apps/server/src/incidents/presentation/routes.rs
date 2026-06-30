use axum::{routing::get, routing::post, Router};

use crate::api::state::AppState;

use super::handlers;

/// Incident routes under `/api/v1/projects/:project_id/incidents`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/incidents",
            get(handlers::list_incidents),
        )
        .route(
            "/projects/{project_id}/incidents/generate",
            post(handlers::generate_incident),
        )
        .route(
            "/projects/{project_id}/incidents/{incident_id}",
            get(handlers::get_incident).patch(handlers::update_incident),
        )
}
