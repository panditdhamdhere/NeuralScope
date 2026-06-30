use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An incident report with timeline and remediation suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub root_cause: Option<String>,
    pub timeline: Vec<TimelineEntry>,
    pub affected_services: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// A single event in an incident timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub entry_type: TimelineEntryType,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineEntryType {
    Log,
    Trace,
    Finding,
    Metric,
    System,
}

/// Incident severity levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl IncidentSeverity {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "critical" => Self::Critical,
            "high" => Self::High,
            "medium" => Self::Medium,
            _ => Self::Low,
        }
    }
}

/// Incident lifecycle status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IncidentStatus {
    Open,
    Investigating,
    Resolved,
}

impl IncidentStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Investigating => "investigating",
            Self::Resolved => "resolved",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "investigating" => Self::Investigating,
            "resolved" => Self::Resolved,
            _ => Self::Open,
        }
    }
}

/// Request to generate a new incident report.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateIncidentRequest {
    pub title: Option<String>,
    #[serde(default = "default_use_ai")]
    pub use_ai: bool,
}

fn default_use_ai() -> bool {
    true
}

/// Request to update incident status.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIncidentRequest {
    pub status: Option<IncidentStatus>,
}

/// Query parameters for listing incidents.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentQuery {
    pub status: Option<IncidentStatus>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}
