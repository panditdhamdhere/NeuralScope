# Production Deployment Guide

This guide covers deploying NeuralScope to production with Docker Compose or Kubernetes.

## Pre-flight checklist

Before going live, verify every item:

- [ ] `APP_ENV=production`
- [ ] `BETTER_AUTH_SECRET` — 32+ chars (`openssl rand -base64 32`)
- [ ] `DATABASE_URL` — managed Postgres (not default `localhost` credentials)
- [ ] `CORS_ALLOWED_ORIGINS` — your exact frontend URL(s), comma-separated
- [ ] `BETTER_AUTH_URL` and `NEXT_PUBLIC_APP_URL` — public HTTPS URL
- [ ] `NEXT_PUBLIC_API_URL` / `NEXT_PUBLIC_WS_URL` — public API URL (baked into web image at build time)
- [ ] `RUN_MIGRATIONS=false` on long-running server/web pods (run migrations via Job)
- [ ] TLS terminated at ingress or reverse proxy
- [ ] `.env` never committed to git

## Render (recommended for first deploy)

See [docs/DEPLOY-RENDER.md](./DEPLOY-RENDER.md) for step-by-step deployment to Render with the included `render.yaml` blueprint (free tier by default).

## Docker Compose (single host)

### 1. Configure environment

```bash
cp .env.example .env
# Edit .env — set production values (see checklist above)
```

Example production `.env` keys:

```env
APP_ENV=production
RUN_MIGRATIONS=false
DATABASE_URL=postgres://user:pass@postgres:5432/neuralscope
BETTER_AUTH_SECRET=<openssl rand -base64 32>
BETTER_AUTH_URL=https://app.yourdomain.com
CORS_ALLOWED_ORIGINS=https://app.yourdomain.com
NEXT_PUBLIC_API_URL=https://app.yourdomain.com
NEXT_PUBLIC_WS_URL=wss://app.yourdomain.com/ws
NEXT_PUBLIC_APP_URL=https://app.yourdomain.com
```

### 2. Run migrations once

```bash
docker compose run --rm server neuralscope-server --migrate-only
```

### 3. Build web with production URLs

```bash
docker compose build \
  --build-arg NEXT_PUBLIC_API_URL=https://app.yourdomain.com \
  --build-arg NEXT_PUBLIC_WS_URL=wss://app.yourdomain.com/ws \
  --build-arg NEXT_PUBLIC_APP_URL=https://app.yourdomain.com \
  web
```

### 4. Start with production overlay

```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

Put nginx or Caddy in front for HTTPS. Route `/api`, `/health`, `/ready`, `/ws` to the server (port 8080) and `/` to the web app (port 3000), or use the K8s ingress pattern in `infra/k8s/ingress.yaml`.

## Kubernetes

### 1. Secrets

```bash
cp infra/k8s/secret.yaml.example infra/k8s/secret.yaml
# Edit with real DATABASE_URL, REDIS_URL, BETTER_AUTH_SECRET, GEMINI_API_KEY
kubectl apply -f infra/k8s/secret.yaml
```

### 2. ConfigMap

Edit `infra/k8s/configmap.yaml`:

- `CORS_ALLOWED_ORIGINS`
- `BETTER_AUTH_URL`
- `NEXT_PUBLIC_APP_URL`
- `NEXT_PUBLIC_API_URL` (if using env at runtime)

### 3. Container images

Images are published to GitHub Container Registry on every push to `main`:

```
ghcr.io/panditdhamdhere/neuralscope/server:<sha>
ghcr.io/panditdhamdhere/neuralscope/web:<sha>
```

Update `infra/k8s/server-deployment.yaml` and `web-deployment.yaml` image tags, or use Kustomize image transformers.

Build web images with production build args (required for client-side API URLs):

```bash
docker build -f infra/docker/Dockerfile.web \
  --build-arg NEXT_PUBLIC_API_URL=https://app.yourdomain.com \
  --build-arg NEXT_PUBLIC_WS_URL=wss://app.yourdomain.com/ws \
  --build-arg NEXT_PUBLIC_APP_URL=https://app.yourdomain.com \
  -t ghcr.io/panditdhamdhere/neuralscope/web:latest .
```

### 4. Deploy

```bash
kubectl apply -k infra/k8s/
kubectl apply -f infra/k8s/migration-job.yaml
kubectl wait --for=condition=complete job/neuralscope-migrate -n neuralscope --timeout=120s
```

## GitHub Actions CD

| Trigger | Action |
|---------|--------|
| Push to `main` | Build + push server and web images to GHCR |
| Tag `v*` | Same, with semver tags |

Configure repository **Variables** (Settings → Secrets and variables → Actions → Variables) for production web builds:

| Variable | Example |
|----------|---------|
| `NEXT_PUBLIC_API_URL` | `https://app.yourdomain.com` |
| `NEXT_PUBLIC_WS_URL` | `wss://app.yourdomain.com/ws` |
| `NEXT_PUBLIC_APP_URL` | `https://app.yourdomain.com` |

If unset, CD falls back to `http://localhost:8080` (development defaults).

## Health checks

| Service | Endpoint | Purpose |
|---------|----------|---------|
| Server | `GET /health` | Liveness |
| Server | `GET /ready` | Readiness (Postgres + Redis) |
| Web | `GET /api/health` | Liveness |

## Security controls (built-in)

| Control | Configuration |
|---------|---------------|
| Secret validation | `AppConfig::validate()` in staging/production |
| CORS allowlist | `CORS_ALLOWED_ORIGINS` |
| Rate limiting | `RATE_LIMIT_PER_MINUTE`, `RATE_LIMIT_BURST` |
| RBAC | Project roles: owner, admin, viewer |
| Auth | Better Auth (web); Rust `/auth/*` disabled outside development |
| Dashboard gate | Next.js middleware |
| Security headers | HSTS, X-Frame-Options, etc. in `next.config.ts` |

## Monitoring recommendations

- Scrape server `/health` and `/ready` for uptime alerts
- Ship JSON logs (`APP_ENV=production`) to your log aggregator
- Monitor Postgres connection pool and Redis latency via `GET /api/v1/status`
- Set up alerts on 5xx rate and rate-limit rejections

## Rollback

```bash
# Docker Compose
docker compose pull
docker compose up -d server web

# Kubernetes
kubectl rollout undo deployment/neuralscope-server -n neuralscope
kubectl rollout undo deployment/neuralscope-web -n neuralscope
```
