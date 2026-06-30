use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ai::application::tools::{limit_param_schema, parse_limit, AiTool, ToolError};
use crate::logs::application::LogService;
use crate::logs::domain::{LogLevel, LogSearchQuery};
use crate::metrics::application::MetricService;
use crate::metrics::domain::MetricQuery;
use crate::traces::application::TraceService;
use crate::traces::domain::TraceQuery;

/// Searches structured logs for a project.
pub struct SearchLogsTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchLogsTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchLogsTool {
    fn name(&self) -> &str {
        "search_logs"
    }

    fn description(&self) -> &str {
        "Search application logs by text, level, service, or trace ID. Returns recent matching log entries."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "search": { "type": "string", "description": "Text to search in log messages" },
                "level": { "type": "string", "enum": ["trace","debug","info","warn","error","fatal"], "description": "Filter by log level" },
                "service": { "type": "string", "description": "Filter by service name" },
                "trace_id": { "type": "string", "description": "Filter by trace ID" },
                "limit": limit_param_schema(20, "Max results")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let level = args
            .get("level")
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<LogLevel>())
            .transpose()
            .map_err(|e| ToolError::InvalidArguments(e.to_string()))?;

        let query = LogSearchQuery {
            level,
            service: args
                .get("service")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            search: args
                .get("search")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            trace_id: args
                .get("trace_id")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            limit: parse_limit(&args, 20, 1, 100),
            offset: 0,
        };

        let events = LogService::new(&self.pool, &crate::events::application::EventBus::new())
            .search(self.project_id, query)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&events).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Queries time-series metrics for a project.
pub struct SearchMetricsTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchMetricsTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchMetricsTool {
    fn name(&self) -> &str {
        "search_metrics"
    }

    fn description(&self) -> &str {
        "Query metric time-series data by name. Returns recent data points for CPU, memory, latency, and custom metrics."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Metric name (e.g. cpu.usage, http.latency)" },
                "limit": limit_param_schema(50, "Max data points")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let events = crate::events::application::EventBus::new();
        let service = MetricService::new(&self.pool, &events);

        if name.is_none() {
            let names = service
                .list_names(self.project_id)
                .await
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
            return serde_json::to_string(&json!({ "available_metrics": names }))
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()));
        }

        let query = MetricQuery {
            name,
            since: None,
            until: None,
            limit: parse_limit(&args, 50, 1, 500),
        };

        let points = service
            .query(self.project_id, query)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&points).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Searches distributed traces for a project.
pub struct SearchTracesTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchTracesTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchTracesTool {
    fn name(&self) -> &str {
        "search_traces"
    }

    fn description(&self) -> &str {
        "Search distributed traces by service or status. Use trace_id from results to investigate specific traces."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": { "type": "string", "description": "Filter by root service name" },
                "status": { "type": "string", "enum": ["ok", "error"], "description": "Filter by trace status" },
                "trace_id": { "type": "string", "description": "Get full trace detail by ID" },
                "limit": limit_param_schema(10, "Max results")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let events = crate::events::application::EventBus::new();
        let service = TraceService::new(&self.pool, &events);

        if let Some(trace_id) = args.get("trace_id").and_then(|v| v.as_str()) {
            let detail = service
                .get_by_trace_id(self.project_id, trace_id)
                .await
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
            return serde_json::to_string(&detail)
                .map_err(|e| ToolError::ExecutionFailed(e.to_string()));
        }

        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.parse())
            .transpose()
            .map_err(|e: crate::AppError| ToolError::InvalidArguments(e.to_string()))?;

        let query = TraceQuery {
            service: args
                .get("service")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            status,
            since: None,
            limit: parse_limit(&args, 10, 1, 100),
            offset: 0,
        };

        let traces = service
            .list(self.project_id, query)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&traces).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Searches deployment history for a project.
pub struct SearchDeploymentsTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchDeploymentsTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchDeploymentsTool {
    fn name(&self) -> &str {
        "search_git"
    }

    fn description(&self) -> &str {
        "Search Git commit history and deployment records. Returns recent commits and deployments with correlation metadata."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "search": { "type": "string", "description": "Search commit messages or authors" },
                "environment": { "type": "string", "description": "Filter deployments by environment" },
                "branch": { "type": "string", "description": "Filter commits by branch" },
                "limit": limit_param_schema(10, "Max results per section")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let limit = parse_limit(&args, 10, 1, 50);
        let environment = args.get("environment").and_then(|v| v.as_str());
        let branch = args.get("branch").and_then(|v| v.as_str());
        let search = args.get("search").and_then(|v| v.as_str());

        let commits = sqlx::query_as::<_, CommitRow>(
            r#"
            SELECT sha, author, message, branch, committed_at
            FROM git_commits
            WHERE project_id = $1
              AND ($2::text IS NULL OR branch = $2)
              AND ($3::text IS NULL OR message ILIKE '%' || $3 || '%' OR author ILIKE '%' || $3 || '%')
            ORDER BY committed_at DESC
            LIMIT $4
            "#,
        )
        .bind(self.project_id)
        .bind(branch)
        .bind(search)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let deployments = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT commit_sha, environment, deployed_by, deployed_at
            FROM deployments
            WHERE project_id = $1
              AND ($2::text IS NULL OR environment = $2)
            ORDER BY deployed_at DESC
            LIMIT $3
            "#,
        )
        .bind(self.project_id)
        .bind(environment)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let payload = json!({
            "commits": commits,
            "deployments": deployments,
        });

