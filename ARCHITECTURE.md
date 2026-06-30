# NeuralScope Architecture

> An AI-powered developer observability platform built with Rust.

## Vision

NeuralScope unifies logs, metrics, traces, network traffic, Git history, and security signals into a single platform where developers ask natural-language questions and receive context-aware answers. The AI orchestration layer retrieves evidence from multiple data sources before synthesizing responses.

## High-Level System Diagram

```mermaid
flowchart TB
    subgraph Clients
        Web[Next.js Web App]
        CLI[Future CLI / MCP]
    end

    subgraph Gateway
        API[Axum REST API]
        WS[WebSocket Hub]
    end

    subgraph Core["Core Services (Rust)"]
        Auth[Auth Module]
        Chat[Chat / AI Orchestrator]
        Collectors[Telemetry Collectors]
        Events[Event Bus]
    end

    subgraph AI["AI Layer"]
        LLM[LLM Provider Abstraction]
        Tools[Tool Registry]
        RAG[RAG Pipeline]
        Embed[Embedding Providers]
    end

    subgraph Data["Data Layer"]
        PG[(PostgreSQL + pgvector)]
        Redis[(Redis Cache / PubSub)]
        Qdrant[(Qdrant Vector DB)]
    end

    subgraph External
        Gemini[Gemini API]
        Groq[Groq]
        Ollama[Ollama]
        Git[Git Repos]
    end

    Web --> API
    Web --> WS
    CLI --> API
    API --> Auth
    API --> Chat
    API --> Collectors
    WS --> Events
    Chat --> LLM
    Chat --> Tools
    Chat --> RAG
    RAG --> Qdrant
    RAG --> Embed
    Tools --> PG
    Tools --> Redis
    Collectors --> PG
    Collectors --> Events
    Events --> Redis
    Events --> WS
    LLM --> Gemini
    LLM --> Groq
    LLM --> Ollama
    Auth --> PG
```

## Monorepo Layout

```
NeuralScope/
├── apps/
│   ├── web/          # Next.js 15+ frontend (React 19)
│   └── server/       # Rust Axum backend
├── packages/
│   ├── ui/           # Shared shadcn/ui component library
│   ├── shared/       # Cross-platform types, schemas, constants
│   └── config/       # Shared ESLint, TypeScript, Tailwind configs
├── infra/
│   ├── docker/       # Dockerfiles
│   └── k8s/          # Kubernetes manifests
├── docs/             # Extended documentation
├── docker-compose.yml
└── .github/workflows/
```

## Clean Architecture (Backend)

Each feature module follows a layered structure inspired by DDD and Clean Architecture:

```
feature/
├── domain/       # Entities, value objects, domain errors, traits (ports)
├── application/  # Use cases, service orchestration, DTOs
├── infrastructure/ # Repository impls, external API clients
└── presentation/ # HTTP handlers, WebSocket handlers, request/response types
```

**Dependency rule:** `presentation → application → domain ← infrastructure`

Infrastructure implements domain traits. Application depends on abstractions, never concrete DB or HTTP clients.

### Backend Modules

| Module | Responsibility |
|--------|----------------|
| `api` | Router composition, middleware stack, OpenAPI, health checks |
| `ai` | LLM provider abstraction, tool registry, prompt templates, streaming |
| `auth` | Better Auth integration, sessions, API keys, RBAC |
| `logs` | Log ingestion, indexing, search, structured parsing |
| `metrics` | CPU, memory, disk, HTTP latency, custom metrics |
| `network` | Connection tracking, traffic analysis, graph data |
| `traces` | OpenTelemetry trace ingestion, span queries, flame graphs |
| `git` | Commit history, diffs, deployment correlation |
| `architecture` | Service dependency graph generation and storage |
| `security` | Secret scanning, config audit, vulnerability reports |
| `chat` | Conversation management, context assembly, streaming responses |
| `vector` | Embeddings, Qdrant indexing, semantic search |
| `events` | Real-time event bus, WebSocket fan-out |
| `db` | Connection pools, migrations, shared repository utilities |
| `common` | Config, errors, telemetry, utilities |

