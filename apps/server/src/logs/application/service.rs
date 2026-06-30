use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::logs::domain::{IngestLogRequest, LogEntry, LogLevel, LogSearchQuery};
use crate::AppError;

/// Log ingestion, search, and real-time event publishing.
pub struct LogService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
}

impl<'a> LogService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, events: &'a EventBus) -> Self {
        Self { pool, events }
    }

    /// Ingests a single log entry and broadcasts it to subscribers.
    pub async fn ingest(
        &self,
        project_id: Uuid,
        request: IngestLogRequest,
    ) -> Result<LogEntry, AppError> {
        let entries = self.ingest_batch(project_id, vec![request]).await?;
        entries
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to ingest log entry".into()))
    }

    /// Ingests multiple log entries in a single transaction.
    pub async fn ingest_batch(
        &self,
        project_id: Uuid,
        requests: Vec<IngestLogRequest>,
    ) -> Result<Vec<LogEntry>, AppError> {
        if requests.is_empty() {
            return Err(AppError::Validation("At least one log entry is required".into()));
        }

        if requests.len() > 1000 {
            return Err(AppError::Validation(
                "Batch size exceeds maximum of 1000 entries".into(),
            ));
        }

        let mut entries = Vec::with_capacity(requests.len());
        let mut tx = self.pool.begin().await?;

        for request in requests {
            if request.message.trim().is_empty() {
                return Err(AppError::Validation("Log message cannot be empty".into()));
            }

            let entry = insert_log(&mut tx, project_id, request).await?;
            entries.push(entry);
        }

        tx.commit().await?;

        for entry in &entries {
            self.publish_log_event(project_id, entry);
        }

        Ok(entries)
    }

    /// Searches log entries for a project with optional filters.
    pub async fn search(
        &self,
        project_id: Uuid,
        query: LogSearchQuery,
    ) -> Result<Vec<LogEntry>, AppError> {
        let limit = query.limit.clamp(1, 500);
        let offset = query.offset.max(0);

        let level = query.level.map(|l| l.as_str().to_string());
        let search_pattern = query
            .search
            .as_ref()
            .map(|s| format!("%{}%", s.to_lowercase()));

        let rows = sqlx::query_as::<_, LogRow>(
            r#"
            SELECT id, project_id, timestamp, level, message, service, trace_id, metadata
            FROM logs
            WHERE project_id = $1
              AND ($2::text IS NULL OR level = $2)
              AND ($3::text IS NULL OR service = $3)
              AND ($4::text IS NULL OR trace_id = $4)
              AND ($5::text IS NULL OR LOWER(message) LIKE $5)
            ORDER BY timestamp DESC
            LIMIT $6 OFFSET $7
            "#,
        )
        .bind(project_id)
        .bind(level)
        .bind(query.service)
        .bind(query.trace_id)
        .bind(search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    fn publish_log_event(&self, project_id: Uuid, entry: &LogEntry) {
        self.events.publish(EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::LogNew {
                entry_id: entry.id,
                level: entry.level.as_str().to_string(),
                message: entry.message.clone(),
            },
            timestamp: entry.timestamp,
        });
    }
}

async fn insert_log(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    request: IngestLogRequest,
) -> Result<LogEntry, AppError> {
    let id = Uuid::new_v4();
    let timestamp = request.timestamp.unwrap_or_else(Utc::now);
    let metadata = if request.metadata.is_null() {
        serde_json::json!({})
    } else {
        request.metadata
    };

    let row = sqlx::query_as::<_, LogRow>(
        r#"
        INSERT INTO logs (id, project_id, timestamp, level, message, service, trace_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, project_id, timestamp, level, message, service, trace_id, metadata
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(timestamp)
    .bind(request.level.as_str())
    .bind(&request.message)
    .bind(&request.service)
    .bind(&request.trace_id)
    .bind(metadata)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.into())
}

#[derive(sqlx::FromRow)]
struct LogRow {
    id: Uuid,
    project_id: Uuid,
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    service: Option<String>,
    trace_id: Option<String>,
    metadata: serde_json::Value,
}

impl From<LogRow> for LogEntry {
    fn from(row: LogRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            timestamp: row.timestamp,
            level: row.level.parse().unwrap_or(LogLevel::Info),
            message: row.message,
            service: row.service,
            trace_id: row.trace_id,
            metadata: row.metadata,
        }
    }
}
