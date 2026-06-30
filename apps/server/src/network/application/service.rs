//! Network event ingestion, querying, and graph aggregation.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::network::domain::{
    GraphEdge, GraphEdgeData, GraphNode, GraphNodeData, GraphResponse, IngestNetworkEventRequest,
    NetworkEvent, NetworkEventQuery, NetworkNode, NodeType,
};
use crate::AppError;

use super::layout::layout_graph;

/// Network telemetry ingestion and graph generation.
pub struct NetworkService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
}

impl<'a> NetworkService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, events: &'a EventBus) -> Self {
        Self { pool, events }
    }

    pub async fn ingest(
        &self,
        project_id: Uuid,
        request: IngestNetworkEventRequest,
    ) -> Result<NetworkEvent, AppError> {
        let events = self.ingest_batch(project_id, vec![request]).await?;
        events
            .into_iter()
            .next()
            .ok_or_else(|| AppError::Internal("Failed to ingest network event".into()))
    }

    pub async fn ingest_batch(
        &self,
        project_id: Uuid,
        requests: Vec<IngestNetworkEventRequest>,
    ) -> Result<Vec<NetworkEvent>, AppError> {
        if requests.is_empty() {
            return Err(AppError::Validation(
                "At least one network event is required".into(),
            ));
        }

        if requests.len() > 1000 {
            return Err(AppError::Validation(
                "Batch size exceeds maximum of 1000 events".into(),
            ));
        }

        let mut events = Vec::with_capacity(requests.len());
        let mut tx = self.pool.begin().await?;

        for request in requests {
            validate_request(&request)?;
            let event = insert_event(&mut tx, project_id, request).await?;
            events.push(event);
        }

        tx.commit().await?;

        for event in &events {
            self.publish_event(project_id, event);
        }

        Ok(events)
    }

    pub async fn query(
        &self,
        project_id: Uuid,
        query: NetworkEventQuery,
    ) -> Result<Vec<NetworkEvent>, AppError> {
        let limit = query.limit.clamp(1, 500);

        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT id, project_id, source_name, source_type, destination_name, destination_type,
                   protocol, bytes_sent, bytes_received, latency_ms, timestamp
            FROM network_events
            WHERE project_id = $1
              AND ($2::text IS NULL OR source_name ILIKE $2)
              AND ($3::text IS NULL OR destination_name ILIKE $3)
              AND ($4::timestamptz IS NULL OR timestamp >= $4)
              AND ($5::timestamptz IS NULL OR timestamp <= $5)
            ORDER BY timestamp DESC
            LIMIT $6
            "#,
        )
        .bind(project_id)
        .bind(query.source.as_ref().map(|value| format!("%{value}%")))
        .bind(query.destination.as_ref().map(|value| format!("%{value}%")))
        .bind(query.since)
        .bind(query.until)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(EventRow::into_event).collect())
    }

    pub async fn get_graph(&self, project_id: Uuid) -> Result<GraphResponse, AppError> {
        let rows = sqlx::query_as::<_, AggregateRow>(
            r#"
            SELECT source_name, source_type, destination_name, destination_type, protocol,
                   COUNT(*)::bigint AS event_count,
                   COALESCE(SUM(bytes_sent + bytes_received), 0)::bigint AS total_bytes,
                   AVG(latency_ms) AS avg_latency_ms
            FROM network_events
            WHERE project_id = $1
            GROUP BY source_name, source_type, destination_name, destination_type, protocol
            ORDER BY total_bytes DESC
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        Ok(build_graph_from_aggregates(&rows))
    }

    fn publish_event(&self, project_id: Uuid, event: &NetworkEvent) {
        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::NetworkConnection {
                source: event.source.name.clone(),
                destination: event.destination.name.clone(),
            },
            timestamp: Utc::now(),
        };
        let _ = self.events.publish(envelope);
    }
}

fn validate_request(request: &IngestNetworkEventRequest) -> Result<(), AppError> {
    if request.source_name.trim().is_empty() || request.destination_name.trim().is_empty() {
        return Err(AppError::Validation(
            "Source and destination names are required".into(),
        ));
    }

    if request.protocol.trim().is_empty() {
        return Err(AppError::Validation("Protocol is required".into()));
    }

    Ok(())
}

