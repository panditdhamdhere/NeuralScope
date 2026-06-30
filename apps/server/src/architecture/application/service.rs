//! Architecture graph generation and persistence.

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::architecture::domain::ServiceType;
use crate::network::domain::{
    GraphEdge, GraphEdgeData, GraphNode, GraphNodeData, GraphResponse, NodeType,
};
use crate::AppError;

/// Manages persisted service dependency graphs.
pub struct ArchitectureService<'a> {
    pool: &'a PgPool,
}

impl<'a> ArchitectureService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_graph(&self, project_id: Uuid) -> Result<GraphResponse, AppError> {
        let node_rows = sqlx::query_as::<_, NodeRow>(
            r#"
            SELECT id, name, service_type, position_x, position_y,
                   COALESCE(metadata->>'eventCount', '0')::bigint AS event_count,
                   COALESCE(metadata->>'totalBytes', '0')::bigint AS total_bytes
            FROM architecture_nodes
            WHERE project_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        if node_rows.is_empty() {
            return Ok(GraphResponse {
                nodes: vec![],
                edges: vec![],
            });
        }

        let edge_rows = sqlx::query_as::<_, EdgeRow>(
            r#"
            SELECT id, source_node_id, target_node_id, protocol, avg_latency_ms, request_count
            FROM architecture_edges
            WHERE project_id = $1
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        let id_to_key: HashMap<Uuid, String> = node_rows
            .iter()
            .map(|row| (row.id, node_key(&row.name)))
            .collect();

        let nodes = node_rows
            .into_iter()
            .map(|row| GraphNode {
                id: node_key(&row.name),
                label: row.name,
                node_type: None,
                service_type: Some(row.service_type),
                position: crate::network::domain::GraphPosition {
                    x: row.position_x,
                    y: row.position_y,
                },
                data: GraphNodeData {
                    event_count: row.event_count.max(0) as u64,
                    total_bytes: row.total_bytes.max(0) as u64,
                },
            })
            .collect();

        let edges = edge_rows
            .into_iter()
            .filter_map(|row| {
                let source = id_to_key.get(&row.source_node_id)?.clone();
                let target = id_to_key.get(&row.target_node_id)?.clone();
                Some(GraphEdge {
                    id: row.id.to_string(),
                    source,
                    target,
                    label: row.avg_latency_ms.map(|latency| format!("{latency:.0}ms avg")),
                    data: GraphEdgeData {
                        protocol: row.protocol.unwrap_or_else(|| "http".into()),
                        event_count: row.request_count.max(0) as u64,
                        total_bytes: 0,
                        avg_latency_ms: row.avg_latency_ms,
                    },
                })
            })
            .collect();

        Ok(GraphResponse { nodes, edges })
    }

    pub async fn regenerate(&self, project_id: Uuid) -> Result<GraphResponse, AppError> {
        let aggregates = sqlx::query_as::<_, AggregateRow>(
            r#"
            SELECT source_name, source_type, destination_name, destination_type, protocol,
                   COUNT(*)::bigint AS event_count,
                   COALESCE(SUM(bytes_sent + bytes_received), 0)::bigint AS total_bytes,
                   AVG(latency_ms) AS avg_latency_ms
            FROM network_events
            WHERE project_id = $1
            GROUP BY source_name, source_type, destination_name, destination_type, protocol
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        let span_edges = sqlx::query_as::<_, SpanEdgeRow>(
            r#"
            SELECT parent.service AS source_name, child.service AS target_name,
                   COUNT(*)::bigint AS event_count,
                   AVG(child.duration_ms) AS avg_latency_ms
            FROM spans child
            JOIN spans parent
              ON parent.project_id = child.project_id
             AND parent.trace_id = child.trace_id
             AND parent.span_id = child.parent_span_id
            WHERE child.project_id = $1
            GROUP BY parent.service, child.service
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool)
        .await?;

        let mut node_map: HashMap<String, (ServiceType, u64, u64)> = HashMap::new();

        for row in &aggregates {
            accumulate_node(
                &mut node_map,
                &row.source_name,
                infer_service_type(&row.source_name, &row.source_type),
                row.event_count,
                row.total_bytes,
            );
            accumulate_node(
                &mut node_map,
                &row.destination_name,
                infer_service_type(&row.destination_name, &row.destination_type),
                row.event_count,
                row.total_bytes,
            );
        }

        for row in &span_edges {
            accumulate_node(
                &mut node_map,
                &row.source_name,
                infer_service_type(&row.source_name, "service"),
                row.event_count,
                0,
            );
            accumulate_node(
                &mut node_map,
                &row.target_name,
                infer_service_type(&row.target_name, "service"),
                row.event_count,
                0,
            );
        }

        let mut nodes: Vec<GraphNode> = node_map
            .into_iter()
            .map(|(name, (service_type, event_count, total_bytes))| GraphNode {
                id: node_key(&name),
                label: name,
                node_type: None,
                service_type: Some(service_type.as_str().to_string()),
                position: Default::default(),
                data: GraphNodeData {
                    event_count,
                    total_bytes,
                },
            })
            .collect();

        layout_architecture(&mut nodes);

        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM architecture_edges WHERE project_id = $1")
            .bind(project_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM architecture_nodes WHERE project_id = $1")
            .bind(project_id)
            .execute(&mut *tx)
            .await?;

        let mut db_ids: HashMap<String, Uuid> = HashMap::new();

        for node in &nodes {
            let service_type = node.service_type.clone().unwrap_or_else(|| "api".into());
            let id = sqlx::query_scalar::<_, Uuid>(
                r#"
                INSERT INTO architecture_nodes (
                    project_id, name, service_type, metadata, position_x, position_y
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id
                "#,
            )
            .bind(project_id)
            .bind(&node.label)
            .bind(service_type)
            .bind(serde_json::json!({
                "eventCount": node.data.event_count,
                "totalBytes": node.data.total_bytes,
            }))
            .bind(node.position.x)
            .bind(node.position.y)
            .fetch_one(&mut *tx)
            .await?;

            db_ids.insert(node.id.clone(), id);
        }

        for (index, row) in aggregates.iter().enumerate() {
            insert_edge(
                &mut tx,
                project_id,
                &db_ids,
                &row.source_name,
                &row.destination_name,
                Some(&row.protocol),
                row.avg_latency_ms,
                row.event_count,
                index,
            )
            .await?;
        }

        for (index, row) in span_edges.iter().enumerate() {
            insert_edge(
                &mut tx,
                project_id,
                &db_ids,
                &row.source_name,
                &row.target_name,
                Some("trace"),
                row.avg_latency_ms,
                row.event_count,
                aggregates.len() + index,
            )
            .await?;
        }

        tx.commit().await?;

        self.get_graph(project_id).await
    }
}

async fn insert_edge(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    project_id: Uuid,
    db_ids: &HashMap<String, Uuid>,
    source_name: &str,
    target_name: &str,
    protocol: Option<&str>,
    avg_latency_ms: Option<f64>,
    request_count: i64,
    seed: usize,
) -> Result<(), AppError> {
    let source_id = match db_ids.get(&node_key(source_name)) {
        Some(id) => *id,
        None => return Ok(()),
    };
    let target_id = match db_ids.get(&node_key(target_name)) {
        Some(id) => *id,
        None => return Ok(()),
    };

    if source_id == target_id {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO architecture_edges (
            id, project_id, source_node_id, target_node_id, protocol, avg_latency_ms, request_count
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (project_id, source_node_id, target_node_id) DO UPDATE
        SET request_count = architecture_edges.request_count + EXCLUDED.request_count,
            avg_latency_ms = COALESCE(EXCLUDED.avg_latency_ms, architecture_edges.avg_latency_ms)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(project_id)
    .bind(source_id)
    .bind(target_id)
    .bind(protocol)
    .bind(avg_latency_ms)
    .bind(request_count.max(1))
    .execute(&mut **tx)
    .await?;

    let _ = seed;
    Ok(())
}

fn accumulate_node(
    map: &mut HashMap<String, (ServiceType, u64, u64)>,
    name: &str,
    service_type: ServiceType,
    events: i64,
    bytes: i64,
) {
    let event_count = events.max(0) as u64;
    let total_bytes = bytes.max(0) as u64;
    map.entry(name.to_string())
        .and_modify(|(_, e, b)| {
            *e += event_count;
            *b += total_bytes;
        })
        .or_insert((service_type, event_count, total_bytes));
}

fn infer_service_type(name: &str, node_type: &str) -> ServiceType {
    let lower = name.to_lowercase();
    if lower.contains("browser") || lower.contains("frontend") || lower.contains("web") {
        return ServiceType::Frontend;
    }
    if lower.contains("gateway") || lower.contains("nginx") || lower.contains("proxy") {
        return ServiceType::Gateway;
    }
    if lower.contains("auth") {
        return ServiceType::Auth;
    }
    if lower.contains("redis") || lower.contains("cache") {
        return ServiceType::Cache;
    }
    if lower.contains("queue") || lower.contains("kafka") || lower.contains("rabbit") {
        return ServiceType::Queue;
    }
    if lower.contains("postgres") || lower.contains("mysql") || lower.contains("db") {
        return ServiceType::Database;
    }
    if lower.contains("stripe") || lower.contains("external") {
        return ServiceType::External;
    }

    match NodeType::parse(node_type) {
        NodeType::Browser => ServiceType::Frontend,
        NodeType::Database => ServiceType::Database,
        NodeType::Cache => ServiceType::Cache,
        NodeType::Queue => ServiceType::Queue,
        NodeType::External => ServiceType::External,
        _ => ServiceType::Api,
    }
}

fn layout_architecture(nodes: &mut [GraphNode]) {
    let mut tiers: HashMap<i32, Vec<usize>> = HashMap::new();

    for (index, node) in nodes.iter().enumerate() {
        let tier = service_tier(node.service_type.as_deref().unwrap_or("api"));
        tiers.entry(tier).or_default().push(index);
    }

    for (tier, indices) in tiers {
        let count = indices.len() as f64;
        for (position, index) in indices.into_iter().enumerate() {
            let x = if count <= 1.0 {
                280.0
            } else {
                60.0 + (position as f64) * (520.0 / (count - 1.0))
            };
            nodes[index].position.x = x;
            nodes[index].position.y = 60.0 + f64::from(tier) * 150.0;
        }
    }
}

fn service_tier(service_type: &str) -> i32 {
    match service_type {
        "frontend" => 0,
        "gateway" => 1,
        "api" | "auth" => 2,
        "cache" | "queue" => 3,
        "database" | "external" => 4,
        _ => 2,
    }
}

fn node_key(name: &str) -> String {
    name.to_lowercase().replace([' ', '.'], "-")
}

#[derive(sqlx::FromRow)]
struct NodeRow {
    id: Uuid,
    name: String,
    service_type: String,
    position_x: f64,
    position_y: f64,
    event_count: i64,
    total_bytes: i64,
}

#[derive(sqlx::FromRow)]
struct EdgeRow {
    id: Uuid,
    source_node_id: Uuid,
    target_node_id: Uuid,
    protocol: Option<String>,
    avg_latency_ms: Option<f64>,
    request_count: i64,
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

#[derive(sqlx::FromRow)]
struct SpanEdgeRow {
    source_name: String,
    target_name: String,
    event_count: i64,
    avg_latency_ms: Option<f64>,
}
