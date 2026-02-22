# External Integrations

**Analysis Date:** 2026-02-21

## APIs & External Services

**Authentication & Identity:**
- Clerk - User authentication and session management
  - SDK/Client: `@clerk/nextjs` 6.37.5
  - Auth: `CLERK_SECRET_KEY` (backend), `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` (frontend)
  - JWKS URL: `CLERK_JWKS_URL` for JWT verification via `axum-jwt-auth`
  - Implementation: `src/api/auth.rs` — ClerkClaims extraction, `frontend/proxy.ts` — Clerk middleware

**Email Delivery:**
- Resend - Transactional email for scan completion notifications
  - SDK/Client: Direct HTTP API via `reqwest`
  - Auth: `RESEND_API_KEY` (optional, email feature disabled if missing)
  - Implementation: `src/email/mod.rs` — async email sending, `src/orchestrator/worker_pool.rs` — triggers on scan completion
  - Endpoint: `https://api.resend.com/emails`
  - Feature: Gracefully skips if API key not configured

**Security Scanning (External Binaries):**
- Nuclei (by ProjectDiscovery) - Vulnerability scanning
  - Binary: `/usr/local/bin/nuclei` (installed in Docker, configurable via `NUCLEI_BINARY_PATH`)
  - Implementation: `src/scanners/container.rs` — subprocess execution, template-based scanning
  - Config: `SHIPSECURE_TEMPLATES_DIR` points to custom or bundled templates in `/app/templates/nuclei`
  - Gracefully skips if binary unavailable

- testssl.sh - TLS/SSL security testing
  - Binary: `/usr/local/bin/testssl.sh` (git clone in Docker, configurable via `TESTSSL_BINARY_PATH`)
  - Implementation: `src/scanners/container.rs` — subprocess execution with 180s timeout
  - Gracefully skips if binary unavailable

## Data Storage

**Databases:**
- PostgreSQL 16
  - Connection: `DATABASE_URL` (format: `postgres://username:password@host:port/database`)
  - Client: SQLx 0.8.6 with runtime-tokio, TLS, UUID, and Chrono support
  - Migrations: SQLx managed migrations in `migrations/` directory (auto-run on startup)
  - Pool: Max 10 connections (configurable via PgPoolOptions)
  - Health check: Latency threshold configurable via `HEALTH_DB_LATENCY_THRESHOLD_MS` (default 200ms)

**File Storage:**
- Local filesystem only
  - Scanner templates: `templates/` directory (copied into Docker image)
  - Fonts: `fonts/` directory (for PDF report generation, if applicable)
  - No cloud storage integration (S3, GCS, etc.)

**Caching:**
- In-memory health cache (custom implementation)
  - Location: `src/api/health.rs` — HealthCache struct
  - Purpose: Track DB health status to reduce redundant database pings
- JWKS token cache (via axum-jwt-auth)
  - Caches Clerk public keys with background refresh
  - Prevents repeated requests to Clerk's JWKS endpoint

## Authentication & Identity

**Auth Provider:**
- Clerk (primary)
  - Implementation: JWT-based (RS256) via `axum-jwt-auth` RemoteJwksDecoder
  - Locations:
    - Backend: `src/api/auth.rs` (ClerkClaims, JWT validation)
    - Frontend: `frontend/proxy.ts` (Clerk middleware for protected routes)
    - Root layout: `frontend/app/layout.tsx` (ClerkProvider wrapper)
  - Protected routes: `/dashboard`, `/verify-domain` (via `createRouteMatcher` in proxy.ts)
  - User data: Clerk user ID stored as `clerk_user_id` in scans and verified_domains tables
  - Token validation: JWKS URL fetched from Clerk, cached with periodic refresh
  - Claims: `sub` (user ID), `exp`, `iat`, `nbf`, `azp`, `sid`

## Webhooks & Callbacks

**Incoming:**
- Clerk Webhooks - User lifecycle events
  - Endpoint: `POST /api/v1/webhooks/clerk`
  - Implementation: `src/api/webhooks.rs` — Svix signature verification
  - Signature secret: `CLERK_WEBHOOK_SIGNING_SECRET` (optional, required if webhook enabled)
  - Verification: Svix 1.x library for HMAC-SHA256 signature validation
  - Events handled: `user.created` (inserts row into users table)
  - Unknown events: Logged and ignored gracefully
  - Response: 204 No Content on success

**Outgoing:**
- None detected (pull-based integration only)

## Monitoring & Observability

**Metrics:**
- Prometheus-compatible metrics
  - Exporter: `metrics-exporter-prometheus` 0.17
  - Endpoint: `GET /metrics`
  - Implementation: `src/api/metrics.rs`
  - Metrics tracked: HTTP request counts, latencies, scan queue depth (via custom middleware in `src/metrics/`)

