use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::traces::domain::{
    IngestSpanRequest, IngestTraceRequest, Span, Trace, TraceDetail, TraceQuery, TraceStatus,
};
use crate::AppError;

/// Trace ingestion, listing, and real-time event publishing.
pub struct TraceService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
}

impl<'a> TraceService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, events: &'a EventBus) -> Self {
        Self { pool, events }
    }

    /// Ingests a distributed trace with all spans atomically.
    pub async fn ingest(
        &self,
        project_id: Uuid,
        request: IngestTraceRequest,
    ) -> Result<TraceDetail, AppError> {
        if request.trace_id.trim().is_empty() {
            return Err(AppError::Validation("trace_id is required".into()));
        }

        if request.spans.is_empty() {
            return Err(AppError::Validation("At least one span is required".into()));
        }

        if request.spans.len() > 1000 {
            return Err(AppError::Validation(
                "Trace exceeds maximum of 1000 spans".into(),
            ));
        }

        let summary = compute_trace_summary(&request.spans)?;
        let mut tx = self.pool.begin().await?;

        let trace_row = sqlx::query_as::<_, TraceRow>(
            r#"
            INSERT INTO traces (id, project_id, trace_id, root_service, duration_ms, span_count, status, started_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (project_id, trace_id) DO UPDATE SET
                root_service = EXCLUDED.root_service,
                duration_ms = EXCLUDED.duration_ms,
                span_count = EXCLUDED.span_count,
                status = EXCLUDED.status,
                started_at = EXCLUDED.started_at
            RETURNING id, project_id, trace_id, root_service, duration_ms, span_count, status, started_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(project_id)
        .bind(&request.trace_id)
        .bind(&summary.root_service)
        .bind(summary.duration_ms)
        .bind(summary.span_count as i32)
        .bind(summary.status.as_str())
        .bind(summary.started_at)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "DELETE FROM spans WHERE project_id = $1 AND trace_id = $2",
        )
        .bind(project_id)
        .bind(&request.trace_id)
        .execute(&mut *tx)
        .await?;

        let mut spans = Vec::with_capacity(request.spans.len());

        for span_req in request.spans {
            validate_span(&span_req)?;
            let span = insert_span(&mut tx, project_id, &request.trace_id, span_req).await?;
            spans.push(span);
        }

        tx.commit().await?;

        let trace: Trace = trace_row.into();
        let detail = TraceDetail {
            trace: trace.clone(),
            spans,
        };

        self.publish_trace_event(project_id, &trace);

        Ok(detail)
    }

    /// Lists traces for a project with optional filters.
    pub async fn list(
        &self,
        project_id: Uuid,
        query: TraceQuery,
    ) -> Result<Vec<Trace>, AppError> {
        let limit = query.limit.clamp(1, 200);
        let offset = query.offset.max(0);
        let status = query.status.map(|s| s.as_str().to_string());

        let rows = sqlx::query_as::<_, TraceRow>(
            r#"
            SELECT id, project_id, trace_id, root_service, duration_ms, span_count, status, started_at
            FROM traces
            WHERE project_id = $1
              AND ($2::text IS NULL OR root_service = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::timestamptz IS NULL OR started_at >= $4)
            ORDER BY started_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(project_id)
        .bind(query.service)
        .bind(status)
        .bind(query.since)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Fetches a trace and all its spans by trace ID.
    pub async fn get_by_trace_id(
        &self,
        project_id: Uuid,
        trace_id: &str,
    ) -> Result<TraceDetail, AppError> {
        let trace_row = sqlx::query_as::<_, TraceRow>(
            r#"
            SELECT id, project_id, trace_id, root_service, duration_ms, span_count, status, started_at
            FROM traces
            WHERE project_id = $1 AND trace_id = $2
            "#,
        )
        .bind(project_id)
        .bind(trace_id)
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Trace not found".into()))?;

        let span_rows = sqlx::query_as::<_, SpanRow>(
            r#"
            SELECT id, trace_id, span_id, parent_span_id, service, operation, duration_ms, status, attributes, started_at
            FROM spans
            WHERE project_id = $1 AND trace_id = $2
            ORDER BY started_at ASC
            "#,
        )
        .bind(project_id)
        .bind(trace_id)
        .fetch_all(self.pool)
        .await?;

        Ok(TraceDetail {
            trace: trace_row.into(),
            spans: span_rows.into_iter().map(Into::into).collect(),
        })
    }

    fn publish_trace_event(&self, project_id: Uuid, trace: &Trace) {
        self.events.publish(EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::TraceComplete {
                trace_id: trace.trace_id.clone(),
                duration_ms: trace.duration_ms,
            },
            timestamp: trace.started_at,
        });
    }
}

struct TraceSummary {
    root_service: String,
    duration_ms: f64,
    span_count: u32,
    status: TraceStatus,
    started_at: DateTime<Utc>,
}

fn compute_trace_summary(spans: &[IngestSpanRequest]) -> Result<TraceSummary, AppError> {
    let started_at = spans
        .iter()
        .filter_map(|s| s.started_at)
        .min()
        .unwrap_or_else(Utc::now);

    let duration_ms = spans.iter().map(|s| s.duration_ms).fold(0.0, f64::max);

    let root_service = spans
        .iter()
        .find(|s| s.parent_span_id.is_none())
        .map(|s| s.service.clone())
        .unwrap_or_else(|| spans[0].service.clone());

    let status = if spans.iter().any(|s| s.status == TraceStatus::Error) {
        TraceStatus::Error
    } else {
        TraceStatus::Ok
    };

    Ok(TraceSummary {
        root_service,
        duration_ms,
        span_count: spans.len() as u32,
        status,
        started_at,
    })
}

fn validate_span(span: &IngestSpanRequest) -> Result<(), AppError> {
    if span.span_id.trim().is_empty() {
        return Err(AppError::Validation("span_id is required".into()));
    }
    if span.service.trim().is_empty() {
        return Err(AppError::Validation("service is required".into()));
    }
    if span.operation.trim().is_empty() {
        return Err(AppError::Validation("operation is required".into()));
    }
    if !span.duration_ms.is_finite() || span.duration_ms < 0.0 {
        return Err(AppError::Validation("duration_ms must be non-negative".into()));
    }
    Ok(())
}

async fn insert_span(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    trace_id: &str,
    request: IngestSpanRequest,
) -> Result<Span, AppError> {
    let id = Uuid::new_v4();
    let started_at = request.started_at.unwrap_or_else(Utc::now);
    let attributes = if request.attributes.is_null() {
        serde_json::json!({})
    } else {
        request.attributes
    };

    let row = sqlx::query_as::<_, SpanRow>(
        r#"
        INSERT INTO spans (id, project_id, trace_id, span_id, parent_span_id, service, operation, duration_ms, status, attributes, started_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, trace_id, span_id, parent_span_id, service, operation, duration_ms, status, attributes, started_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(trace_id)
    .bind(&request.span_id)
    .bind(&request.parent_span_id)
    .bind(&request.service)
    .bind(&request.operation)
    .bind(request.duration_ms)
    .bind(request.status.as_str())
    .bind(attributes)
    .bind(started_at)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.into())
}

#[derive(sqlx::FromRow)]
struct TraceRow {
    id: Uuid,
    project_id: Uuid,
    trace_id: String,
    root_service: String,
    duration_ms: f64,
    span_count: i32,
    status: String,
    started_at: DateTime<Utc>,
}

impl From<TraceRow> for Trace {
    fn from(row: TraceRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            trace_id: row.trace_id,
            root_service: row.root_service,
            duration_ms: row.duration_ms,
            span_count: row.span_count as u32,
            status: row.status.parse().unwrap_or(TraceStatus::Unset),
            started_at: row.started_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct SpanRow {
    id: Uuid,
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    service: String,
    operation: String,
    duration_ms: f64,
    status: String,
    attributes: serde_json::Value,
    started_at: DateTime<Utc>,
}

impl From<SpanRow> for Span {
    fn from(row: SpanRow) -> Self {
        Self {
            id: row.id,
            trace_id: row.trace_id,
            span_id: row.span_id,
            parent_span_id: row.parent_span_id,
            service: row.service,
            operation: row.operation,
            duration_ms: row.duration_ms,
            status: row.status.parse().unwrap_or(TraceStatus::Unset),
            attributes: row.attributes,
            started_at: row.started_at,
        }
    }
}