async fn insert_event(
    tx: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    request: IngestNetworkEventRequest,
) -> Result<NetworkEvent, AppError> {
    let timestamp = request.timestamp.unwrap_or_else(Utc::now);

    let row = sqlx::query_as::<_, EventRow>(
        r#"
        INSERT INTO network_events (
            project_id, source_name, source_type, destination_name, destination_type,
            protocol, bytes_sent, bytes_received, latency_ms, timestamp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, project_id, source_name, source_type, destination_name, destination_type,
                  protocol, bytes_sent, bytes_received, latency_ms, timestamp
        "#,
    )
    .bind(project_id)
    .bind(request.source_name.trim())
    .bind(request.source_type.as_str())
    .bind(request.destination_name.trim())
    .bind(request.destination_type.as_str())
    .bind(request.protocol.trim())
    .bind(i64::try_from(request.bytes_sent).unwrap_or(i64::MAX))
    .bind(i64::try_from(request.bytes_received).unwrap_or(i64::MAX))
    .bind(request.latency_ms)
    .bind(timestamp)
    .fetch_one(&mut **tx)
    .await?;

    Ok(row.into_event())
}

fn build_graph_from_aggregates(rows: &[AggregateRow]) -> GraphResponse {
    use std::collections::HashMap;

    let mut node_stats: HashMap<String, (NodeType, u64, u64)> = HashMap::new();

    for row in rows {
        let source_type = NodeType::parse(&row.source_type);
        let destination_type = NodeType::parse(&row.destination_type);
        let event_count = row.event_count.max(0) as u64;
        let total_bytes = row.total_bytes.max(0) as u64;

        node_stats
            .entry(row.source_name.clone())
            .and_modify(|(_, events, bytes)| {
                *events += event_count;
                *bytes += total_bytes;
            })
            .or_insert((source_type, event_count, total_bytes));

        node_stats
            .entry(row.destination_name.clone())
            .and_modify(|(_, events, bytes)| {
                *events += event_count;
                *bytes += total_bytes;
            })
            .or_insert((destination_type, event_count, total_bytes));
    }

    let mut nodes: Vec<GraphNode> = node_stats
        .into_iter()
        .map(|(name, (node_type, event_count, total_bytes))| GraphNode {
            id: node_id(&name),
            label: name,
            node_type: Some(node_type),
            service_type: None,
            position: Default::default(),
            data: GraphNodeData {
                event_count,
                total_bytes,
            },
        })
        .collect();

    let edges: Vec<GraphEdge> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let event_count = row.event_count.max(0) as u64;
            let total_bytes = row.total_bytes.max(0) as u64;
            GraphEdge {
                id: format!("edge-{index}"),
                source: node_id(&row.source_name),
                target: node_id(&row.destination_name),
                label: Some(format_bytes_label(total_bytes, row.avg_latency_ms)),
                data: GraphEdgeData {
                    protocol: row.protocol.clone(),
                    event_count,
                    total_bytes,
                    avg_latency_ms: row.avg_latency_ms,
                },
            }
        })
        .collect();

    layout_graph(&mut nodes);

    GraphResponse { nodes, edges }
}

fn node_id(name: &str) -> String {
    name.to_lowercase().replace([' ', '.'], "-")
}

fn format_bytes_label(total_bytes: u64, latency_ms: Option<f64>) -> String {
    let bytes_label = if total_bytes >= 1_048_576 {
        format!("{:.1} MB", total_bytes as f64 / 1_048_576.0)
    } else if total_bytes >= 1024 {
        format!("{:.1} KB", total_bytes as f64 / 1024.0)
    } else {
        format!("{total_bytes} B")
    };

    match latency_ms {
        Some(latency) => format!("{bytes_label} · {latency:.0}ms"),
        None => bytes_label,
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    project_id: Uuid,
    source_name: String,
    source_type: String,
    destination_name: String,
    destination_type: String,
    protocol: String,
    bytes_sent: i64,
    bytes_received: i64,
    latency_ms: Option<f64>,
    timestamp: DateTime<Utc>,
}

impl EventRow {
    fn into_event(self) -> NetworkEvent {
        NetworkEvent {
            id: self.id,
            project_id: self.project_id,
            source: NetworkNode {
                name: self.source_name,
                node_type: NodeType::parse(&self.source_type),
                address: None,
            },
            destination: NetworkNode {
                name: self.destination_name,
                node_type: NodeType::parse(&self.destination_type),
                address: None,
            },
            protocol: self.protocol,
            bytes_sent: self.bytes_sent.max(0) as u64,
            bytes_received: self.bytes_received.max(0) as u64,
            latency_ms: self.latency_ms,
            timestamp: self.timestamp,
        }
    }
}

#[derive(sqlx::FromRow)]
struct AggregateRow {
    source_name: String,
    source_type: String,
    destination_name: String,
    destination_type: String,
    protocol: String,
    event_count: i64,
    total_bytes: i64,
    avg_latency_ms: Option<f64>,
}