## AI Orchestration Flow

```mermaid
sequenceDiagram
    participant User
    participant Chat
    participant Orchestrator
    participant Tools
    participant LLM
    participant Data

    User->>Chat: "Why is my API slow?"
    Chat->>Orchestrator: Build context + system prompt
    Orchestrator->>LLM: Initial request with tool definitions
    LLM-->>Orchestrator: tool_call: search_metrics
    Orchestrator->>Tools: Execute search_metrics
    Tools->>Data: Query PostgreSQL / Redis
    Data-->>Tools: Metric samples
    Tools-->>Orchestrator: Tool result
    Orchestrator->>LLM: Continue with tool results
    LLM-->>Orchestrator: tool_call: search_traces
    Orchestrator->>Tools: Execute search_traces
    Tools->>Data: Query traces
    Data-->>Tools: Span data
    Tools-->>Orchestrator: Tool result
    Orchestrator->>LLM: Final synthesis request
    LLM-->>Chat: Streamed answer
    Chat-->>User: SSE / WebSocket stream
```

### AI Tools

| Tool | Data Source | Purpose |
|------|-------------|---------|
| `search_logs` | PostgreSQL | Full-text + semantic log search |
| `search_metrics` | PostgreSQL / Redis | Time-series metric queries |
| `search_traces` | PostgreSQL | Distributed trace lookup |
| `search_git` | PostgreSQL + Git | Commit/deployment correlation |
| `search_architecture` | PostgreSQL | Service dependency graph |
| `search_codebase` | Qdrant + Git | Semantic code search (RAG) |
| `search_docs` | Qdrant | Documentation retrieval |
| `search_security` | PostgreSQL | Security events and findings |
| `search_network` | PostgreSQL | Network connection analysis |

### LLM Provider Abstraction

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<CompletionStream, LlmError>;
    fn supports_tools(&self) -> bool;
}
```

Implementations: `GeminiProvider`, `GroqProvider`, `OpenRouterProvider`, `OllamaProvider`.

## Data Flow: Telemetry Ingestion

```mermaid
flowchart LR
    Agent[NeuralScope Agent / SDK] -->|HTTP/gRPC| Ingest[Ingest API]
    OTel[OpenTelemetry Collector] -->|OTLP| Ingest
    Docker[Docker Events] -->|Webhook| Ingest

    Ingest --> Validate[Validation Layer]
    Validate --> Batch[Batch Writer]
    Batch --> PG[(PostgreSQL)]
    Validate --> Events[Event Bus]
    Events --> Redis[(Redis PubSub)]
    Redis --> WS[WebSocket Clients]
    Validate --> Vector[Vector Pipeline]
    Vector --> Qdrant[(Qdrant)]
