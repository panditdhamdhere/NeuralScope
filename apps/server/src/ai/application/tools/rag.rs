use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ai::application::tools::{limit_param_schema, parse_limit_u32, AiTool, ToolError};
use crate::vector::application::{SearchVectorRequest, VectorService};

/// Semantic search over indexed source code.
pub struct SearchCodebaseTool {
    vector: Arc<VectorService>,
    project_id: Uuid,
}

impl SearchCodebaseTool {
    #[must_use]
    pub fn new(vector: Arc<VectorService>, project_id: Uuid) -> Self {
        Self { vector, project_id }
    }
}

#[async_trait]
impl AiTool for SearchCodebaseTool {
    fn name(&self) -> &str {
        "search_codebase"
    }

    fn description(&self) -> &str {
        "Semantic search over indexed source code and repository snippets. Use for finding relevant code by meaning."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Natural language search query" },
                "limit": limit_param_schema(5, "Max results")
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("query is required".into()))?;

        let limit = Some(parse_limit_u32(&args, 5, 1, 20));

        let results = self
            .vector
            .search(
                self.project_id,
                SearchVectorRequest {
                    query: query.to_string(),
                    source_type: Some("code".into()),
                    limit,
                },
            )
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&results).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}

/// Semantic search over indexed documentation.
pub struct SearchDocsTool {
    vector: Arc<VectorService>,
    project_id: Uuid,
}

impl SearchDocsTool {
    #[must_use]
    pub fn new(vector: Arc<VectorService>, project_id: Uuid) -> Self {
        Self { vector, project_id }
    }
}

#[async_trait]
impl AiTool for SearchDocsTool {
    fn name(&self) -> &str {
        "search_docs"
    }

    fn description(&self) -> &str {
        "Semantic search over indexed documentation, runbooks, and README content."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Natural language search query" },
                "limit": limit_param_schema(5, "Max results")
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("query is required".into()))?;

        let limit = Some(parse_limit_u32(&args, 5, 1, 20));

        let results = self
            .vector
            .search(
                self.project_id,
                SearchVectorRequest {
                    query: query.to_string(),
                    source_type: Some("documentation".into()),
                    limit,
                },
            )
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        serde_json::to_string(&results).map_err(|e| ToolError::ExecutionFailed(e.to_string()))
    }
}
