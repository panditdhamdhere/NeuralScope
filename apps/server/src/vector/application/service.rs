//! Vector indexing and semantic search use cases.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::vector::application::EmbeddingService;
use crate::vector::domain::{EmbeddingRequest, VectorSourceType};
use crate::vector::infrastructure::qdrant::{PointPayload, QdrantClient, QdrantError, SearchHit};
use crate::AppError;

/// Indexes and searches project-scoped vector embeddings via Qdrant.
pub struct VectorService {
    embeddings: EmbeddingService,
    qdrant: QdrantClient,
    pool: PgPool,
}

impl VectorService {
    #[must_use]
    pub fn new(embeddings: EmbeddingService, qdrant: QdrantClient, pool: PgPool) -> Self {
        Self {
            embeddings,
            qdrant,
            pool,
        }
    }

    #[must_use]
    pub fn from_parts(
        provider: Arc<dyn crate::vector::domain::EmbeddingProvider>,
        qdrant_url: &str,
        pool: PgPool,
    ) -> Self {
        Self::new(
            EmbeddingService::new(provider),
            QdrantClient::new(qdrant_url),
            pool,
        )
    }

    /// Checks Qdrant connectivity.
    ///
    /// # Errors
    ///
    /// Propagates Qdrant client errors.
    pub async fn health_check(&self) -> Result<(), QdrantError> {
        self.qdrant.health_check().await
    }

    #[must_use]
    pub fn embedding_provider(&self) -> &str {
        self.embeddings.provider_name()
    }

    /// Indexes a text document into Qdrant and records metadata in Postgres.
    ///
    /// # Errors
    ///
    /// Returns validation, embedding, Qdrant, or database errors.
    pub async fn index(
        &self,
        project_id: Uuid,
        request: IndexVectorRequest,
    ) -> Result<IndexedVector, AppError> {
        let source_type = parse_source_type(&request.source_type)?;
        if request.content.trim().is_empty() {
            return Err(AppError::Validation("content cannot be empty".into()));
        }

        let collection = QdrantClient::collection_name(project_id, &source_type);
        let embed_response = self
            .embeddings
            .embed(EmbeddingRequest {
                texts: vec![request.content.clone()],
                task: Some("retrieval.passage".into()),
            })
            .await
            .map_err(|e| AppError::External(e.to_string()))?;

        let vector =
            embed_response.vectors.into_iter().next().ok_or_else(|| {
                AppError::External("Embedding provider returned no vectors".into())
            })?;

        self.qdrant
            .ensure_collection(&collection, embed_response.dimensions)
            .await
            .map_err(|e| AppError::External(e.to_string()))?;

        let id = Uuid::new_v4();
        self.qdrant
            .upsert_point(
                &collection,
                id,
                vector,
                PointPayload {
                    content: request.content.clone(),
                    source_id: request.source_id.clone(),
                    source_type: request.source_type.clone(),
                    project_id: project_id.to_string(),
                },
            )
            .await
            .map_err(|e| AppError::External(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO vectors (id, project_id, content, source_type, source_id, collection)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(project_id)
        .bind(&request.content)
        .bind(&request.source_type)
        .bind(&request.source_id)
        .bind(&collection)
        .execute(&self.pool)
        .await?;

        Ok(IndexedVector {
            id,
            collection,
            source_type: request.source_type,
            source_id: request.source_id,
        })
    }

    /// Performs semantic search over indexed vectors.
    ///
    /// # Errors
    ///
    /// Returns validation, embedding, or Qdrant errors.
    pub async fn search(
        &self,
        project_id: Uuid,
        request: SearchVectorRequest,
    ) -> Result<Vec<VectorSearchResult>, AppError> {
        if request.query.trim().is_empty() {
            return Err(AppError::Validation("query cannot be empty".into()));
        }

        let source_type = match request.source_type.as_deref() {
            Some(value) => parse_source_type(value)?,
            None => VectorSourceType::Code,
        };

        let collection = QdrantClient::collection_name(project_id, &source_type);
        let limit = request.limit.unwrap_or(5).clamp(1, 20);

        let embed_response = self
            .embeddings
            .embed(EmbeddingRequest {
                texts: vec![request.query.clone()],
                task: Some("retrieval.query".into()),
            })
            .await
            .map_err(|e| AppError::External(e.to_string()))?;

        let vector =
            embed_response.vectors.into_iter().next().ok_or_else(|| {
                AppError::External("Embedding provider returned no vectors".into())
            })?;

        let hits = self
            .qdrant
            .search(&collection, vector, limit)
            .await
            .map_err(|e| AppError::External(e.to_string()))?;

        Ok(hits.into_iter().map(map_hit).collect())
    }
}

fn map_hit(hit: SearchHit) -> VectorSearchResult {
    VectorSearchResult {
        content: hit.content,
        source_id: hit.source_id,
        source_type: hit.source_type,
        score: hit.score,
    }
}

fn parse_source_type(value: &str) -> Result<VectorSourceType, AppError> {
    match value.to_lowercase().as_str() {
        "code" => Ok(VectorSourceType::Code),
        "documentation" | "docs" => Ok(VectorSourceType::Documentation),
        "log" | "logs" => Ok(VectorSourceType::Log),
        "trace" | "traces" => Ok(VectorSourceType::Trace),
        "incident" | "incidents" => Ok(VectorSourceType::Incident),
        other => Err(AppError::Validation(format!(
            "Invalid source_type '{other}'. Use code, documentation, log, trace, or incident."
        ))),
    }
}

#[derive(Debug, Deserialize)]
pub struct IndexVectorRequest {
    pub content: String,
    pub source_type: String,
    pub source_id: String,
}

#[derive(Debug, Serialize)]
pub struct IndexedVector {
    pub id: Uuid,
    pub collection: String,
    pub source_type: String,
    pub source_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchVectorRequest {
    pub query: String,
    pub source_type: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct VectorSearchResult {
    pub content: String,
    pub source_id: String,
    pub source_type: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct VectorStatusResponse {
    pub provider: String,
    pub qdrant: String,
}
