mod graph;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use graph::{
    GraphEdge, GraphEdgeData, GraphNode, GraphNodeData, GraphPosition, GraphResponse,
};

/// A network connection event between two endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEvent {
    pub id: Uuid,
    pub project_id: Uuid,
    pub source: NetworkNode,
    pub destination: NetworkNode,
    pub protocol: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency_ms: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

/// A node in the network graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct NetworkNode {
    pub name: String,
    pub node_type: NodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Payload for ingesting a network event.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestNetworkEventRequest {
    pub source_name: String,
    pub source_type: NodeType,
    pub destination_name: String,
    pub destination_type: NodeType,
    pub protocol: String,
    #[serde(default)]
    pub bytes_sent: u64,
    #[serde(default)]
    pub bytes_received: u64,
    pub latency_ms: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Query parameters for network event search.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkEventQuery {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    100
}

/// Classification of network node types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Service,
    External,
    Database,
    Cache,
    Queue,
    Browser,
    Unknown,
}

impl NodeType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::External => "external",
            Self::Database => "database",
            Self::Cache => "cache",
            Self::Queue => "queue",
            Self::Browser => "browser",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "service" => Self::Service,
            "external" => Self::External,
            "database" => Self::Database,
            "cache" => Self::Cache,
            "queue" => Self::Queue,
            "browser" => Self::Browser,
            _ => Self::Unknown,
        }
    }

    #[must_use]
    pub fn layout_tier(self) -> i32 {
        match self {
            Self::Browser => 0,
            Self::Service => 1,
            Self::Cache | Self::Queue => 2,
            Self::Database | Self::External => 3,
            Self::Unknown => 2,
        }
    }
}
