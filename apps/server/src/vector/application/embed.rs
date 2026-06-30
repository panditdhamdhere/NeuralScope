//! Embedding use case — thin wrapper over the embedding provider.

use std::sync::Arc;

use crate::vector::domain::{EmbeddingError, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};

/// Generates vector embeddings for text content.
pub struct EmbeddingService {
    provider: Arc<dyn EmbeddingProvider>,
}

impl EmbeddingService {
    #[must_use]
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { provider }
    }

    /// Embeds one or more text strings.
    pub async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError> {
        if request.texts.is_empty() {
            return Err(EmbeddingError::InvalidInput(
                "At least one text is required".into(),
            ));
        }

        self.provider.embed(request).await
    }

    #[must_use]
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::infrastructure::StubEmbeddingProvider;

    #[tokio::test]
    async fn embeds_text_with_stub_provider() {
        let service = EmbeddingService::new(Arc::new(StubEmbeddingProvider));
        let response = service
            .embed(EmbeddingRequest {
                texts: vec!["hello world".into()],
                task: None,
            })
            .await
            .expect("embed");

        assert_eq!(response.vectors.len(), 1);
        assert_eq!(response.vectors[0].len(), response.dimensions);
        assert_eq!(service.provider_name(), "stub");
    }
}
