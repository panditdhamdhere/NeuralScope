-- Architecture graph persistence

CREATE TABLE architecture_nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    service_type VARCHAR(50) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    position_x DOUBLE PRECISION NOT NULL DEFAULT 0,
    position_y DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, name)
);

CREATE TABLE architecture_edges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    source_node_id UUID NOT NULL REFERENCES architecture_nodes(id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES architecture_nodes(id) ON DELETE CASCADE,
    protocol VARCHAR(20),
    avg_latency_ms DOUBLE PRECISION,
    request_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_id, source_node_id, target_node_id)
);

CREATE INDEX idx_architecture_nodes_project ON architecture_nodes (project_id);
CREATE INDEX idx_architecture_edges_project ON architecture_edges (project_id);
