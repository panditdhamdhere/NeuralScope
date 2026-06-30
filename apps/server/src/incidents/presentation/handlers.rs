use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::{ensure_project_member, ensure_project_writer};
use crate::auth::presentation::AuthUser;
use crate::incidents::application::IncidentService;
use crate::incidents::domain::{
    GenerateIncidentRequest, Incident, IncidentQuery, UpdateIncidentRequest,
};
use crate::AppError;

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub total: usize,
}

/// `GET /api/v1/projects/:project_id/incidents`
pub async fn list_incidents(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<IncidentQuery>,
) -> Result<Json<ListResponse<Incident>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let incidents = IncidentService::new(&state.db, &state.events, state.ai_provider.clone())
        .list(project_id, query)
        .await?;
    let total = incidents.len();

    Ok(Json(ListResponse {
        data: incidents,
        meta: ListMeta { total },
    }))
}

/// `GET /api/v1/projects/:project_id/incidents/:incident_id`
pub async fn get_incident(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((project_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Incident>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let incident = IncidentService::new(&state.db, &state.events, state.ai_provider.clone())
        .get(project_id, incident_id)
        .await?;

    Ok(Json(incident))
}

/// `POST /api/v1/projects/:project_id/incidents/generate`
pub async fn generate_incident(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<GenerateIncidentRequest>,
) -> Result<(StatusCode, Json<Incident>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let incident = IncidentService::new(&state.db, &state.events, state.ai_provider.clone())
        .generate(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(incident)))
}

/// `PATCH /api/v1/projects/:project_id/incidents/:incident_id`
pub async fn update_incident(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((project_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateIncidentRequest>,
) -> Result<Json<Incident>, AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let incident = IncidentService::new(&state.db, &state.events, state.ai_provider.clone())
        .update(project_id, incident_id, body)
        .await?;

    Ok(Json(incident))
}
