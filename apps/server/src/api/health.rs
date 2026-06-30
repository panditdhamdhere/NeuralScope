use ::redis::aio::ConnectionManager;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use super::state::AppState;
use crate::db::{pool, redis_client};

/// Liveness probe — process is running.
#[derive(Serialize)]
pub struct LivenessResponse {
    pub status: &'static str,
    pub version: &'static str,
}

/// Readiness probe — dependencies are reachable.
#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: ReadinessStatus,
    pub version: &'static str,
    pub checks: DependencyChecks,
}

#[derive(Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessStatus {
    Ready,
    NotReady,
}

#[derive(Serialize)]
pub struct DependencyChecks {
    pub database: CheckResult,
    pub redis: CheckResult,
}

#[derive(Serialize)]
pub struct CheckResult {
    pub status: CheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Up,
    Down,
}

/// Detailed platform status for the dashboard and operators.
#[derive(Serialize)]
pub struct StatusResponse {
    pub status: ReadinessStatus,
    pub version: &'static str,
    pub environment: String,
    pub uptime_seconds: u64,
    pub checks: DependencyChecks,
}

/// `GET /health` — Kubernetes liveness probe.
pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        status: "ok",
        version: crate::VERSION,
    })
}

/// `GET /ready` — Kubernetes readiness probe.
pub async fn readiness(State(state): State<AppState>) -> Response {
    let checks = run_dependency_checks(&state.db, &state.redis).await;
    readiness_response(checks)
}

/// `GET /api/v1/status` — detailed status with uptime and environment.
pub async fn status(State(state): State<AppState>) -> Response {
    let checks = run_dependency_checks(&state.db, &state.redis).await;
    let all_up = is_healthy(&checks);

    let body = StatusResponse {
        status: if all_up {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::NotReady
        },
        version: crate::VERSION,
        environment: state.config.environment.to_string(),
        uptime_seconds: state.uptime_seconds(),
        checks,
    };

    let status = if all_up {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(body)).into_response()
}

fn is_healthy(checks: &DependencyChecks) -> bool {
    checks.database.status == CheckStatus::Up && checks.redis.status == CheckStatus::Up
}

fn readiness_response(checks: DependencyChecks) -> Response {
    let all_up = is_healthy(&checks);

    let body = ReadinessResponse {
        status: if all_up {
            ReadinessStatus::Ready
        } else {
            ReadinessStatus::NotReady
        },
        version: crate::VERSION,
        checks,
    };

    let status = if all_up {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(body)).into_response()
}

async fn run_dependency_checks(db: &PgPool, redis: &ConnectionManager) -> DependencyChecks {
    let (database, redis_check) =
        tokio::join!(check_database(db), check_redis(redis));

    DependencyChecks {
        database,
        redis: redis_check,
    }
}

async fn check_database(pool: &PgPool) -> CheckResult {
    let start = std::time::Instant::now();
    match pool::verify(pool).await {
        Ok(()) => CheckResult {
            status: CheckStatus::Up,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => CheckResult {
            status: CheckStatus::Down,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

async fn check_redis(redis: &ConnectionManager) -> CheckResult {
    let start = std::time::Instant::now();
    let mut conn = redis.clone();
    match redis_client::ping(&mut conn).await {
        Ok(()) => CheckResult {
            status: CheckStatus::Up,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => CheckResult {
            status: CheckStatus::Down,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn liveness_returns_ok_with_version() {
        let Json(body) = liveness().await;
        assert_eq!(body.status, "ok");
        assert_eq!(body.version, crate::VERSION);
    }
}
