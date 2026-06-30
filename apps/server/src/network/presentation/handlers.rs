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
use crate::network::application::NetworkService;
use crate::network::domain::{GraphResponse, IngestNetworkEventRequest, NetworkEvent, NetworkEventQuery};
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

/// `POST /api/v1/projects/:project_id/network/events`
pub async fn ingest_network_event(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<IngestNetworkEventRequest>,
) -> Result<(StatusCode, Json<NetworkEvent>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let event = NetworkService::new(&state.db, &state.events)
        .ingest(project_id, body)
        .await?;

    Ok((StatusCode::CREATED, Json(event)))
}

/// `POST /api/v1/projects/:project_id/network/events/batch`
pub async fn ingest_network_events_batch(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<Vec<IngestNetworkEventRequest>>,
) -> Result<(StatusCode, Json<ListResponse<NetworkEvent>>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let events = NetworkService::new(&state.db, &state.events)
        .ingest_batch(project_id, body)
        .await?;

    let total = events.len();
    Ok((StatusCode::CREATED, Json(ListResponse { data: events, meta: ListMeta { total } })))
}

/// `GET /api/v1/projects/:project_id/network/events`
pub async fn query_network_events(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<NetworkEventQuery>,
) -> Result<Json<ListResponse<NetworkEvent>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let events = NetworkService::new(&state.db, &state.events)
        .query(project_id, query)
        .await?;
    let total = events.len();

    Ok(Json(ListResponse { data: events, meta: ListMeta { total } }))
}

/// `GET /api/v1/projects/:project_id/network/graph`
pub async fn get_network_graph(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<GraphResponse>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let graph = NetworkService::new(&state.db, &state.events)
        .get_graph(project_id)
        .await?;

    Ok(Json(graph))
}
