use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::application::ensure_project_member;
use crate::auth::presentation::AuthUser;
use crate::overview::application::OverviewService;
use crate::overview::domain::ProjectOverview;
use crate::AppError;

/// `GET /api/v1/projects/:project_id/overview`
pub async fn get_overview(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectOverview>, AppError> {
    ensure_project_member(&state.db, auth.user.id, project_id).await?;

    let overview = OverviewService::new(&state.db)
        .get_overview(project_id)
        .await?;

    Ok(Json(overview))
}
