# CLAUDE.md

## Project Overview

ShipSecure — security scanning SaaS for vibe-coded apps. Rust/Axum backend, Next.js 16 frontend, PostgreSQL, Clerk auth. Live at https://shipsecure.ai.

## Production Infrastructure

### Server
- DigitalOcean droplet at `shipsecure.ai`
- SSH: `deploy@shipsecure.ai` on port `2222`
- App directory: `/opt/shipsecure/`
- Managed PostgreSQL (DigitalOcean, no local DB)
- Nginx reverse proxy with Let's Encrypt SSL

### How Deployment Works

```
push to main → GitHub Actions builds Docker images → pushes to GHCR →
scp compose files + systemd service to server → systemctl restart shipsecure
```

Key files:
- `docker-compose.prod.yml` — **standalone** production compose (NOT an override of docker-compose.yml)
- `docker-compose.yml` — local development only, never used in production
- `deploy/shipsecure.service` — systemd unit that manages Docker Compose
- `deploy/setup-production.sh` — full server setup/reset script
- `.github/workflows/build-push.yml` — CI/CD build and deploy
- `.github/workflows/ci.yml` — tests (unit + E2E)

### Critical Rules

- **docker-compose.prod.yml is standalone** — it does NOT extend or merge with docker-compose.yml. This is intentional. Docker Compose merge behavior is unreliable (duplicate ports, build directives leaking, depends_on to disabled services).
- **systemd manages Docker on the server** — never run `docker compose up/down` directly in CI. Use `sudo systemctl restart shipsecure`. The systemd service handles pull + start.
- **HOSTNAME=0.0.0.0 is required for frontend** — Next.js uses the HOSTNAME env var to bind. Docker sets HOSTNAME to the container ID which can't be resolved. Without this, the frontend crashes with `getaddrinfo EAI_AGAIN`.
- **All env vars must be explicit in docker-compose.prod.yml** — do NOT rely on `env_file:` from the dev compose file. Every variable the backend or frontend needs must be listed.

### Server .env Variables

Required in `/opt/shipsecure/.env`:

| Variable | Service | Description |
|----------|---------|-------------|
| `DATABASE_URL` | backend | PostgreSQL connection string |
| `PORT` | backend | Backend HTTP port (default: 3000) |
| `SHIPSECURE_BASE_URL` | backend | Base URL for email links (https://shipsecure.ai) |
| `FRONTEND_URL` | backend | Frontend URL for CORS (https://shipsecure.ai) |
| `MAX_CONCURRENT_SCANS` | backend | Max parallel scans (default: 10) |
| `CLERK_JWKS_URL` | backend | Clerk JWKS endpoint for JWT verification |
| `CLERK_SECRET_KEY` | frontend | Clerk secret key for middleware |
| `NEXT_PUBLIC_BACKEND_URL` | frontend | Backend URL for browser requests |
| `RESEND_API_KEY` | backend | Email delivery (optional) |
| `RUST_LOG` | backend | Log level filter (optional) |
| `LOG_FORMAT` | backend | `json` for structured logging (optional) |
| `SHUTDOWN_TIMEOUT` | backend | Graceful shutdown seconds (optional, default: 90) |

When adding a new required env var to the backend or frontend:
1. Add it to `docker-compose.prod.yml` environment section
2. Add it to `.env.example` with documentation
3. Add it to `deploy/setup-production.sh` validation list
4. Update this table

### Server Reset

If production is broken beyond repair:
```bash
scp -P 2222 deploy/setup-production.sh deploy@shipsecure.ai:/opt/shipsecure/
ssh -p 2222 deploy@shipsecure.ai 'bash /opt/shipsecure/setup-production.sh'
```

### GitHub Secrets

| Secret | Used By | Purpose |
|--------|---------|---------|
| `DEPLOY_SSH_PRIVATE_KEY` | build-push.yml | SSH to production server |
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | build-push.yml | Baked into frontend Docker image at build time |
| `CLERK_SECRET_KEY` | ci.yml | E2E test runner |

### Docker Images

- `ghcr.io/trustedge-labs/shipsecure-backend:latest`
- `ghcr.io/trustedge-labs/shipsecure-frontend:latest`
- Tagged with both `latest` and commit SHA

## Development

### Key Files
- `frontend/proxy.ts` — Clerk middleware (Next.js 16 uses `proxy.ts`, not `middleware.ts`)
- `frontend/vitest.setup.ts` — global test mocks (next/navigation, next/image, @clerk/nextjs)
- `frontend/__tests__/helpers/test-utils.tsx` — `renderWithProviders()` wrapper
- `frontend/app/layout.tsx` — root layout wraps everything in `<ClerkProvider>`

### Testing
- Vitest (unit) + Playwright (E2E) + MSW for API mocking
- CI: `ci.yml` runs unit-tests then e2e-tests sequentially
- Coverage: 80/80/75 thresholds, scoped to `components/**`
- E2E runs on port 3001 against production builds

### Branch Protection
- `main` branch is protected: no force pushes, 2 required status checks
- Admin bypass disabled
