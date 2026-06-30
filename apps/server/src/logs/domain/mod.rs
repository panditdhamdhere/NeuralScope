//! Log domain: log entry entity, levels, and query types.

mod level;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use level::LogLevel;

/// A structured log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: serde_json::Value,
}

/// Payload for ingesting a single log entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestLogRequest {
    pub level: LogLevel,
    pub message: String,
    pub service: Option<String>,
    pub trace_id: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Query parameters for searching logs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSearchQuery {
    pub level: Option<LogLevel>,
    pub service: Option<String>,
    pub search: Option<String>,
    pub trace_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
}
