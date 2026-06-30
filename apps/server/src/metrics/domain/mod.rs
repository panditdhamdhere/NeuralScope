//! Metrics domain: data points, units, and query types.

mod unit;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use unit::MetricUnit;

/// A single metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricPoint {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    pub tags: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Payload for ingesting a metric sample.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestMetricRequest {
    pub name: String,
    pub value: f64,
    pub unit: MetricUnit,
    #[serde(default)]
    pub tags: serde_json::Value,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Query parameters for metric time-series retrieval.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricQuery {
    pub name: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    500
}
