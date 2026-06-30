use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Real-time event types broadcast to connected clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Event {
    #[serde(rename = "log.new")]
    LogNew {
        entry_id: Uuid,
        level: String,
        message: String,
    },
    #[serde(rename = "metric.sample")]
    MetricSample { name: String, value: f64 },
    #[serde(rename = "trace.complete")]
    TraceComplete { trace_id: String, duration_ms: f64 },
    #[serde(rename = "network.connection")]
    NetworkConnection { source: String, destination: String },
    #[serde(rename = "incident.created")]
    IncidentCreated { incident_id: Uuid, severity: String },
    #[serde(rename = "security.finding")]
    SecurityFinding {
        finding_id: Uuid,
        severity: String,
        title: String,
    },
    DeploymentDetected {
        commit_sha: String,
        environment: String,
    },
}

/// Envelope wrapping an event with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub id: Uuid,
    pub project_id: Uuid,
    pub event: Event,
    pub timestamp: DateTime<Utc>,
}