        serde_json::to_string(&payload).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct CommitRow {
    sha: String,
    author: String,
    message: String,
    branch: String,
    committed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct DeploymentRow {
    commit_sha: String,
    environment: String,
    deployed_by: Option<String>,
    deployed_at: chrono::DateTime<chrono::Utc>,
}

/// Searches network connection events for a project.
pub struct SearchNetworkTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchNetworkTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchNetworkTool {
    fn name(&self) -> &str {
        "search_network"
    }

    fn description(&self) -> &str {
        "Search network connection events between services and external endpoints."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "source": { "type": "string", "description": "Filter by source service name" },
                "destination": { "type": "string", "description": "Filter by destination service name" },
                "limit": limit_param_schema(20, "Max results")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let limit = parse_limit(&args, 20, 1, 100);

        let rows = sqlx::query_as::<_, NetworkRow>(
            r#"
            SELECT source_name, destination_name, protocol, bytes_sent, bytes_received, latency_ms, timestamp
            FROM network_events
            WHERE project_id = $1
              AND ($2::text IS NULL OR source_name ILIKE $2)
              AND ($3::text IS NULL OR destination_name ILIKE $3)
            ORDER BY timestamp DESC
            LIMIT $4
            "#,
        )
        .bind(self.project_id)
        .bind(
            args.get("source")
                .and_then(|v| v.as_str())
                .map(|s| format!("%{s}%")),
        )
        .bind(
            args.get("destination")
                .and_then(|v| v.as_str())
                .map(|s| format!("%{s}%")),
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&rows).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct NetworkRow {
    source_name: String,
    destination_name: String,
    protocol: String,
    bytes_sent: i64,
    bytes_received: i64,
    latency_ms: Option<f64>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Searches the persisted architecture graph for a project.
pub struct SearchArchitectureTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchArchitectureTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchArchitectureTool {
    fn name(&self) -> &str {
        "search_architecture"
    }

    fn description(&self) -> &str {
        "Search the service architecture graph — nodes, dependencies, and connection counts."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "service": { "type": "string", "description": "Filter by service name" }
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let service_filter = args
            .get("service")
            .and_then(|v| v.as_str())
            .map(|s| format!("%{s}%"));

        let nodes = sqlx::query_as::<_, ArchitectureNodeRow>(
            r#"
            SELECT name, service_type
            FROM architecture_nodes
            WHERE project_id = $1
              AND ($2::text IS NULL OR name ILIKE $2)
            ORDER BY name ASC
            LIMIT 50
            "#,
        )
        .bind(self.project_id)
        .bind(service_filter.clone())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let edges = sqlx::query_as::<_, ArchitectureEdgeRow>(
            r#"
            SELECT source.name AS source_name, target.name AS target_name,
                   e.protocol, e.avg_latency_ms, e.request_count
            FROM architecture_edges e
            JOIN architecture_nodes source ON source.id = e.source_node_id
            JOIN architecture_nodes target ON target.id = e.target_node_id
            WHERE e.project_id = $1
              AND ($2::text IS NULL OR source.name ILIKE $2 OR target.name ILIKE $2)
            ORDER BY e.request_count DESC
            LIMIT 50
            "#,
        )
        .bind(self.project_id)
        .bind(service_filter)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&json!({ "nodes": nodes, "edges": edges }))
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct ArchitectureNodeRow {
    name: String,
    service_type: String,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct ArchitectureEdgeRow {
    source_name: String,
    target_name: String,
    protocol: Option<String>,
    avg_latency_ms: Option<f64>,
    request_count: i64,
}

/// Searches security findings for a project.
pub struct SearchSecurityTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchSecurityTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchSecurityTool {
    fn name(&self) -> &str {
        "search_security"
    }

    fn description(&self) -> &str {
        "Search security findings including secrets, weak configs, and exposed ports."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "severity": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
                "limit": limit_param_schema(20, "Max results")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let limit = parse_limit(&args, 20, 1, 100);
        let severity = args
            .get("severity")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let rows = sqlx::query_as::<_, SecurityFindingRow>(
            r#"
            SELECT finding_type, severity, title, description, resource, detected_at
            FROM security_findings
            WHERE project_id = $1
              AND ($2::text IS NULL OR severity = $2)
            ORDER BY detected_at DESC
            LIMIT $3
            "#,
        )
        .bind(self.project_id)
        .bind(severity)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&rows).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Searches incident reports for a project.
pub struct SearchIncidentsTool {
    pool: PgPool,
    project_id: Uuid,
}

impl SearchIncidentsTool {
    #[must_use]
    pub fn new(pool: PgPool, project_id: Uuid) -> Self {
        Self { pool, project_id }
    }
}

#[async_trait]
impl AiTool for SearchIncidentsTool {
    fn name(&self) -> &str {
        "search_incidents"
    }

    fn description(&self) -> &str {
        "Search incident reports with root cause analysis, timelines, and suggested fixes."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "status": { "type": "string", "enum": ["open", "investigating", "resolved"] },
                "limit": limit_param_schema(10, "Max results")
            }
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let limit = parse_limit(&args, 10, 1, 50);
        let status = args
            .get("status")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let rows = sqlx::query_as::<_, IncidentToolRow>(
            r#"
            SELECT title, severity, status, root_cause, affected_services, created_at
            FROM incidents
            WHERE project_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(self.project_id)
        .bind(status)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&rows).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct SecurityFindingRow {
    finding_type: String,
    severity: String,
    title: String,
    description: String,
    resource: Option<String>,
    detected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct IncidentToolRow {
    title: String,
    severity: String,
    status: String,
    root_cause: Option<String>,
    affected_services: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}
