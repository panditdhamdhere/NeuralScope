use std::sync::Arc;

use crate::ai::domain::{LlmError, LlmProvider};
use crate::ai::infrastructure::{GeminiProvider, OpenAiCompatibleProvider};
use crate::common::config::AppConfig;

/// Creates the configured LLM provider from application config.
pub fn create_llm_provider(config: &AppConfig) -> Result<Arc<dyn LlmProvider>, LlmError> {
    match config.ai_default_provider.as_str() {
        "gemini" => {
            let api_key = config
                .gemini_api_key
                .clone()
                .ok_or_else(|| LlmError::NotConfigured("GEMINI_API_KEY not set".into()))?;
            Ok(Arc::new(GeminiProvider::new(api_key)))
        }
        "groq" => {
            let api_key = config
                .groq_api_key
                .clone()
                .ok_or_else(|| LlmError::NotConfigured("GROQ_API_KEY not set".into()))?;
            Ok(Arc::new(OpenAiCompatibleProvider::groq(
                api_key,
                &config.groq_model,
            )))
        }
        "openrouter" => {
            let api_key = config
                .openrouter_api_key
                .clone()
                .ok_or_else(|| LlmError::NotConfigured("OPENROUTER_API_KEY not set".into()))?;
            Ok(Arc::new(OpenAiCompatibleProvider::openrouter(api_key)))
        }
        "ollama" => Ok(Arc::new(OpenAiCompatibleProvider::ollama(
            config.ollama_base_url.clone(),
        ))),
        other => Err(LlmError::NotConfigured(format!(
            "Unknown AI provider: {other}"
        ))),
    }
}
