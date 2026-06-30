//! Jina embedding provider (stub until Qdrant indexing is wired in M7+).

use async_trait::async_trait;

use crate::vector::domain::{EmbeddingError, EmbeddingProvider, EmbeddingRequest, EmbeddingResponse};

/// Jina AI embeddings via the v1 API.
pub struct JinaEmbeddingProvider {
    api_key: String,
    model: String,
    dimensions: usize,
    client: reqwest::Client,
}

impl JinaEmbeddingProvider {
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: "jina-embeddings-v3".into(),
            dimensions: 1024,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for JinaEmbeddingProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError> {
        let body = serde_json::json!({
            "model": self.model,
            "input": request.texts,
            "task": request.task.unwrap_or_else(|| "retrieval.passage".into()),
        });

        let response = self
            .client
            .post("https://api.jina.ai/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbeddingError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::RequestFailed(format!(
                "Jina API {status}: {text}"
            )));
        }

        let payload: JinaResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::InvalidResponse(e.to_string()))?;

        Ok(EmbeddingResponse {
            vectors: payload
                .data
                .into_iter()
                .map(|item| item.embedding)
                .collect(),
            model: self.model.clone(),
            dimensions: self.dimensions,
        })
    }

    fn name(&self) -> &str {
        "jina"
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Deterministic stub used when no embedding API key is configured.
pub struct StubEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for StubEmbeddingProvider {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError> {
        let dimensions = self.dimensions();
        let vectors = request
            .texts
            .iter()
            .map(|text| {
                let mut vector = vec![0.0_f32; dimensions];
                for (index, byte) in text.bytes().enumerate().take(dimensions) {
                    vector[index] = f32::from(byte) / 255.0;
                }
                vector
            })
            .collect();

        Ok(EmbeddingResponse {
            vectors,
            model: "stub".into(),
            dimensions,
        })
    }

    fn name(&self) -> &str {
        "stub"
    }

    fn dimensions(&self) -> usize {
        384
    }
}

#[derive(serde::Deserialize)]
struct JinaResponse {
    data: Vec<JinaEmbedding>,
}

#[derive(serde::Deserialize)]
struct JinaEmbedding {
    embedding: Vec<f32>,
}
