use axum::{routing::get, routing::post, Router};

use crate::api::state::AppState;

use super::handlers;

/// Security routes under `/api/v1/projects/:project_id/security`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/security/findings",
            get(handlers::list_findings),
        )
        .route(
            "/projects/{project_id}/security/scan",
            post(handlers::run_scan),
        )
        .route(
            "/projects/{project_id}/security/findings/sample",
            post(handlers::load_sample_findings),
        )
}
