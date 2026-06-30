//! Incident report generation and management.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::ai::domain::{ChatMessage, CompletionRequest, LlmProvider};
use crate::events::application::EventBus;
use crate::events::domain::{Event, EventEnvelope};
use crate::incidents::domain::{
    GenerateIncidentRequest, Incident, IncidentQuery, IncidentSeverity, IncidentStatus,
    TimelineEntry, TimelineEntryType, UpdateIncidentRequest,
};
use crate::AppError;

/// Creates and manages incident reports.
pub struct IncidentService<'a> {
    pool: &'a PgPool,
    events: &'a EventBus,
    ai_provider: Option<Arc<dyn LlmProvider>>,
}

impl<'a> IncidentService<'a> {
    #[must_use]
    pub fn new(
        pool: &'a PgPool,
        events: &'a EventBus,
        ai_provider: Option<Arc<dyn LlmProvider>>,
    ) -> Self {
        Self {
            pool,
            events,
            ai_provider,
        }
    }

    pub async fn list(
        &self,
        project_id: Uuid,
        query: IncidentQuery,
    ) -> Result<Vec<Incident>, AppError> {
        let limit = query.limit.clamp(1, 200);

        let rows = sqlx::query_as::<_, IncidentRow>(
            r#"
            SELECT id, project_id, title, severity, status, root_cause,
                   timeline, affected_services, suggested_fixes, created_at, resolved_at
            FROM incidents
            WHERE project_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(project_id)
        .bind(query.status.map(|s| s.as_str().to_string()))
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows.into_iter().map(IncidentRow::into_incident).collect())
    }

    pub async fn get(&self, project_id: Uuid, incident_id: Uuid) -> Result<Incident, AppError> {
        let row = sqlx::query_as::<_, IncidentRow>(
            r#"
            SELECT id, project_id, title, severity, status, root_cause,
                   timeline, affected_services, suggested_fixes, created_at, resolved_at
            FROM incidents
            WHERE project_id = $1 AND id = $2
            "#,
        )
        .bind(project_id)
        .bind(incident_id)
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Incident not found".into()))?;

        Ok(row.into_incident())
    }

    pub async fn generate(
        &self,
        project_id: Uuid,
        request: GenerateIncidentRequest,
    ) -> Result<Incident, AppError> {
        let context = gather_context(self.pool, project_id).await?;

        if context.timeline.is_empty() {
            return Err(AppError::Validation(
                "Not enough telemetry to generate an incident report. Ingest logs, traces, or run a security scan first.".into(),
            ));
        }

        let severity = context.severity;
        let title = request.title.unwrap_or_else(|| context.default_title());
        let mut root_cause = build_root_cause(&context);
        let mut suggested_fixes = build_suggested_fixes(&context);

        if request.use_ai {
            if let Some(provider) = &self.ai_provider {
                if let Ok(ai_summary) = generate_ai_summary(provider, &context).await {
                    root_cause = ai_summary.root_cause;
                    if !ai_summary.suggested_fixes.is_empty() {
                        suggested_fixes = ai_summary.suggested_fixes;
                    }
                }
            }
        }

        let incident = insert_incident(
            self.pool,
            project_id,
            &title,
            severity,
            &root_cause,
            &context.timeline,
            &context.affected_services,
            &suggested_fixes,
        )
        .await?;

        self.publish_incident_event(project_id, &incident);
        Ok(incident)
    }

