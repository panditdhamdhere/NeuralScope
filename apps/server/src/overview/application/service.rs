use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::overview::domain::{ProjectOverview, RecentLog};
use crate::AppError;

/// Aggregates telemetry counts for the project overview dashboard.
pub struct OverviewService<'a> {
    pool: &'a PgPool,
}

impl<'a> OverviewService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_overview(&self, project_id: Uuid) -> Result<ProjectOverview, AppError> {
        let since = Utc::now() - Duration::hours(24);

        let error_logs_24h = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM logs
            WHERE project_id = $1 AND timestamp >= $2
              AND level IN ('error', 'fatal')
            "#,
        )
        .bind(project_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        let total_logs_24h = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM logs
            WHERE project_id = $1 AND timestamp >= $2
            "#,
        )
        .bind(project_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        let traces_24h = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM traces
            WHERE project_id = $1 AND started_at >= $2
            "#,
        )
        .bind(project_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        let failed_traces_24h = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM traces
            WHERE project_id = $1 AND started_at >= $2 AND status = 'error'
            "#,
        )
        .bind(project_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        let open_incidents = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM incidents
            WHERE project_id = $1 AND status != 'resolved'
            "#,
        )
        .bind(project_id)
        .fetch_one(self.pool)
        .await?;

        let critical_findings = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM security_findings
            WHERE project_id = $1 AND severity IN ('critical', 'high')
            "#,
        )
        .bind(project_id)
        .fetch_one(self.pool)
        .await?;

        let conversations = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM chat_history
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_one(self.pool)
        .await?;

        let recent_logs = sqlx::query_as::<_, RecentLogRow>(
            r#"
            SELECT id, level, message, service, timestamp
            FROM logs
            WHERE project_id = $1
            ORDER BY timestamp DESC
            LIMIT 8
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?
        .into_iter()
        .map(|row| RecentLog {
            id: row.id,
            level: row.level,
            message: row.message,
            service: row.service,
            timestamp: row.timestamp,
        })
        .collect();

        let cpu_usage = latest_metric(self.pool, project_id, "cpu.usage").await?;
        let memory_usage = latest_metric(self.pool, project_id, "memory.usage").await?;

        Ok(ProjectOverview {
            error_logs_24h,
            total_logs_24h,
            traces_24h,
            failed_traces_24h,
            open_incidents,
            critical_findings,
            conversations,
            recent_logs,
            cpu_usage,
            memory_usage,
            server_status: "ready".into(),
        })
    }
}

async fn latest_metric(
    pool: &PgPool,
    project_id: Uuid,
    name: &str,
) -> Result<Option<f64>, AppError> {
    let value = sqlx::query_scalar::<_, Option<f64>>(
        r#"
        SELECT value FROM metrics
        WHERE project_id = $1 AND name = $2
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
    )
    .bind(project_id)
    .bind(name)
    .fetch_optional(pool)
    .await?;

    Ok(value.flatten())
}

#[derive(sqlx::FromRow)]
struct RecentLogRow {
    id: Uuid,
    level: String,
    message: String,
    service: Option<String>,
    timestamp: chrono::DateTime<Utc>,
}
