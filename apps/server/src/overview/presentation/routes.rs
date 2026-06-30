use axum::{routing::get, Router};

use crate::api::state::AppState;

use super::handlers;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/projects/{project_id}/overview",
        get(handlers::get_overview),
    )
}
