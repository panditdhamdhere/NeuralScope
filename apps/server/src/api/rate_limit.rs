use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;

use crate::common::config::AppConfig;

/// In-memory per-IP sliding-window rate limiter.
#[derive(Clone)]
pub struct RateLimiter {
    hits: Arc<DashMap<String, Vec<Instant>>>,
    max_per_minute: u32,
}

impl RateLimiter {
    #[must_use]
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            hits: Arc::new(DashMap::new()),
            max_per_minute: max_per_minute.max(1),
        }
    }

    fn allow(&self, key: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let mut entry = self.hits.entry(key.to_string()).or_default();
        entry.retain(|instant| now.duration_since(*instant) < window);

        if entry.len() >= self.max_per_minute as usize {
            return false;
        }

        entry.push(now);
        true
    }
}

/// Builds a rate limiter from application configuration.
#[must_use]
pub fn from_config(config: &AppConfig) -> RateLimiter {
    RateLimiter::new(config.rate_limit_per_minute)
}

fn client_key(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
        })
        .unwrap_or("unknown")
        .to_string()
}

/// Axum middleware that enforces per-IP request limits.
pub async fn rate_limit_middleware(
    limiter: RateLimiter,
    request: Request,
    next: Next,
) -> Response {
    if !limiter.allow(&client_key(&request)) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            "Rate limit exceeded. Try again later.",
        )
            .into_response();
    }

    next.run(request).await
}
