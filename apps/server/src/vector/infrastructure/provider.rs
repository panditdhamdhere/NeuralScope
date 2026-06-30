use std::sync::Arc;

use crate::common::config::AppConfig;
use crate::vector::domain::EmbeddingProvider;
use crate::vector::infrastructure::{JinaEmbeddingProvider, StubEmbeddingProvider};

/// Creates the configured embedding provider.
#[must_use]
pub fn create_embedding_provider(config: &AppConfig) -> Arc<dyn EmbeddingProvider> {
    if let Some(api_key) = config.jina_api_key.clone() {
        Arc::new(JinaEmbeddingProvider::new(api_key))
    } else {
        Arc::new(StubEmbeddingProvider)
    }
}
