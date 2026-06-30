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
use crate::logs::application::LogService;
use crate::logs::domain::{IngestLogRequest, LogEntry, LogSearchQuery};
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

/// `POST /api/v1/projects/:project_id/logs`
pub async fn ingest_log(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<IngestLogRequest>,
) -> Result<(StatusCode, Json<LogEntry>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let entry = LogService::new(&state.db, &state.events)
        .ingest(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(entry)))
}

/// `POST /api/v1/projects/:project_id/logs/batch`
pub async fn ingest_logs_batch(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<Vec<IngestLogRequest>>,
) -> Result<(StatusCode, Json<ListResponse<LogEntry>>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let entries = LogService::new(&state.db, &state.events)
        .ingest_batch(project_id, body)
        .await?;

    let total = entries.len();
    Ok((
        StatusCode::CREATED,
        Json(ListResponse {
            data: entries,
            meta: ListMeta {
                total,
                limit: total as i64,
                offset: 0,
            },
        }),
    ))
}

/// `GET /api/v1/projects/:project_id/logs`
pub async fn search_logs(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<LogSearchQuery>,
) -> Result<Json<ListResponse<LogEntry>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let limit = query.limit;
    let offset = query.offset;

    let entries = LogService::new(&state.db, &state.events)
        .search(project_id, query)
        .await?;

    let total = entries.len();
    Ok(Json(ListResponse {
        data: entries,
        meta: ListMeta {
            total,
            limit,
            offset,
        },
    }))
}
