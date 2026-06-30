use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::metrics::domain::{IngestMetricRequest, MetricPoint, MetricQuery, MetricUnit};
use crate::AppError;

/// Metric ingestion, querying, and real-time event publishing.
pub struct MetricService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
}

impl<'a> MetricService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, events: &'a EventBus) -> Self {
        Self { pool, events }
    }

    /// Ingests a single metric sample.
    pub async fn ingest(
        &self,
        project_id: Uuid,
        request: IngestMetricRequest,
    ) -> Result<MetricPoint, AppError> {
        let points = self.ingest_batch(project_id, vec![request]).await?;
        points
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to ingest metric".into()))
    }

    /// Ingests multiple metric samples in one transaction.
    pub async fn ingest_batch(
        &self,
        project_id: Uuid,
        requests: Vec<IngestMetricRequest>,
    ) -> Result<Vec<MetricPoint>, AppError> {
        if requests.is_empty() {
            return Err(AppError::Validation("At least one metric is required".into()));
        }

        if requests.len() > 5000 {
            return Err(AppError::Validation(
                "Batch size exceeds maximum of 5000 metrics".into(),
            ));
        }

        let mut points = Vec::with_capacity(requests.len());
        let mut tx = self.pool.begin().await?;

        for request in requests {
            validate_metric_request(&request)?;
            let point = insert_metric(&mut tx, project_id, request).await?;
            points.push(point);
        }

        tx.commit().await?;

        for point in &points {
            self.publish_metric_event(project_id, point);
        }

        Ok(points)
    }

    /// Queries metric time-series data with optional filters.
    pub async fn query(
        &self,
        project_id: Uuid,
        query: MetricQuery,
    ) -> Result<Vec<MetricPoint>, AppError> {
        let limit = query.limit.clamp(1, 2000);

        let rows = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT id, project_id, name, value, unit, tags, timestamp
            FROM metrics
            WHERE project_id = $1
              AND ($2::text IS NULL OR name = $2)
              AND ($3::timestamptz IS NULL OR timestamp >= $3)
              AND ($4::timestamptz IS NULL OR timestamp <= $4)
            ORDER BY timestamp ASC
            LIMIT $5
            "#,
        )
        .bind(project_id)
        .bind(query.name)
        .bind(query.since)
        .bind(query.until)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Lists distinct metric names for a project.
    pub async fn list_names(&self, project_id: Uuid) -> Result<Vec<String>, AppError> {
        let names = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT name
            FROM metrics
            WHERE project_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        Ok(names)
    }

    fn publish_metric_event(&self, project_id: Uuid, point: &MetricPoint) {
        self.events.publish(EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::MetricSample {
                name: point.name.clone(),
                value: point.value,
            },
            timestamp: point.timestamp,
        });
    }
}

fn validate_metric_request(request: &IngestMetricRequest) -> Result<(), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::Validation("Metric name is required".into()));
    }

    if !request.value.is_finite() {
        return Err(AppError::Validation("Metric value must be finite".into()));
    }

    Ok(())
}

async fn insert_metric(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    request: IngestMetricRequest,
) -> Result<MetricPoint, AppError> {
    let id = Uuid::new_v4();
    let timestamp = request.timestamp.unwrap_or_else(Utc::now);
    let tags = if request.tags.is_null() {
        serde_json::json!({})
    } else {
        request.tags
    };

    let row = sqlx::query_as::<_, MetricRow>(
        r#"
        INSERT INTO metrics (id, project_id, name, value, unit, tags, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, project_id, name, value, unit, tags, timestamp
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(&request.name)
    .bind(request.value)
    .bind(request.unit.as_str())
    .bind(tags)
    .bind(timestamp)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.into())
}

#[derive(sqlx::FromRow)]
struct MetricRow {
    id: Uuid,
    project_id: Uuid,
    name: String,
    value: f64,
    unit: String,
    tags: serde_json::Value,
    timestamp: DateTime<Utc>,
}

impl From<MetricRow> for MetricPoint {
    fn from(row: MetricRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            name: row.name,
            value: row.value,
            unit: row.unit.parse().unwrap_or(MetricUnit::Count),
            tags: row.tags,
            timestamp: row.timestamp,
        }
    }
}