```

## Database Schema (Overview)

| Table | Purpose |
|-------|---------|
| `users` | User accounts, preferences |
| `projects` | Multi-tenant project/workspace isolation |
| `logs` | Structured log entries (partitioned by time) |
| `metrics` | Time-series metric data points |
| `traces` | Trace and span records |
| `incidents` | Detected or reported incidents |
| `chat_history` | AI conversation threads and messages |
| `vectors` | Embedding metadata (vectors stored in Qdrant) |
| `network_events` | Network connection records |
| `deployments` | Deployment events linked to Git commits |

PostgreSQL extensions: `pgvector` for in-DB similarity when Qdrant is unavailable.

## Frontend Architecture

```
apps/web/
├── app/                    # Next.js App Router
│   ├── (dashboard)/        # Authenticated layout group
│   │   ├── overview/
│   │   ├── logs/
│   │   ├── metrics/
│   │   ├── traces/
│   │   ├── chat/
│   │   ├── git/
│   │   ├── architecture/
│   │   ├── network/
│   │   ├── security/
│   │   ├── incidents/
│   │   └── settings/
│   ├── (auth)/             # Login / register
│   └── api/                # BFF routes (optional proxy)
├── components/             # Shared UI primitives
├── features/               # Feature-sliced modules
│   ├── logs/
│   ├── metrics/
│   ├── chat/
│   └── ...
├── hooks/                  # Shared React hooks
├── lib/                    # Utilities, API client
├── services/               # TanStack Query service layer
└── types/                  # Frontend-specific types
```

**State management:**
- Server state: TanStack Query
- Client UI state: Zustand
- Real-time: WebSocket hooks with automatic reconnection

**Design system:** Dark-first, glassmorphism cards, Framer Motion transitions, Recharts for metrics, React Flow for network/architecture graphs, Monaco for log/trace viewers.

## Real-Time Architecture

```
Collector → Event Bus (Tokio broadcast) → Redis PubSub → WebSocket Hub → Clients
```

Event types: `log.new`, `metric.sample`, `trace.complete`, `network.connection`, `incident.created`, `deployment.detected`.

## Security Model

- **Authentication:** Better Auth (session + API keys)
- **Authorization:** Project-scoped RBAC (owner, admin, viewer)
- **Data isolation:** Row-level security per project
- **Secrets:** Never logged; scanned content redacted before storage
- **API:** Rate limiting via Tower middleware, CORS restricted to configured origins

## Deployment Topology

```mermaid
flowchart TB
    subgraph Docker Compose [Development]
        WebDev[web:3000]
        ServerDev[server:8080]
        PGDev[postgres:5432]
        RedisDev[redis:6379]
        QdrantDev[qdrant:6333]
    end

    subgraph Kubernetes [Production]
        Ingress[Ingress / Load Balancer]
        WebPods[Web Deployment]
        ServerPods[Server Deployment]
        PGOp[PostgreSQL Operator]
        RedisOp[Redis Cluster]
        QdrantOp[Qdrant Cluster]
    end
```

## Performance Principles

1. **Async everywhere** — Tokio runtime, non-blocking I/O
2. **Connection pooling** — SQLx pool, Redis multiplexed connections
3. **Batch inserts** — Telemetry buffered and written in batches
4. **Caching** — Redis for hot queries, metric aggregations
5. **Streaming** — AI responses and log tailing via SSE/WebSocket
6. **Partitioning** — Time-based partitions for logs, metrics, traces

## Testing Strategy

| Layer | Approach |
|-------|----------|
| Domain | Pure unit tests, no I/O |
| Application | Mock repositories via trait objects |
| Infrastructure | Integration tests with testcontainers |
| API | HTTP tests with `axum-test` |
| Frontend | Vitest + React Testing Library |
| E2E | Playwright (future milestone) |

## Incremental Build Plan

| Milestone | Scope |
|-----------|-------|
| **M1** ✅ | Architecture, monorepo scaffold, docs |
| **M2** ✅ | Rust server foundation: config, DB, API router, health |
| **M3** ✅ | Auth module + PostgreSQL migrations |
| **M4** ✅ | Log ingestion + real-time WebSocket streaming |
| **M5** ✅ | Metrics + traces collectors |
| **M6** ✅ | AI orchestration layer + Gemini provider |
| **M7** ✅ | Chat UI + tool execution loop |
| **M8** ✅ | Network graph + architecture view |
| **M9** ✅ | Security scanner + incident reports |
| **M10** ✅ | Dashboard polish, CI/CD, K8s manifests |
| **M11** ✅ | Production hardening: CORS, rate limits, RBAC, auth middleware |

## Future: MCP Integration

NeuralScope will expose an MCP server allowing AI assistants (Cursor, Claude Desktop) to query observability data directly. The tool registry in the `ai` module is designed to be reusable as MCP tool definitions.
