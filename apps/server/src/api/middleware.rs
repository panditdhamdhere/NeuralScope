//! HTTP middleware: authentication, rate limiting, request tracing.

pub use crate::auth::presentation::extractors::{
    AuthRejection, AuthUser, API_KEY_HEADER, BETTER_AUTH_SESSION_COOKIE, SESSION_COOKIE,
};