    pub async fn update(
        &self,
        project_id: Uuid,
        incident_id: Uuid,
        request: UpdateIncidentRequest,
    ) -> Result<Incident, AppError> {
        let status = request
            .status
            .ok_or_else(|| AppError::Validation("Status is required".into()))?;

        let resolved_at = if status == IncidentStatus::Resolved {
            Some(Utc::now())
        } else {
            None
        };

        let row = sqlx::query_as::<_, IncidentRow>(
            r#"
            UPDATE incidents
            SET status = $3,
                resolved_at = COALESCE($4, resolved_at)
            WHERE project_id = $1 AND id = $2
            RETURNING id, project_id, title, severity, status, root_cause,
                      timeline, affected_services, suggested_fixes, created_at, resolved_at
            "#,
        )
        .bind(project_id)
        .bind(incident_id)
        .bind(status.as_str())
        .bind(resolved_at)
        .fetch_optional(self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Incident not found".into()))?;

        Ok(row.into_incident())
    }

    fn publish_incident_event(&self, project_id: Uuid, incident: &Incident) {
        let envelope = EventEnvelope {
            id: Uuid::new_v4(),
            project_id,
            event: Event::IncidentCreated {
                incident_id: incident.id,
                severity: incident.severity.as_str().to_string(),
            },
            timestamp: Utc::now(),
        };
        let _ = self.events.publish(envelope);
    }
}

struct IncidentContext {
    timeline: Vec<TimelineEntry>,
    affected_services: Vec<String>,
    severity: IncidentSeverity,
    error_count: usize,
    failed_traces: usize,
    critical_findings: usize,
}

impl IncidentContext {
    fn default_title(&self) -> String {
        if self.critical_findings > 0 {
            "Critical security findings detected".into()
        } else if self.failed_traces > 0 {
            format!("Service degradation — {} failed traces", self.failed_traces)
        } else {
            format!("Elevated error rate — {} error events", self.error_count)
        }
    }
}

struct AiSummary {
    root_cause: String,
    suggested_fixes: Vec<String>,
}

async fn gather_context(pool: &PgPool, project_id: Uuid) -> Result<IncidentContext, AppError> {
    let mut timeline = Vec::new();
    let mut affected_services = Vec::new();
    let mut severity = IncidentSeverity::Low;

    let logs = sqlx::query_as::<_, LogContextRow>(
        r#"
        SELECT timestamp, level, message, service
        FROM logs
        WHERE project_id = $1 AND level IN ('error', 'fatal', 'warn')
        ORDER BY timestamp DESC
        LIMIT 30
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let error_count = logs.iter().filter(|l| l.level == "error" || l.level == "fatal").count();
    if error_count > 0 {
        severity = severity.max(IncidentSeverity::High);
    }

    for log in logs {
        if let Some(service) = &log.service {
            if !affected_services.contains(service) {
                affected_services.push(service.clone());
            }
        }
        timeline.push(TimelineEntry {
            timestamp: log.timestamp,
            entry_type: TimelineEntryType::Log,
            title: format!("{} log event", log.level),
            detail: log.message,
        });
    }

    let traces = sqlx::query_as::<_, TraceContextRow>(
        r#"
        SELECT started_at, trace_id, root_service, duration_ms, status
        FROM traces
        WHERE project_id = $1 AND status = 'error'
        ORDER BY started_at DESC
        LIMIT 15
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let failed_traces = traces.len();
    if failed_traces > 0 {
        severity = severity.max(IncidentSeverity::High);
    }

    for trace in traces {
        if !affected_services.contains(&trace.root_service) {
            affected_services.push(trace.root_service.clone());
        }
        timeline.push(TimelineEntry {
            timestamp: trace.started_at,
            entry_type: TimelineEntryType::Trace,
            title: format!("Failed trace {}", trace.trace_id),
            detail: format!(
                "{} took {:.0}ms with status {}",
                trace.root_service, trace.duration_ms, trace.status
            ),
        });
    }

    let findings = sqlx::query_as::<_, FindingContextRow>(
        r#"
        SELECT detected_at, severity, title, description
        FROM security_findings
        WHERE project_id = $1 AND severity IN ('high', 'critical')
        ORDER BY detected_at DESC
        LIMIT 10
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let critical_findings = findings
        .iter()
        .filter(|f| f.severity == "critical")
        .count();
    if critical_findings > 0 {
        severity = IncidentSeverity::Critical;
    } else if !findings.is_empty() {
        severity = severity.max(IncidentSeverity::High);
    }

    for finding in findings {
        timeline.push(TimelineEntry {
            timestamp: finding.detected_at,
            entry_type: TimelineEntryType::Finding,
            title: finding.title.clone(),
            detail: finding.description.clone(),
        });
    }

    timeline.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(IncidentContext {
        timeline,
        affected_services,
        severity,
        error_count,
        failed_traces,
        critical_findings,
    })
}

fn build_root_cause(context: &IncidentContext) -> String {
    let mut parts = Vec::new();

    if context.critical_findings > 0 {
        parts.push(format!(
            "{} critical security finding(s) require immediate attention.",
            context.critical_findings
        ));
    }
    if context.failed_traces > 0 {
        parts.push(format!(
            "{} distributed trace(s) failed, indicating service-level errors.",
            context.failed_traces
        ));
    }
    if context.error_count > 0 {
        parts.push(format!(
            "{} error/fatal log event(s) were recorded in the analysis window.",
            context.error_count
        ));
    }
    if !context.affected_services.is_empty() {
        parts.push(format!(
            "Affected services: {}.",
            context.affected_services.join(", ")
        ));
    }

    parts.join(" ")
}

fn build_suggested_fixes(context: &IncidentContext) -> Vec<String> {
    let mut fixes = Vec::new();

    if context.critical_findings > 0 {
        fixes.push("Rotate exposed credentials and remove secrets from configuration files.".into());
        fixes.push("Run a full security scan and verify no secrets are committed to git.".into());
    }
    if context.failed_traces > 0 {
        fixes.push("Inspect failed traces for downstream dependency timeouts.".into());
        fixes.push("Check recent deployments for regressions in affected services.".into());
    }
    if context.error_count > 0 {
        fixes.push("Review error logs for recurring stack traces or connection failures.".into());
    }
    if fixes.is_empty() {
        fixes.push("Monitor metrics and logs for continued anomalies.".into());
    }

    fixes
}

async fn generate_ai_summary(
    provider: &Arc<dyn LlmProvider>,
    context: &IncidentContext,
) -> Result<AiSummary, crate::ai::domain::LlmError> {
    let payload = json!({
        "timeline": context.timeline,
        "affectedServices": context.affected_services,
        "severity": context.severity.as_str(),
    });

    let response = provider
        .complete(CompletionRequest {
            messages: vec![
                ChatMessage::system(
                    "You are an SRE assistant. Given observability evidence, write a concise root cause analysis and 3 actionable remediation steps. Respond in JSON: {\"rootCause\":\"...\",\"suggestedFixes\":[\"...\"]}",
                ),
                ChatMessage::user(payload.to_string()),
            ],
            tools: None,
            model: None,
            temperature: Some(0.2),
            max_tokens: Some(1024),
        })
        .await?;

    let content = response.content.unwrap_or_default();
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
        let root_cause = parsed["rootCause"]
            .as_str()
            .or_else(|| parsed["root_cause"].as_str())
            .unwrap_or(&content)
            .to_string();
        let suggested_fixes = parsed["suggestedFixes"]
            .as_array()
            .or_else(|| parsed["suggested_fixes"].as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        return Ok(AiSummary {
            root_cause,
            suggested_fixes,
        });
    }

    Ok(AiSummary {
        root_cause: content,
        suggested_fixes: vec![],
    })
}

async fn insert_incident(
    pool: &PgPool,
    project_id: Uuid,
    title: &str,
    severity: IncidentSeverity,
    root_cause: &str,
    timeline: &[TimelineEntry],
    affected_services: &[String],
    suggested_fixes: &[String],
) -> Result<Incident, AppError> {
    let row = sqlx::query_as::<_, IncidentRow>(
        r#"
        INSERT INTO incidents (
            project_id, title, severity, status, root_cause,
            timeline, affected_services, suggested_fixes
        )
        VALUES ($1, $2, $3, 'open', $4, $5, $6, $7)
        RETURNING id, project_id, title, severity, status, root_cause,
                  timeline, affected_services, suggested_fixes, created_at, resolved_at
        "#,
    )
    .bind(project_id)
    .bind(title)
    .bind(severity.as_str())
    .bind(root_cause)
    .bind(json!(timeline))
    .bind(json!(affected_services))
    .bind(json!(suggested_fixes))
    .fetch_one(pool)
    .await?;

    Ok(row.into_incident())
}

#[derive(sqlx::FromRow)]
struct IncidentRow {
    id: Uuid,
    project_id: Uuid,
    title: String,
    severity: String,
    status: String,
    root_cause: Option<String>,
    timeline: serde_json::Value,
    affected_services: serde_json::Value,
    suggested_fixes: serde_json::Value,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
}

impl IncidentRow {
    fn into_incident(self) -> Incident {
        Incident {
            id: self.id,
            project_id: self.project_id,
            title: self.title,
            severity: IncidentSeverity::parse(&self.severity),
            status: IncidentStatus::parse(&self.status),
            root_cause: self.root_cause,
            timeline: serde_json::from_value(self.timeline).unwrap_or_default(),
            affected_services: serde_json::from_value(self.affected_services).unwrap_or_default(),
            suggested_fixes: serde_json::from_value(self.suggested_fixes).unwrap_or_default(),
            created_at: self.created_at,
            resolved_at: self.resolved_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LogContextRow {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
    service: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TraceContextRow {
    started_at: DateTime<Utc>,
    trace_id: String,
    root_service: String,
    duration_ms: f64,
    status: String,
}

#[derive(sqlx::FromRow)]
struct FindingContextRow {
    detected_at: DateTime<Utc>,
    severity: String,
    title: String,
    description: String,
}
