mod embed;
mod service;

pub use embed::EmbeddingService;
pub use service::{
    IndexVectorRequest, IndexedVector, SearchVectorRequest, VectorSearchResult, VectorService,
    VectorStatusResponse,
};
