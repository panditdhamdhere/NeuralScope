# Contributing to NeuralScope

Thank you for your interest in contributing to NeuralScope. This guide covers development setup, conventions, and the contribution workflow.

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/neuralscope/neuralscope.git
   cd neuralscope
   ```

2. **Start infrastructure**
   ```bash
   docker compose up -d
   ```

3. **Install dependencies**
   ```bash
   npm install          # Root + workspace packages
   cd apps/server && cargo build
   ```

4. **Environment variables**
   Copy `.env.example` to `.env` and fill in required values.

## Code Conventions

### Rust (Backend)

- Follow Clean Architecture: `domain → application → infrastructure → presentation`
- Use typed errors with `thiserror`; never `unwrap()` in production paths
- Async everywhere; use `tokio` for concurrency
- Traits for dependency injection and testability
- Repository pattern for data access
- Run `cargo fmt` and `cargo clippy` before committing

### TypeScript (Frontend)

- Feature-sliced design under `features/`
- Server state via TanStack Query; client state via Zustand
- Strict TypeScript; no `any`
- Components in `packages/ui` when shared across features
- Run `npm run lint` and `npm run typecheck` before committing

### Commits

Use conventional commits:

```
feat(logs): add structured log ingestion endpoint
fix(chat): resolve streaming disconnect on timeout
docs: update architecture diagram
```

## Pull Request Process

1. Fork and create a feature branch from `main`
2. Write tests for new functionality
3. Ensure CI passes (lint, test, build)
4. Update documentation if behavior changes
5. Open a PR with a clear description and test plan

## Module Ownership

When adding a new feature, place code in the appropriate module:

| Change type | Location |
|-------------|----------|
| New API endpoint | `apps/server/src/<module>/presentation/` |
| Business logic | `apps/server/src/<module>/application/` |
| Database query | `apps/server/src/<module>/infrastructure/` |
| UI page | `apps/web/app/(dashboard)/<feature>/` |
| Shared component | `packages/ui/src/` |
| Shared type | `packages/shared/src/` |

## Questions

Open a GitHub Discussion or issue for architectural questions before large changes.
