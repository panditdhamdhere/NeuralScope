//! Project overview aggregates for the dashboard.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Aggregated project health snapshot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOverview {
    pub error_logs_24h: i64,
    pub total_logs_24h: i64,
    pub traces_24h: i64,
    pub failed_traces_24h: i64,
    pub open_incidents: i64,
    pub critical_findings: i64,
    pub conversations: i64,
    pub recent_logs: Vec<RecentLog>,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub server_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentLog {
    pub id: Uuid,
    pub level: String,
    pub message: String,
    pub service: Option<String>,
    pub timestamp: DateTime<Utc>,
}
