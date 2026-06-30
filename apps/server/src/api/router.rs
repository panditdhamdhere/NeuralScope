use axum::{
    middleware::from_fn,
    routing::get,
    Router,
};
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

use super::{cors, health, rate_limit, state::AppState};
use crate::architecture::presentation::architecture_routes;
use crate::auth::presentation::auth_routes;
use crate::chat::presentation::routes as chat_routes;
use crate::events::presentation::ws_handler;
use crate::incidents::presentation::incident_routes;
use crate::logs::presentation::log_routes;
use crate::metrics::presentation::metric_routes;
use crate::network::presentation::network_routes;
use crate::overview::presentation::overview_routes;
use crate::security::presentation::security_routes;
use crate::traces::presentation::trace_routes;

/// Builds the root Axum router with middleware and route modules.
#[must_use]
pub fn create_router(state: AppState) -> Router {
    let cors = cors::cors_layer(&state.config);
    let limiter = rate_limit::from_config(&state.config);

    Router::new()
        .route("/health", get(health::liveness))
        .route("/ready", get(health::readiness))
        .route("/ws", get(ws_handler))
        .nest("/api/v1", v1_routes(&state))
        .with_state(state)
        .layer(from_fn(move |request, next| {
            let limiter = limiter.clone();
            async move { rate_limit::rate_limit_middleware(limiter, request, next).await }
        }))
        .layer(CookieManagerLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

fn v1_routes(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/status", get(health::status))
        .merge(auth_routes(state))
        .merge(log_routes())
        .merge(metric_routes())
        .merge(trace_routes())
        .merge(chat_routes())
        .merge(network_routes())
        .merge(architecture_routes())
        .merge(security_routes())
        .merge(incident_routes())
        .merge(overview_routes())
}