**Error Tracking:**
- Custom structured logging (no external service integration)
  - Framework: `tracing` 0.1 with `tracing-subscriber`
  - Output: JSON format (configurable via `LOG_FORMAT` env var, default plaintext for dev)
  - Log filtering: Controlled by `RUST_LOG` environment variable
  - Panic hook: `tracing-panic` for structured panic logging

**Logs:**
- Structured logging to stdout/stderr
  - Format: JSON (production) or plaintext (development)
  - Container logging: Docker json-file driver with 10MB max-size, 3-file rotation
  - Log levels: Configurable per module, defaults (debug in dev, info in release)

## CI/CD & Deployment

**Hosting:**
- Docker containers on Linux host
- Production: Self-hosted at `shipsecure.ai`
- Docker Compose for orchestration (dev and prod)

**Container Registry:**
- GitHub Container Registry (GHCR)
  - Backend image: `ghcr.io/trustedge-labs/shipsecure-backend:latest` and `:{git-sha}`
  - Frontend image: `ghcr.io/trustedge-labs/shipsecure-frontend:latest` and `:{git-sha}`
  - Authentication: GitHub GITHUB_TOKEN (via Actions)

**CI Pipeline:**
- GitHub Actions
  - Unit tests: Vitest with coverage (`npm run test:ci`), Node 22
  - E2E tests: Playwright on chromium, depends on unit tests passing
  - Build: Docker multi-stage builds
  - Push: GHCR automatic push on main branch commits
  - Deploy: SSH to `shipsecure.ai` port 2222 for file sync and systemd restart
  - Workflow files: `.github/workflows/ci.yml`, `.github/workflows/build-push.yml`

**Deployment Process:**
1. Commit to main branch triggers CI pipeline
2. Unit tests run (Node 22, npm ci)
3. E2E tests run (Playwright chromium, Next.js testmode on port 3001)
4. Docker images built (backend and frontend)
5. Images pushed to GHCR
6. Production deployment:
   - SCP `docker-compose.prod.yml` and `deploy/shipsecure.service` to `/opt/shipsecure`
   - Systemd daemon-reload and service restart
   - Docker Compose pulls latest images and restarts containers

**Secrets Management:**
- GitHub Secrets (used in Actions):
  - `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` - Frontend Clerk key
  - `CLERK_SECRET_KEY` - Backend Clerk key
  - `DEPLOY_SSH_PRIVATE_KEY` - SSH key for production access
  - All others sourced from `.env` on production host

## Environment Configuration

**Required env vars (all contexts):**
- `DATABASE_URL` - PostgreSQL connection string
- `CLERK_JWKS_URL` - Clerk public keys endpoint (format: `https://{instance}.clerk.accounts.dev/.well-known/jwks.json`)
- `PORT` - Server port (3000 for backend, 3001 for frontend in prod)
- `SHIPSECURE_BASE_URL` - Instance URL for email links (e.g., `https://shipsecure.ai`)
- `FRONTEND_URL` - Frontend URL for CORS allowlist (e.g., `https://shipsecure.ai`)
- `MAX_CONCURRENT_SCANS` - Concurrency limit (dev: 5, prod 8GB+: 10-20)

**Frontend env vars:**
- `NEXT_PUBLIC_BACKEND_URL` - Backend API URL (exposed to client)
- `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` - Clerk publishable key
- `BACKEND_URL` - Server-side backend URL (private)
- `CLERK_SECRET_KEY` - Clerk secret key (server-side)

**Optional env vars:**
- `RESEND_API_KEY` - Email feature (skipped if missing)
- `RUST_LOG` - Log level and filtering
- `LOG_FORMAT` - `json` or `text` (default: json in prod, text in dev)
- `SHUTDOWN_TIMEOUT` - Graceful shutdown timeout in seconds (default: 90)
- `HEALTH_DB_LATENCY_THRESHOLD_MS` - Health check threshold (default: 200ms)
- `NUCLEI_BINARY_PATH` - Nuclei binary location (auto-resolved if not set)
- `TESTSSL_BINARY_PATH` - testssl.sh binary location (auto-resolved if not set)
- `SHIPSECURE_TEMPLATES_DIR` - Custom Nuclei templates directory

**Development configuration:**
- Local PostgreSQL: Docker Compose with `postgres://shipsecure:shipsecure@localhost:5432/shipsecure_dev`
- Backend: `http://localhost:3000`
- Frontend: `http://localhost:3001`
- Clerk test keys from Clerk Dashboard

**Production configuration:**
- External PostgreSQL (password-protected, accessible only internally)
- Backend: `http://backend:3000` (container-to-container), exposed via reverse proxy
- Frontend: Built with production Clerk keys, backend URL set at runtime
- HTTPS via reverse proxy (Nginx), not in Docker containers

---

*Integration audit: 2026-02-21*
