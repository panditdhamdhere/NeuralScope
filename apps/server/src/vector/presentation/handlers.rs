use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::{ensure_project_member, ensure_project_writer};
use crate::auth::presentation::AuthUser;
use crate::vector::application::{
    IndexVectorRequest, IndexedVector, SearchVectorRequest, VectorSearchResult,
    VectorStatusResponse,
};
use crate::AppError;

/// `GET /api/v1/projects/:project_id/vectors/status`
pub async fn vector_status(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<VectorStatusResponse>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let vector = state
        .vector
        .as_ref()
        .ok_or_else(|| AppError::Internal("Vector search is not configured".into()))?;

    let qdrant_status = match vector.health_check().await {
        Ok(()) => "up".into(),
        Err(error) => format!("down: {error}"),
    };

    Ok(Json(VectorStatusResponse {
        provider: vector.embedding_provider().into(),
        qdrant: qdrant_status,
    }))
}

/// `POST /api/v1/projects/:project_id/vectors/index`
pub async fn index_vector(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<IndexVectorRequest>,
) -> Result<(StatusCode, Json<IndexedVector>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let vector = state
        .vector
        .as_ref()
        .ok_or_else(|| AppError::Internal("Vector search is not configured".into()))?;

    let indexed = vector.index(project_id, body).await?;
    Ok((StatusCode::CREATED, Json(indexed)))
}

/// `POST /api/v1/projects/:project_id/vectors/search`
pub async fn search_vectors(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<SearchVectorRequest>,
) -> Result<Json<Vec<VectorSearchResult>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let vector = state
        .vector
        .as_ref()
        .ok_or_else(|| AppError::Internal("Vector search is not configured".into()))?;

    let results = vector.search(project_id, body).await?;
    Ok(Json(results))
}
