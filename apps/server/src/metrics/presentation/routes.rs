use axum::{
    routing::{get, post},
    Router,
};

use crate::api::state::AppState;

use super::handlers;

/// Metric routes under `/api/v1/projects/:project_id/metrics`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/metrics",
            get(handlers::query_metrics).post(handlers::ingest_metric),
        )
        .route(
            "/projects/{project_id}/metrics/batch",
            post(handlers::ingest_metrics_batch),
        )
        .route(
            "/projects/{project_id}/metrics/names",
            get(handlers::list_metric_names),
        )
}
