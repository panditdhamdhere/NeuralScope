use axum::{
    routing::{get, post},
    Router,
};

use crate::api::state::AppState;

use super::handlers;

/// Vector routes under `/api/v1/projects/:project_id/vectors`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{project_id}/vectors/status",
            get(handlers::vector_status),
        )
        .route(
            "/projects/{project_id}/vectors/index",
            post(handlers::index_vector),
        )
        .route(
            "/projects/{project_id}/vectors/search",
            post(handlers::search_vectors),
        )
}
