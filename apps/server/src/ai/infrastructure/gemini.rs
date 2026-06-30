//! Google Gemini LLM provider implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::domain::{
    ChatMessage, CompletionRequest, CompletionResponse, LlmError, LlmProvider, MessageRole,
    ToolCall, ToolDefinition,
};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

/// Gemini API provider.
pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "gemini-2.0-flash".into(),
            client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let (system_instruction, contents) = convert_messages(&request.messages);

        let body = GeminiRequest {
            system_instruction,
            contents,
            tools: request.tools.as_ref().map(|tools| {
                vec![GeminiTools {
                    function_declarations: tools.iter().map(convert_tool).collect(),
                }]
            }),
            tool_config: request.tools.as_ref().map(|_| GeminiToolConfig {
                function_calling_config: FunctionCallingConfig {
                    mode: "AUTO".into(),
                },
            }),
            generation_config: Some(GenerationConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            }),
        };

        let url = format!(
            "{GEMINI_API_BASE}/models/{}:generateContent?key={}",
            request.model.as_deref().unwrap_or(&self.model),
            self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LlmError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(format!("Gemini API {status}: {text}")));
        }

        let gemini: GeminiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        parse_gemini_response(gemini)
    }

    fn name(&self) -> &str {
        "gemini"
    }

    fn supports_tools(&self) -> bool {
        true
    }
}

fn convert_messages(messages: &[ChatMessage]) -> (Option<GeminiContent>, Vec<GeminiContent>) {
    let mut system_parts = Vec::new();
    let mut contents = Vec::new();

    for message in messages {
        match message.role {
            MessageRole::System => {
                system_parts.push(GeminiPart::Text {
                    text: message.content.clone(),
                });
            }
            MessageRole::User => {
                contents.push(GeminiContent {
                    role: "user".into(),
                    parts: vec![GeminiPart::Text {
                        text: message.content.clone(),
                    }],
                });
            }
            MessageRole::Assistant => {
                let mut parts = Vec::new();
                if !message.content.is_empty() {
                    parts.push(GeminiPart::Text {
                        text: message.content.clone(),
                    });
                }
                if let Some(tool_calls) = &message.tool_calls {
                    for call in tool_calls {
                        parts.push(GeminiPart::FunctionCall {
                            function_call: GeminiFunctionCall {
                                name: call.name.clone(),
                                args: call.arguments.clone(),
                            },
                        });
                    }
                }
                if !parts.is_empty() {
                    contents.push(GeminiContent {
                        role: "model".into(),
                        parts,
                    });
                }
            }
            MessageRole::Tool => {
                contents.push(GeminiContent {
                    role: "user".into(),
                    parts: vec![GeminiPart::FunctionResponse {
                        function_response: GeminiFunctionResponse {
                            name: message.tool_name.clone().unwrap_or_default(),
                            response: json!({ "result": message.content }),
                        },
                    }],
                });
            }
        }
    }

    let system_instruction = if system_parts.is_empty() {
        None
    } else {
        Some(GeminiContent {
            role: "user".into(),
            parts: system_parts,
        })
    };

    (system_instruction, contents)
}

fn convert_tool(tool: &ToolDefinition) -> GeminiFunctionDeclaration {
    GeminiFunctionDeclaration {
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters: tool.parameters.clone(),
    }
}

fn parse_gemini_response(response: GeminiResponse) -> Result<CompletionResponse, LlmError> {
    let candidate = response
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| LlmError::InvalidResponse("No candidates in response".into()))?;

    let mut content_text = String::new();
    let mut tool_calls = Vec::new();

    for part in candidate.content.parts {
        match part {
            GeminiPart::Text { text } => content_text.push_str(&text),
            GeminiPart::FunctionCall { function_call } => {
                tool_calls.push(ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: function_call.name,
                    arguments: function_call.args,
                });
            }
            GeminiPart::FunctionResponse { .. } => {}
        }
    }

    Ok(CompletionResponse {
        content: if content_text.is_empty() {
            None
        } else {
            Some(content_text)
        },
        tool_calls,
        finish_reason: candidate.finish_reason,
    })
}

#[derive(Serialize)]
struct GeminiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiTools>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_config: Option<GeminiToolConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    FunctionCall { function_call: GeminiFunctionCall },
    FunctionResponse { function_response: GeminiFunctionResponse },
}

#[derive(Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: Value,
}

#[derive(Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: Value,
}

#[derive(Serialize)]
struct GeminiTools {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Serialize)]
struct GeminiToolConfig {
    function_calling_config: FunctionCallingConfig,
}

#[derive(Serialize)]
struct FunctionCallingConfig {
    mode: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(default)]
    finish_reason: Option<String>,
}
