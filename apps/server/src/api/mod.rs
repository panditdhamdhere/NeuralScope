//! API gateway — router composition, middleware, and health endpoints.

pub mod cors;
pub mod health;
pub mod middleware;
pub mod rate_limit;
pub mod routes;
pub mod router;
pub mod state;

pub use router::create_router;
pub use state::AppState;
