//! Auth presentation: HTTP handlers, extractors, and route definitions.

pub mod extractors;
pub mod handlers;
pub mod routes;

pub use extractors::AuthUser;
pub use routes::routes as auth_routes;
