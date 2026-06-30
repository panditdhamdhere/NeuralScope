-- Spans table for distributed trace details

CREATE TABLE spans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    trace_id VARCHAR(64) NOT NULL,
    span_id VARCHAR(64) NOT NULL,
    parent_span_id VARCHAR(64),
    service VARCHAR(255) NOT NULL,
    operation VARCHAR(255) NOT NULL,
    duration_ms DOUBLE PRECISION NOT NULL,
    status VARCHAR(10) NOT NULL DEFAULT 'ok',
    attributes JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_spans_project_trace ON spans (project_id, trace_id);
CREATE INDEX idx_spans_project_started ON spans (project_id, started_at DESC);
CREATE UNIQUE INDEX idx_spans_project_span ON spans (project_id, trace_id, span_id);
CREATE UNIQUE INDEX idx_traces_project_trace ON traces (project_id, trace_id);
