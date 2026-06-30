//! Traces domain: traces, spans, status, and ingest types.

mod status;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use status::TraceStatus;

/// A distributed trace consisting of multiple spans.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    pub id: Uuid,
    pub project_id: Uuid,
    pub trace_id: String,
    pub root_service: String,
    pub duration_ms: f64,
    pub span_count: u32,
    pub status: TraceStatus,
    pub started_at: DateTime<Utc>,
}

/// Individual span within a trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub id: Uuid,
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service: String,
    pub operation: String,
    pub duration_ms: f64,
    pub status: TraceStatus,
    pub attributes: serde_json::Value,
    pub started_at: DateTime<Utc>,
}

/// Full trace with all spans for detail view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceDetail {
    #[serde(flatten)]
    pub trace: Trace,
    pub spans: Vec<Span>,
}

/// Payload for ingesting a complete trace with spans.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestTraceRequest {
    pub trace_id: String,
    pub spans: Vec<IngestSpanRequest>,
}

/// Payload for a single span within a trace ingest request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestSpanRequest {
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service: String,
    pub operation: String,
    pub duration_ms: f64,
    #[serde(default)]
    pub status: TraceStatus,
    #[serde(default)]
    pub attributes: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
}

/// Query parameters for listing traces.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceQuery {
    pub service: Option<String>,
    pub status: Option<TraceStatus>,
    pub since: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}
