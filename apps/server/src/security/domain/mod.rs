use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A security finding from automated scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFinding {
    pub id: Uuid,
    pub project_id: Uuid,
    pub finding_type: FindingType,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub resource: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Payload for triggering a security scan.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    pub content: Option<String>,
    #[serde(default = "default_scan_logs")]
    pub scan_logs: bool,
}

fn default_scan_logs() -> bool {
    true
}

/// Query parameters for listing findings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindingQuery {
    pub severity: Option<Severity>,
    pub finding_type: Option<FindingType>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// Result of a security scan run.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub findings: Vec<SecurityFinding>,
    pub scanned_sources: u32,
}

/// Types of security findings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    Secret,
    ApiKey,
    ExposedPort,
    WeakConfig,
    DependencyVulnerability,
    DockerIssue,
}

impl FindingType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Secret => "secret",
            Self::ApiKey => "api_key",
            Self::ExposedPort => "exposed_port",
            Self::WeakConfig => "weak_config",
            Self::DependencyVulnerability => "dependency_vulnerability",
            Self::DockerIssue => "docker_issue",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "secret" => Self::Secret,
            "api_key" => Self::ApiKey,
            "exposed_port" => Self::ExposedPort,
            "weak_config" => Self::WeakConfig,
            "dependency_vulnerability" => Self::DependencyVulnerability,
            "docker_issue" => Self::DockerIssue,
            _ => Self::Secret,
        }
    }
}

/// Severity classification for security findings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
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

/// Raw finding detected by a scanner before persistence.
#[derive(Debug, Clone)]
pub struct DetectedFinding {
    pub finding_type: FindingType,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub resource: Option<String>,
}
