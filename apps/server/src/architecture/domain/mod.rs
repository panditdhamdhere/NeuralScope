use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A node in the service dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNode {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub service_type: ServiceType,
    pub metadata: serde_json::Value,
}

/// A directed edge between two services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEdge {
    pub id: Uuid,
    pub project_id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub protocol: Option<String>,
    pub avg_latency_ms: Option<f64>,
}

/// Classification of service types in the architecture graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    Gateway,
    Api,
    Auth,
    Database,
    Cache,
    Queue,
    External,
    Frontend,
}

impl ServiceType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gateway => "gateway",
            Self::Api => "api",
            Self::Auth => "auth",
            Self::Database => "database",
            Self::Cache => "cache",
            Self::Queue => "queue",
            Self::External => "external",
            Self::Frontend => "frontend",
        }
    }
}
