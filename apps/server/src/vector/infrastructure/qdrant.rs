//! Qdrant vector store client (HTTP REST API).

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::vector::domain::VectorSourceType;

#[derive(Debug, Error)]
pub enum QdrantError {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// HTTP client for Qdrant collections and point search.
#[derive(Clone)]
pub struct QdrantClient {
    base_url: String,
    client: Client,
}

impl QdrantClient {
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Verifies Qdrant is reachable.
    ///
    /// # Errors
    ///
    /// Returns an error when the health endpoint is unreachable or non-success.
    pub async fn health_check(&self) -> Result<(), QdrantError> {
        let response = self
            .client
            .get(format!("{}/healthz", self.base_url))
            .send()
            .await
            .map_err(|e| QdrantError::RequestFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(QdrantError::RequestFailed(format!(
                "Qdrant health check returned {}",
                response.status()
            )))
        }
    }

    pub fn collection_name(project_id: Uuid, source_type: &VectorSourceType) -> String {
        let suffix = match source_type {
            VectorSourceType::Code => "code",
            VectorSourceType::Documentation => "docs",
            VectorSourceType::Log => "logs",
            VectorSourceType::Trace => "traces",
            VectorSourceType::Incident => "incidents",
        };
        format!("p_{project_id}_{suffix}").replace('-', "_")
    }

    /// Creates the collection if it does not already exist.
    ///
    /// # Errors
    ///
    /// Returns an error when Qdrant rejects the create request.
    pub async fn ensure_collection(
        &self,
        name: &str,
        dimensions: usize,
    ) -> Result<(), QdrantError> {
        let url = format!("{}/collections/{name}", self.base_url);
        let check = self.client.get(&url).send().await;

        if let Ok(response) = check {
            if response.status().is_success() {
                return Ok(());
            }
        }

        let body = serde_json::json!({
            "vectors": {
                "size": dimensions,
                "distance": "Cosine"
            }
        });

        let response = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::RequestFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(QdrantError::RequestFailed(format!(
                "Create collection {name}: {status} {text}"
            )))
        }
    }

    /// Upserts a single vector point with metadata payload.
    ///
    /// # Errors
    ///
    /// Returns an error when the upsert request fails.
    pub async fn upsert_point(
        &self,
        collection: &str,
        point_id: Uuid,
        vector: Vec<f32>,
        payload: PointPayload,
    ) -> Result<(), QdrantError> {
        let url = format!("{}/collections/{collection}/points", self.base_url);
        let body = UpsertRequest {
            points: vec![UpsertPoint {
                id: point_id.to_string(),
                vector,
                payload,
            }],
        };

        let response = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::RequestFailed(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(QdrantError::RequestFailed(format!(
                "Upsert point: {status} {text}"
            )))
        }
    }

    /// Performs cosine similarity search in a collection.
    ///
    /// # Errors
    ///
    /// Returns an error when the search request fails or the response is invalid.
    pub async fn search(
        &self,
        collection: &str,
        vector: Vec<f32>,
        limit: u32,
    ) -> Result<Vec<SearchHit>, QdrantError> {
        let url = format!("{}/collections/{collection}/points/search", self.base_url);
        let body = SearchRequest {
            vector,
            limit,
            with_payload: true,
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(QdrantError::RequestFailed(format!(
                "Search: {status} {text}"
            )));
        }

        let payload: SearchResponse = response
            .json()
            .await
            .map_err(|e| QdrantError::InvalidResponse(e.to_string()))?;

        Ok(payload
            .result
            .into_iter()
            .map(|point| SearchHit {
                score: point.score,
                content: point.payload.content.unwrap_or_default(),
                source_id: point.payload.source_id.unwrap_or_default(),
                source_type: point.payload.source_type.unwrap_or_default(),
            })
            .collect())
    }
}

#[derive(Serialize)]
struct UpsertRequest {
    points: Vec<UpsertPoint>,
}

#[derive(Serialize)]
struct UpsertPoint {
    id: String,
    vector: Vec<f32>,
    payload: PointPayload,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PointPayload {
    pub content: String,
    pub source_id: String,
    pub source_type: String,
    pub project_id: String,
}

#[derive(Serialize)]
struct SearchRequest {
    vector: Vec<f32>,
    limit: u32,
    with_payload: bool,
}

#[derive(Deserialize)]
struct SearchResponse {
    result: Vec<ScoredPoint>,
}

#[derive(Deserialize)]
struct ScoredPoint {
    score: f32,
    payload: PointPayloadPartial,
}

#[derive(Deserialize)]
struct PointPayloadPartial {
    content: Option<String>,
    source_id: Option<String>,
    source_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub score: f32,
    pub content: String,
    pub source_id: String,
    pub source_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collection_name_sanitizes_uuid() {
        let id = Uuid::parse_str("f484961a-8d90-4dc2-9f4e-249081102b66").expect("uuid");
        let name = QdrantClient::collection_name(id, &VectorSourceType::Code);
        assert!(name.starts_with("p_f484961a_8d90_4dc2_9f4e_249081102b66_code"));
        assert!(!name.contains('-'));
    }
}
