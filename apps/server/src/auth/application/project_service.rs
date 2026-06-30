use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::application::crypto::slugify;
use crate::auth::domain::{Project, ProjectRole};
use crate::AppError;

/// Manages project workspaces and membership.
pub struct ProjectService<'a> {
    pool: &'a PgPool,
}

impl<'a> ProjectService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Creates a project and assigns the creator as owner.
    pub async fn create(&self, user_id: Uuid, name: &str) -> Result<Project, AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Project name is required".into()));
        }

        let base_slug = slugify(name);
        if base_slug.is_empty() {
            return Err(AppError::Validation("Invalid project name".into()));
        }

        let slug = self.unique_slug(&base_slug).await?;
        let project_id = Uuid::new_v4();
        let now = Utc::now();

        let mut tx = self.pool.begin().await?;

        let project = sqlx::query_as::<_, ProjectRow>(
            r#"
            INSERT INTO projects (id, name, slug, owner_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $5)
            RETURNING id, name, slug, owner_id, created_at
            "#,
        )
        .bind(project_id)
        .bind(name)
        .bind(&slug)
        .bind(user_id)
        .bind(now)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO project_members (project_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
        )
        .bind(project_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(project.into_project(ProjectRole::Owner))
    }

    /// Lists all projects accessible to the user.
    pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Project>, AppError> {
        let rows = sqlx::query_as::<_, ProjectMemberRow>(
            r#"
            SELECT p.id, p.name, p.slug, p.owner_id, p.created_at, pm.role
            FROM projects p
            INNER JOIN project_members pm ON pm.project_id = p.id
            WHERE pm.user_id = $1
            ORDER BY p.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Fetches a project by slug if the user is a member.
    pub async fn get_by_slug(&self, user_id: Uuid, slug: &str) -> Result<Project, AppError> {
        let row = sqlx::query_as::<_, ProjectMemberRow>(
            r#"
            SELECT p.id, p.name, p.slug, p.owner_id, p.created_at, pm.role
            FROM projects p
            INNER JOIN project_members pm ON pm.project_id = p.id
            WHERE p.slug = $1 AND pm.user_id = $2
            "#,
        )
        .bind(slug)
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;

        row.map(Into::into)
            .ok_or_else(|| AppError::NotFound("Project not found".into()))
    }

    /// Returns the user's role in a project.
    pub async fn get_member_role(
        &self,
        user_id: Uuid,
        project_id: Uuid,
    ) -> Result<ProjectRole, AppError> {
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM project_members WHERE project_id = $1 AND user_id = $2",
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(self.pool)
        .await?;

        role.map(|r| ProjectRole::parse(&r))
            .transpose()
            .map_err(|e| AppError::Internal(e))?
            .ok_or_else(|| AppError::Forbidden("Not a project member".into()))
    }

    async fn unique_slug(&self, base: &str) -> Result<String, AppError> {
        let mut slug = base.to_string();
        let mut suffix = 0;

        loop {
            let exists: bool =
                sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE slug = $1)")
                    .bind(&slug)
                    .fetch_one(self.pool)
                    .await?;

            if !exists {
                return Ok(slug);
            }

            suffix += 1;
            slug = format!("{base}-{suffix}");
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectRow {
    id: Uuid,
    name: String,
    slug: String,
    owner_id: Uuid,
    created_at: DateTime<Utc>,
}

impl ProjectRow {
    fn into_project(self, role: ProjectRole) -> Project {
        Project {
            id: self.id,
            name: self.name,
            slug: self.slug,
            owner_id: self.owner_id,
            role,
            created_at: self.created_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProjectMemberRow {
    id: Uuid,
    name: String,
    slug: String,
    owner_id: Uuid,
    created_at: DateTime<Utc>,
    role: String,
}

impl From<ProjectMemberRow> for Project {
    fn from(row: ProjectMemberRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            owner_id: row.owner_id,
            role: ProjectRole::parse(&row.role).unwrap_or(ProjectRole::Viewer),
            created_at: row.created_at,
        }
    }
}
