# NeuralScope

**An AI-powered developer observability platform built with Rust.**

Ask questions about your application in natural language. NeuralScope gathers context from logs, metrics, traces, Git history, network traffic, and security events before answering.

```
"Why is my API slow?"
"Which deployment introduced this bug?"
"What service is causing high CPU?"
"Explain this stack trace."
"Generate an incident report."
```

## Features

- **Unified Observability** — Logs, metrics, traces, network, and system metrics in one place
- **AI Chat** — Natural-language queries with tool-augmented retrieval (RAG)
- **Real-Time Streaming** — Live telemetry via WebSockets
- **Network Visualization** — Interactive connection graphs with React Flow
- **Architecture View** — Auto-generated service dependency diagrams
- **Security Scanning** — Secret detection, config audit, vulnerability reports
- **Incident Reports** — AI-generated root cause analysis and timelines

## Tech Stack

| Layer | Technologies |
|-------|-------------|
| Backend | Rust, Axum, Tokio, SQLx, PostgreSQL, Redis |
| Frontend | Next.js, React 19, TypeScript, Tailwind CSS v4, shadcn/ui |
| AI | Gemini (default), Groq, OpenRouter, Ollama |
| Vector DB | Qdrant |
| Auth | Better Auth |
| Infra | Docker, Docker Compose, Kubernetes |

## Monorepo Structure

```
apps/
  web/       → Next.js frontend
  server/    → Rust Axum backend
packages/
  ui/        → Shared component library
  shared/    → Cross-platform types and schemas
  config/    → Shared tooling configuration
infra/
  docker/    → Container definitions
  k8s/       → Kubernetes manifests
```

## Quick Start

### Prerequisites

- Rust 1.88+ (stable)
- Node.js 22+
- Docker & Docker Compose

### Option A — Full stack with Docker

```bash
cp .env.example .env

# Build and start PostgreSQL, Redis, Qdrant, API server, and web UI
npm run docker:up

# Or build images first
npm run docker:build && docker compose up -d
```

| Service | URL |
|---------|-----|
| Web UI | http://localhost:3000 |
| API | http://localhost:8080 |
| PostgreSQL | localhost:5432 |
| Redis | localhost:6379 |
| Qdrant | http://localhost:6333 |

### Option B — Local development

```bash
# Start infrastructure only
docker compose up -d postgres redis qdrant

# Copy environment variables
cp .env.example .env

# Run database migrations and start the API server
npm run dev:server

# In another terminal — install deps and start the frontend
npm ci
npm run dev:web
```

The server starts on `http://localhost:8080`:

| Endpoint | Description |
|----------|-------------|
| `GET /health` | Liveness probe (always returns 200) |
| `GET /ready` | Readiness probe (checks PostgreSQL + Redis) |
| `GET /api/v1/status` | Detailed status with uptime and dependency latency |
| `POST /api/v1/auth/register` | Register with email and password |
| `POST /api/v1/auth/login` | Login and receive session cookie |
| `GET /api/v1/auth/me` | Current authenticated user |
| `GET /api/v1/projects` | List projects (authenticated) |
| `POST /api/v1/api-keys` | Create programmatic API key |
| `POST /api/v1/projects/:id/logs` | Ingest a structured log entry |
| `POST /api/v1/projects/:id/logs/batch` | Batch ingest up to 1000 logs |
| `GET /api/v1/projects/:id/logs` | Search and filter logs |
| `WS /ws?project_id=:id&token=:token` | Real-time log event stream |
| `POST /api/v1/projects/:id/metrics` | Ingest metric sample |
| `GET /api/v1/projects/:id/metrics` | Query metric time-series |
| `POST /api/v1/projects/:id/traces` | Ingest distributed trace with spans |
| `GET /api/v1/projects/:id/traces` | List traces |
| `POST /api/v1/projects/:id/chat/completions` | AI chat with observability tool calling |
| `GET /api/v1/projects/:id/chat/conversations` | List chat threads |
| `GET /api/v1/projects/:id/chat/conversations/:id/messages` | Load conversation messages |
| `POST /api/v1/projects/:id/network/events` | Ingest network connection event |
| `GET /api/v1/projects/:id/network/graph` | Aggregated network graph for React Flow |
| `GET /api/v1/projects/:id/architecture/graph` | Service dependency graph |
| `POST /api/v1/projects/:id/architecture/regenerate` | Rebuild architecture from network + traces |
| `GET /api/v1/projects/:id/security/findings` | List security findings |
| `POST /api/v1/projects/:id/security/scan` | Scan content and logs for secrets |
| `GET /api/v1/projects/:id/incidents` | List incident reports |
| `POST /api/v1/projects/:id/incidents/generate` | Generate incident report from telemetry |
| `GET /api/v1/projects/:id/overview` | Dashboard aggregates (24h counts, recent logs, metrics) |
| `POST /api/v1/projects/:id/vectors/index` | Index code/docs for semantic search (RAG) |
| `POST /api/v1/projects/:id/vectors/search` | Semantic vector search |
| `GET /api/v1/projects/:id/vectors/status` | Embedding provider + Qdrant status |
| `GET /api/v1/projects/:id/git/commits` | List Git commits |
| `POST /api/v1/projects/:id/git/commits` | Ingest a commit record |
| `GET /api/v1/projects/:id/git/deployments` | List deployments with commit correlation |
| `POST /api/v1/projects/:id/git/deployments` | Record a deployment event |

