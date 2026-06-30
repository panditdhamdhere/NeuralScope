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
use crate::metrics::application::MetricService;
use crate::metrics::domain::{IngestMetricRequest, MetricPoint, MetricQuery};
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

#[derive(Serialize)]
pub struct NamesResponse {
    pub data: Vec<String>,
}

/// `POST /api/v1/projects/:project_id/metrics`
pub async fn ingest_metric(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<IngestMetricRequest>,
) -> Result<(StatusCode, Json<MetricPoint>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let point = MetricService::new(&state.db, &state.events)
        .ingest(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(point)))
}

/// `POST /api/v1/projects/:project_id/metrics/batch`
pub async fn ingest_metrics_batch(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<Vec<IngestMetricRequest>>,
) -> Result<(StatusCode, Json<ListResponse<MetricPoint>>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let points = MetricService::new(&state.db, &state.events)
        .ingest_batch(project_id, body)
        .await?;

    let total = points.len();
    Ok((
        StatusCode::CREATED,
        Json(ListResponse {
            data: points,
            meta: ListMeta { total },
        }),
    ))
}

/// `GET /api/v1/projects/:project_id/metrics`
pub async fn query_metrics(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<MetricQuery>,
) -> Result<Json<ListResponse<MetricPoint>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let points = MetricService::new(&state.db, &state.events)
        .query(project_id, query)
        .await?;

    let total = points.len();
    Ok(Json(ListResponse {
        data: points,
        meta: ListMeta { total },
    }))
}

/// `GET /api/v1/projects/:project_id/metrics/names`
pub async fn list_metric_names(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<NamesResponse>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let names = MetricService::new(&state.db, &state.events)
        .list_names(project_id)
        .await?;

    Ok(Json(NamesResponse { data: names }))
}
