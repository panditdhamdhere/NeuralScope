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
use crate::security::application::SecurityService;
use crate::security::domain::{FindingQuery, ScanRequest, ScanResult, SecurityFinding};
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

/// `GET /api/v1/projects/:project_id/security/findings`
pub async fn list_findings(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<FindingQuery>,
) -> Result<Json<ListResponse<SecurityFinding>>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let findings = SecurityService::new(&state.db, &state.events)
        .list(project_id, query)
        .await?;
    let total = findings.len();

    Ok(Json(ListResponse {
        data: findings,
        meta: ListMeta { total },
    }))
}

/// `POST /api/v1/projects/:project_id/security/scan`
pub async fn run_scan(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<ScanRequest>,
) -> Result<(StatusCode, Json<ScanResult>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let result = SecurityService::new(&state.db, &state.events)
        .scan(project_id, body)
        .await?;

    Ok((StatusCode::OK, Json(result)))
}

/// `POST /api/v1/projects/:project_id/security/findings/sample`
pub async fn load_sample_findings(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ScanResult>), AppError> {
    ensure_project_writer(&state.db, auth.user.id, project_id).await?;

    let result = SecurityService::new(&state.db, &state.events)
        .scan(
            project_id,
            ScanRequest {
                content: Some(sample_config()),
                scan_logs: false,
            },
        )
        .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

fn sample_config() -> String {
    r#"
    DEBUG=true
    DATABASE_URL=postgres://admin:password@0.0.0.0:5432/app
    STRIPE_KEY=sk_test_abc123
    GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456
    "#
    .to_string()
}
