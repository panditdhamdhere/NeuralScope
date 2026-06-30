//! Security scanning and findings management.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::security::domain::{
    DetectedFinding, FindingQuery, FindingType, ScanRequest, ScanResult, SecurityFinding, Severity,
};
use crate::security::infrastructure::SecurityScanner;
use crate::AppError;

/// Security scanning and findings persistence.
pub struct SecurityService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
}

impl<'a> SecurityService<'a> {
    #[must_use]
    pub fn new(pool: &'a PgPool, events: &'a EventBus) -> Self {
        Self { pool, events }
    }

    pub async fn list(
        &self,
        project_id: Uuid,
        query: FindingQuery,
    ) -> Result<Vec<SecurityFinding>, AppError> {
        let limit = query.limit.clamp(1, 500);

        let rows = sqlx::query_as::<_, FindingRow>(
            r#"
            SELECT id, project_id, finding_type, severity, title, description, resource, detected_at
            FROM security_findings
            WHERE project_id = $1
              AND ($2::text IS NULL OR severity = $2)
              AND ($3::text IS NULL OR finding_type = $3)
            ORDER BY detected_at DESC
            LIMIT $4
            "#,
        )
        .bind(project_id)
        .bind(query.severity.map(|s| s.as_str().to_string()))
        .bind(query.finding_type.map(|t| t.as_str().to_string()))
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(FindingRow::into_finding).collect())
    }

    pub async fn scan(
        &self,
        project_id: Uuid,
        request: ScanRequest,
    ) -> Result<ScanResult, AppError> {
        let mut detected = Vec::new();
        let mut scanned_sources = 0u32;

        if let Some(content) = request.content.as_ref().filter(|c| !c.trim().is_empty()) {
            scanned_sources += 1;
            detected.extend(SecurityScanner::scan(content, Some("uploaded-content")));
        }

        if request.scan_logs {
            let logs = sqlx::query_as::<_, LogScanRow>(
                r#"
                SELECT id::text AS resource, message
                FROM logs
                WHERE project_id = $1
                  AND level IN ('error', 'warn', 'fatal')
                ORDER BY timestamp DESC
                LIMIT 200
                "#,
            )
            .bind(project_id)
            .fetch_all(self.pool)
            .await?;

            scanned_sources += logs.len() as u32;
            for log in logs {
                let mut log_findings =
                    SecurityScanner::scan(&log.message, Some(&format!("log:{}", log.resource)));
                detected.append(&mut log_findings);
            }
        }

        let mut persisted = Vec::new();
        for finding in dedupe_findings(detected) {
            let row = insert_finding(self.pool, project_id, finding).await?;
            self.publish_finding_event(project_id, &row);
            persisted.push(row);
        }

        Ok(ScanResult {
            findings: persisted,
            scanned_sources,
        })
    }
}

fn dedupe_findings(findings: Vec<DetectedFinding>) -> Vec<DetectedFinding> {
    let mut unique = Vec::new();
    for finding in findings {
        if !unique.iter().any(|existing: &DetectedFinding| {
            existing.title == finding.title && existing.resource == finding.resource
        }) {
            unique.push(finding);
        }
    }
    unique
}

async fn insert_finding(
    pool: &PgPool,
    project_id: Uuid,
    finding: DetectedFinding,
) -> Result<SecurityFinding, AppError> {
    let row = sqlx::query_as::<_, FindingRow>(
        r#"
        INSERT INTO security_findings (
            project_id, finding_type, severity, title, description, resource
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, project_id, finding_type, severity, title, description, resource, detected_at
        "#,
    )
    .bind(project_id)
    .bind(finding.finding_type.as_str())
    .bind(finding.severity.as_str())
    .bind(finding.title)
    .bind(SecurityScanner::redact(&finding.description))
    .bind(finding.resource)
    .fetch_one(pool)
    .await?;

    Ok(row.into_finding())
}

impl SecurityService<'_> {
    fn publish_finding_event(&self, project_id: Uuid, finding: &SecurityFinding) {
        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::SecurityFinding {
                finding_id: finding.id,
                severity: finding.severity.as_str().to_string(),
                title: finding.title.clone(),
            },
            timestamp: Utc::now(),
        };
        let _ = self.events.publish(envelope);
    }
}

#[derive(sqlx::FromRow)]
struct FindingRow {
    id: Uuid,
    project_id: Uuid,
    finding_type: String,
    severity: String,
    title: String,
    description: String,
    resource: Option<String>,
    detected_at: DateTime<Utc>,
}

impl FindingRow {
    fn into_finding(self) -> SecurityFinding {
        SecurityFinding {
            id: self.id,
            project_id: self.project_id,
            finding_type: FindingType::parse(&self.finding_type),
            severity: Severity::parse(&self.severity),
            title: self.title,
            description: self.description,
            resource: self.resource,
            detected_at: self.detected_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LogScanRow {
    resource: String,
    message: String,
}