Set `GEMINI_API_KEY` (or `GROQ_API_KEY` / `OPENROUTER_API_KEY` with `AI_DEFAULT_PROVIDER`) to enable chat.

### Kubernetes

```bash
# 1. Copy and edit secrets (never commit real values)
cp infra/k8s/secret.yaml.example infra/k8s/secret.yaml

# 2. Edit infra/k8s/configmap.yaml (domain, CORS origins)

# 3. Run migrations once, then deploy
kubectl apply -f infra/k8s/secret.yaml
kubectl apply -k infra/k8s/
kubectl apply -f infra/k8s/migration-job.yaml
kubectl wait --for=condition=complete job/neuralscope-migrate -n neuralscope --timeout=120s
```

Build web images with production URLs baked in:

```bash
docker build -f infra/docker/Dockerfile.web \
  --build-arg NEXT_PUBLIC_API_URL=https://app.yourdomain.com \
  --build-arg NEXT_PUBLIC_WS_URL=wss://app.yourdomain.com/ws \
  --build-arg NEXT_PUBLIC_APP_URL=https://app.yourdomain.com \
  -t neuralscope/web:latest .
```

### Production deployment

See [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md) for the full production checklist, Docker Compose overlay, Kubernetes steps, and GHCR image workflow.

Quick production start:

```bash
# Run migrations once
docker compose run --rm server neuralscope-server --migrate-only

# Start with production settings (no auto-migrate on server)
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Production hardening

| Control | Env / config |
|---------|----------------|
| Secret validation | `APP_ENV=production` rejects weak `BETTER_AUTH_SECRET` |
| CORS allowlist | `CORS_ALLOWED_ORIGINS=https://app.yourdomain.com` |
| Rate limiting | `RATE_LIMIT_PER_MINUTE=120`, `RATE_LIMIT_BURST=30` |
| RBAC | Viewers read-only; owner/admin can write |
| Auth | Better Auth only in production (Rust `/auth/login` disabled) |
| Dashboard gate | Next.js middleware redirects unauthenticated users |
| Migrations | `RUN_MIGRATIONS=false` on server pods; use `--migrate-only` Job |

### CI

GitHub Actions runs Rust tests (with PostgreSQL + Redis + Qdrant), frontend typecheck/lint/build, Docker image builds, and Playwright E2E smoke tests on every PR. Pushes to `main` trigger CD to publish container images to GHCR.

Run E2E tests locally (requires stack running on :3000 and :8080):

```bash
docker compose up -d
npm run test:e2e
```

Run integration tests against live infrastructure:

```bash
docker compose up -d
cd apps/server && cargo test --test integration_test -- --ignored
```

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for the complete system design, data flows, and module responsibilities.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT
