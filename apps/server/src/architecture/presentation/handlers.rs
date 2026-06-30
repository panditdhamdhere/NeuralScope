use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::architecture::application::ArchitectureService;
use crate::auth::application::{ensure_project_member, ensure_project_writer};
use crate::auth::presentation::AuthUser;
use crate::network::domain::GraphResponse;
use crate::AppError;

/// `GET /api/v1/projects/:project_id/architecture/graph`
pub async fn get_architecture_graph(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<GraphResponse>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let graph = ArchitectureService::new(&state.db)
        .get_graph(project_id)
        .await?;

    Ok(Json(graph))
}

/// `POST /api/v1/projects/:project_id/architecture/regenerate`
pub async fn regenerate_architecture_graph(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<(StatusCode, Json<GraphResponse>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let graph = ArchitectureService::new(&state.db)
        .regenerate(project_id)
        .await?;

    Ok((StatusCode::OK, Json(graph)))
}
