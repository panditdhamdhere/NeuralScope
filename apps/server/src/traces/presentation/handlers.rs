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
use crate::traces::application::TraceService;
use crate::traces::domain::{IngestTraceRequest, Trace, TraceDetail, TraceQuery};
use crate::AppError;

#[derive(Serialize)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub meta: ListMeta,
}

#[derive(Serialize)]
pub struct ListMeta {
    pub total: usize,
    pub limit: i64,
    pub offset: i64,
}

/// `POST /api/v1/projects/:project_id/traces`
pub async fn ingest_trace(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<IngestTraceRequest>,
) -> Result<(StatusCode, Json<TraceDetail>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let detail = TraceService::new(&state.db, &state.events)
        .ingest(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(detail)))
}

/// `GET /api/v1/projects/:project_id/traces`
pub async fn list_traces(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<TraceQuery>,
) -> Result<Json<ListResponse<Trace>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let limit = query.limit;
    let offset = query.offset;

    let traces = TraceService::new(&state.db, &state.events)
        .list(project_id, query)
        .await?;

    let total = traces.len();
    Ok(Json(ListResponse {
        data: traces,
        meta: ListMeta {
            total,
            limit,
            offset,
        },
    }))
}

/// `GET /api/v1/projects/:project_id/traces/:trace_id`
pub async fn get_trace(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((project_id, trace_id)): Path<(Uuid, String)>,
) -> Result<Json<TraceDetail>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let detail = TraceService::new(&state.db, &state.events)
        .get_by_trace_id(project_id, &trace_id)
        .await?;

    Ok(Json(detail))
}
