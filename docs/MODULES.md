# NeuralScope Module Reference

This document explains why each module exists and how they interact.

## Backend Modules (`apps/server/src/`)

### `api` — API Gateway
**Why:** Single entry point for all HTTP traffic. Composes routers from feature modules, applies middleware (CORS, tracing, auth, rate limiting), and exposes health/readiness endpoints for orchestrators.

### `ai` — AI Orchestration
**Why:** The core differentiator. Abstracts LLM providers behind a trait, registers observability tools the LLM can call, and runs the multi-turn tool-calling loop before synthesizing answers. Designed to be reusable as MCP tools in the future.

**Layers:**
- `domain` — `LlmProvider` trait, `ToolDefinition`, completion types
- `application` — `AiOrchestrator`, `ToolRegistry`
- `infrastructure` — Gemini, Groq, OpenRouter, Ollama implementations
- `presentation` — AI-related HTTP endpoints

### `auth` — Authentication & Authorization
**Why:** Multi-tenant security. Integrates Better Auth for session management, API keys for programmatic access, and project-scoped RBAC (owner/admin/viewer).

### `logs` — Log Management
**Why:** Structured log ingestion, indexing, and search. Powers "Find today's errors" and "Summarize logs" AI queries. Streams new entries via the event bus.

### `metrics` — Metrics Collection
**Why:** Time-series data for CPU, memory, disk, HTTP latency, and custom metrics. Powers "Which endpoint consumes the most CPU?" and dashboard charts.

### `traces` — Distributed Tracing
**Why:** OpenTelemetry trace ingestion and span queries. Enables flame graphs, latency analysis, and "Why is my API slow?" investigations.

### `network` — Network Traffic
**Why:** Tracks connections between services and external endpoints. Feeds the React Flow network visualization graph.

### `git` — Git History
**Why:** Correlates commits and deployments with observability anomalies. Powers "Which deployment introduced this bug?" and "What changed after deployment?"

### `architecture` — Service Dependency Graph
**Why:** Auto-generates and stores service topology. Rendered as an interactive React Flow diagram showing Browser → API Gateway → Auth → Users → Redis → Postgres.

### `security` — Security Scanning
**Why:** Detects secrets, exposed ports, weak configs, and dependency vulnerabilities. Generates AI explanations for each finding.

### `chat` — AI Chat
**Why:** Manages conversation threads, assembles context from tool results, and streams responses to the frontend. The user-facing AI interface.

### `vector` — Vector Search / RAG
**Why:** Embeds logs, code, and docs into Qdrant for semantic search. Powers `search_codebase` and `search_docs` AI tools.

### `events` — Real-Time Event Bus
**Why:** Broadcasts telemetry events to WebSocket clients. Decouples ingestion from real-time UI updates via Redis PubSub.

### `db` — Database Layer
**Why:** Shared connection pool management, migration runner, and repository utilities used across all feature modules.

### `common` — Cross-Cutting Concerns
**Why:** Application config, unified error types, and shared utilities. Prevents duplication across modules.

---

## Frontend Structure (`apps/web/`)

### `app/(dashboard)/` — Dashboard Pages
One route per observability view: Overview, Logs, Metrics, Traces, Chat, Git, Architecture, Network, Security, Incidents, Settings.

### `components/` — Shared UI
Layout components (sidebar, header), not tied to a specific feature.

### `features/` — Feature-Sliced Modules
Each observability domain gets its own directory with components, hooks, and local state. Keeps pages thin.

### `services/` — API Layer
TanStack Query hooks and fetch functions. Single source of truth for server communication.

### `hooks/` — Shared React Hooks
WebSocket connection, debounce, media queries, etc.

### `lib/` — Utilities
API constants, `cn()` helper, formatters.

### `types/` — Frontend Types
Re-exports from `@neuralscope/shared` plus UI-specific types.

---

## Shared Packages

### `@neuralscope/shared`
Cross-platform TypeScript types and constants shared between frontend and API contract documentation.

### `@neuralscope/ui`
Shared shadcn/ui component library. Ensures consistent design across all dashboard views.

### `@neuralscope/config`
Shared ESLint, TypeScript, and Tailwind configuration. Single source of truth for tooling.

---

## Data Flow Summary

```
Telemetry → Ingest API → PostgreSQL
                        → Event Bus → Redis → WebSocket → Dashboard

User Question → Chat → AI Orchestrator → Tools → Data Sources
                                      → LLM → Streamed Answer

Logs/Code → Embedding → Qdrant → Semantic Search → AI Context
```
