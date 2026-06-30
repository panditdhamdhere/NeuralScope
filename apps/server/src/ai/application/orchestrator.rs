//! AI orchestrator — coordinates LLM calls and tool execution loops.

use std::sync::Arc;
use tracing::{debug, warn};

use crate::ai::application::tools::ToolRegistry;
use crate::ai::domain::{
    ChatMessage, CompletionRequest, CompletionResponse, LlmError, LlmProvider, OrchestratorResult,
};

const DEFAULT_MAX_TOOL_ROUNDS: u32 = 10;

/// Orchestrates multi-turn LLM conversations with tool calling.
pub struct AiOrchestrator {
    provider: Arc<dyn LlmProvider>,
    max_tool_rounds: u32,
}

impl AiOrchestrator {
    #[must_use]
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            max_tool_rounds: DEFAULT_MAX_TOOL_ROUNDS,
        }
    }

    #[must_use]
    pub fn with_max_tool_rounds(mut self, rounds: u32) -> Self {
        self.max_tool_rounds = rounds;
        self
    }

    /// Execute a conversation, running tool calls until the LLM produces a final answer.
    pub async fn run(
        &self,
        mut messages: Vec<ChatMessage>,
        tools: &ToolRegistry,
    ) -> Result<OrchestratorResult, LlmError> {
        let tool_definitions = tools.definitions();
        let mut tool_calls_made = 0u32;

        for round in 0..self.max_tool_rounds {
            debug!(round, "AI orchestrator iteration");

            let request = CompletionRequest {
                messages: messages.clone(),
                tools: if tool_definitions.is_empty() {
                    None
                } else {
                    Some(tool_definitions.clone())
                },
                model: None,
                temperature: Some(0.3),
                max_tokens: Some(4096),
            };

            let response = self.provider.complete(request).await?;

            if response.tool_calls.is_empty() {
                let content = response.content.unwrap_or_default();
                return Ok(OrchestratorResult {
                    content,
                    tool_calls_made,
                });
            }

            messages.push(assistant_message_with_tools(&response));

            for tool_call in &response.tool_calls {
                tool_calls_made += 1;
                debug!(tool = %tool_call.name, "Executing AI tool");

                let result = match tools.execute(&tool_call.name, tool_call.arguments.clone()).await {
                    Ok(content) => content,
                    Err(error) => {
                        warn!(tool = %tool_call.name, %error, "Tool execution failed");
                        format!("{{\"error\":\"{error}\"}}")
                    }
                };

                messages.push(ChatMessage::tool_result(
                    &tool_call.id,
                    &tool_call.name,
                    result,
                ));
            }

            if round == self.max_tool_rounds - 1 {
                warn!("Max tool rounds reached");
                break;
            }
        }

        // Final call without tools to force a text response
        let final_response = self
            .provider
            .complete(CompletionRequest {
                messages,
                tools: None,
                model: None,
                temperature: Some(0.3),
                max_tokens: Some(4096),
            })
            .await?;

        Ok(OrchestratorResult {
            content: final_response.content.unwrap_or_else(|| {
                "I gathered context but couldn't synthesize a final answer.".into()
            }),
            tool_calls_made,
        })
    }
}

fn assistant_message_with_tools(response: &CompletionResponse) -> ChatMessage {
    ChatMessage {
        role: crate::ai::domain::MessageRole::Assistant,
        content: response.content.clone().unwrap_or_default(),
        tool_call_id: None,
        tool_name: None,
        tool_calls: Some(response.tool_calls.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::ai::domain::{CompletionRequest, ToolCall};

    struct MockProvider {
        responses: std::sync::Mutex<Vec<CompletionResponse>>,
    }

    impl MockProvider {
        fn new(responses: Vec<CompletionResponse>) -> Arc<Self> {
            Arc::new(Self {
                responses: std::sync::Mutex::new(responses),
            })
        }
    }

    #[async_trait]
    impl LlmProvider for MockProvider {
        async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let mut guard = self.responses.lock().expect("lock");
            if guard.is_empty() {
                return Ok(CompletionResponse {
                    content: Some("Done".into()),
                    tool_calls: vec![],
                    finish_reason: Some("stop".into()),
                });
            }
            Ok(guard.remove(0))
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn supports_tools(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn orchestrator_executes_tools_then_returns_answer() {
        let provider = MockProvider::new(vec![
            CompletionResponse {
                content: None,
                tool_calls: vec![ToolCall {
                    id: "call_1".into(),
                    name: "search_logs".into(),
                    arguments: serde_json::json!({"search": "error"}),
                }],
                finish_reason: Some("tool_calls".into()),
            },
            CompletionResponse {
                content: Some("Found 3 errors in the API gateway.".into()),
                tool_calls: vec![],
                finish_reason: Some("stop".into()),
            },
        ]);

        let pool = sqlx::PgPool::connect_lazy("postgres://localhost/test").expect("pool");
        let project_id = uuid::Uuid::new_v4();
        let tools = ToolRegistry::for_project(project_id, pool);
        let orchestrator = AiOrchestrator::new(provider);

        let result = orchestrator
            .run(vec![ChatMessage::user("Find today's errors")], &tools)
            .await
            .expect("orchestrator");

        assert_eq!(result.tool_calls_made, 1);
        assert!(result.content.contains("errors"));
    }
}
