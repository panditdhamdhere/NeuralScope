use async_trait::async_trait;

/// Request to generate embeddings.
#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
    pub task: Option<String>,
}

/// Embedding vectors returned by a provider.
#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub vectors: Vec<Vec<f32>>,
    pub model: String,
    pub dimensions: usize,
}

/// Metadata for a stored vector embedding.
#[derive(Debug, Clone)]
pub struct VectorRecord {
    pub id: uuid::Uuid,
    pub project_id: uuid::Uuid,
    pub content: String,
    pub source_type: VectorSourceType,
    pub source_id: String,
    pub collection: String,
}

/// Source type for embedded content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorSourceType {
    Log,
    Code,
    Documentation,
    Trace,
    Incident,
}

/// Abstraction over embedding providers (Jina, Nomic, stub).
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse, EmbeddingError>;
    fn name(&self) -> &str;
    fn dimensions(&self) -> usize;
}

/// Errors from embedding providers.
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
