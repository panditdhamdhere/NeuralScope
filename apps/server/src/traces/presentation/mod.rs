//! Traces presentation: ingest and query HTTP handlers.

pub mod handlers;
pub mod routes;

pub use routes::routes as trace_routes;
