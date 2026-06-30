use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::api::state::AppState;
use crate::common::config::Environment;

use super::handlers;

/// Auth and project routes under `/api/v1`.
pub fn routes(state: &AppState) -> Router<AppState> {
    let mut router = Router::new()
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/me", get(handlers::me))
        .route("/projects", get(handlers::list_projects).post(handlers::create_project))
        .route("/projects/{slug}", get(handlers::get_project))
        .route(
            "/api-keys",
            get(handlers::list_api_keys).post(handlers::create_api_key),
        )
        .route("/api-keys/{id}", delete(handlers::revoke_api_key));

    // Better Auth handles registration/login for the web app in production.
    if state.config.environment == Environment::Development {
        router = router
            .route("/auth/register", post(handlers::register))
            .route("/auth/login", post(handlers::login));
    }

    router
}
