use uuid::Uuid;

use crate::auth::application::ProjectService;
use crate::auth::domain::ProjectRole;
use crate::AppError;

/// Verifies that a user is a member of the given project.
///
/// # Errors
///
/// Returns `Forbidden` if the user is not a project member.
pub async fn ensure_project_member(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectRole, AppError> {
    ProjectService::new(pool)
        .get_member_role(user_id, project_id)
        .await
}

/// Verifies that a user can mutate project data (owner or admin).
///
/// # Errors
///
/// Returns `Forbidden` if the user is a viewer or not a member.
pub async fn ensure_project_writer(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectRole, AppError> {
    let role = ensure_project_member(pool, user_id, project_id).await?;
    if role.can_write() {
        Ok(role)
    } else {
        Err(AppError::Forbidden(
            "Write access requires owner or admin role".into(),
        ))
    }
}

/// Verifies that a user can administer a project (owner only).
///
/// # Errors
///
/// Returns `Forbidden` if the user is not the project owner.
pub async fn ensure_project_admin(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<ProjectRole, AppError> {
    let role = ensure_project_member(pool, user_id, project_id).await?;
    if role.can_admin() {
        Ok(role)
    } else {
        Err(AppError::Forbidden(
            "Admin access requires project owner role".into(),
        ))
    }
}
