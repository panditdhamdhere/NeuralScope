//! Tool registry — defines and executes AI tools against observability data sources.

mod observability;
mod rag;

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ai::domain::ToolDefinition;
use crate::vector::application::VectorService;

pub use observability::{
    SearchArchitectureTool, SearchDeploymentsTool, SearchIncidentsTool, SearchLogsTool,
    SearchMetricsTool, SearchNetworkTool, SearchSecurityTool, SearchTracesTool,
};
pub use rag::{SearchCodebaseTool, SearchDocsTool};

/// JSON Schema for `limit` — Groq models often emit numeric limits as strings.
#[must_use]
pub fn limit_param_schema(default: i64, description: &str) -> Value {
    json!({
        "type": "string",
        "description": format!("{description} (default {default})")
    })
}

/// Parses a limit from JSON number or string (Groq-compatible).
#[must_use]
pub fn parse_limit(args: &Value, default: i64, min: i64, max: i64) -> i64 {
    let raw = args.get("limit");
    let parsed = match raw {
        None | Some(Value::Null) => default,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(default),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(default),
        _ => default,
    };
    parsed.clamp(min, max)
}

/// Parses an optional u32 limit for vector search.
#[must_use]
pub fn parse_limit_u32(args: &Value, default: u32, min: u32, max: u32) -> u32 {
    parse_limit(args, i64::from(default), i64::from(min), i64::from(max)) as u32
}

/// Errors during tool execution.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Unknown tool: {0}")]
    UnknownTool(String),
}

/// A tool that the AI can invoke to retrieve observability context.
#[async_trait]
pub trait AiTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, arguments: Value) -> Result<String, ToolError>;

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
}

/// Registry of all available AI tools for a project.
pub struct ToolRegistry {
    tools: Vec<Arc<dyn AiTool>>,
}

impl ToolRegistry {
    /// Creates a registry with all observability tools for the given project.
    #[must_use]
    pub fn for_project(project_id: Uuid, pool: PgPool, vector: Option<Arc<VectorService>>) -> Self {
        let mut registry = Self { tools: Vec::new() };
        registry.register(Arc::new(SearchLogsTool::new(pool.clone(), project_id)));
        registry.register(Arc::new(SearchMetricsTool::new(pool.clone(), project_id)));
        registry.register(Arc::new(SearchTracesTool::new(pool.clone(), project_id)));
        registry.register(Arc::new(SearchDeploymentsTool::new(
            pool.clone(),
            project_id,
        )));
        registry.register(Arc::new(SearchNetworkTool::new(pool.clone(), project_id)));
        registry.register(Arc::new(SearchArchitectureTool::new(
            pool.clone(),
            project_id,
        )));
        registry.register(Arc::new(SearchSecurityTool::new(pool.clone(), project_id)));
        registry.register(Arc::new(SearchIncidentsTool::new(pool.clone(), project_id)));

        if let Some(vector) = vector {
            registry.register(Arc::new(SearchCodebaseTool::new(
                vector.clone(),
                project_id,
            )));
            registry.register(Arc::new(SearchDocsTool::new(vector, project_id)));
        }

        registry
    }

    fn register(&mut self, tool: Arc<dyn AiTool>) {
        self.tools.push(tool);
    }

    /// Returns tool definitions for LLM function calling.
    #[must_use]
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.definition()).collect()
    }

    /// Executes a tool by name with the given arguments.
    pub async fn execute(&self, name: &str, arguments: Value) -> Result<String, ToolError> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| ToolError::UnknownTool(name.to_string()))?;

        tool.execute(arguments).await
    }

    #[must_use]
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_limit_accepts_string_and_number() {
        assert_eq!(parse_limit(&json!({ "limit": "10" }), 20, 1, 100), 10);
        assert_eq!(parse_limit(&json!({ "limit": 7 }), 20, 1, 100), 7);
        assert_eq!(parse_limit(&json!({}), 20, 1, 100), 20);
    }
}

/// Stub tool for features not yet implemented.
pub struct StubTool {
    name: &'static str,
    description: &'static str,
}

impl StubTool {
    #[must_use]
    pub const fn new(name: &'static str, description: &'static str) -> Self {
        Self { name, description }
    }
}

#[async_trait]
impl AiTool for StubTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, _arguments: Value) -> Result<String, ToolError> {
        Ok(format!(
            "{{\"status\":\"unavailable\",\"message\":\"Tool '{}' is not yet available in this release.\"}}",
            self.name
        ))
    }
}
