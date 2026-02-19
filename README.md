# ShipSecure

Security scanning platform for vibe-coded apps. Paste a URL, get an A-F security grade with actionable fixes — no security expertise required.

Built for developers shipping fast with AI code generators (Cursor, Bolt, Lovable, etc.) who need to know if their app is leaking secrets, missing headers, or exposing admin panels.

Live at [shipsecure.ai](https://shipsecure.ai)

## Why This Exists

- 45% of AI-generated code contains security flaws
- 86% of AI tools fail XSS defenses; 88% fail log injection tests
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys

## What It Scans

**Anonymous tier** (no signup — paste a URL and email):

- **Security headers** — CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- **TLS/SSL** — Certificate validity, cipher strength, protocol versions, known vulnerabilities (via SSL Labs API)
- **Exposed files** — `.env`, `.git`, `/debug`, `/admin`, source maps, API docs, `phpinfo.php`, `server-status`
- **Client-side secrets** — AWS keys, Stripe keys, Supabase/Firebase credentials, API tokens leaked in JavaScript bundles
- **Framework detection** — Auto-detects Next.js, Vite, React, SvelteKit, Nuxt
- **Platform detection** — Auto-detects Vercel, Netlify, Railway, Render, Supabase, Firebase
- Results: low/medium findings shown in full; high/critical findings gated behind signup

**Developer tier** (free signup):

- Everything above, plus full high/critical finding details with remediation
- Domain ownership verification via meta tag
- Deeper scanning (more JS files analyzed, longer timeouts)
- Scan history dashboard with expiry countdown and quota tracking
- Up to 5 scans per month, 30-day result retention

**Vibe-code intelligence** (both tiers):

- **Supabase** — RLS misconfigurations, exposed anon keys, unprotected PostgREST
- **Firebase** — Insecure security rules, exposed API keys, open Firestore
- **Vercel** — Environment variable leaks, exposed source maps
- **Netlify** — Function exposure, form handling issues
- **Railway/Render** — Debug endpoints, exposed metrics
- **Remediation playbooks** — Framework-specific copy-paste fixes

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Axum) |
| Frontend | Next.js 15 (App Router), React, Tailwind CSS |
| Database | PostgreSQL |
| Auth | Clerk (email/password, Google OAuth, GitHub OAuth) |
| Scanners | Nuclei (native binary), testssl.sh (native binary), custom probes |
| Observability | tracing (structured logging), metrics-exporter-prometheus, tower-http |
| Email | Resend API |
| CI/CD | GitHub Actions, GHCR, SSH deploy |
| Infrastructure | Ansible, Docker Compose, Nginx, Let's Encrypt, systemd |

## Getting Started

### Prerequisites

- Docker and Docker Compose (for full-stack development)
- OR for native development: Rust 1.88+, Node.js 20+, PostgreSQL 16, Nuclei, testssl.sh

### Quick Start with Docker

```bash
cp .env.example .env
# Edit .env with your configuration (all required variables must be set)
docker compose up
```

This starts PostgreSQL, the Rust backend (port 3000), and the Next.js frontend (port 3001).

### Local Development

```bash
# Start just the database
docker compose up db

# Backend (separate terminal)
cp .env.example .env
# Edit .env -- see .env.example for all required variables
cargo build
cargo run  # Runs on http://localhost:3000

# Frontend (separate terminal)
cd frontend
cp .env.example .env
# Edit .env -- set Clerk keys from https://dashboard.clerk.com
npm install
npm run dev  # Runs on http://localhost:3001
```

### Backend Configuration

Copy `.env.example` and set the following:

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `PORT` | Backend HTTP port | Yes |
| `SHIPSECURE_BASE_URL` | Base URL for email links | Yes |
| `FRONTEND_URL` | Frontend URL for CORS | Yes |
| `MAX_CONCURRENT_SCANS` | Maximum parallel scans | Yes |
| `CLERK_JWKS_URL` | Clerk JWKS endpoint for JWT verification | Yes |
| `LOG_FORMAT` | Log format: `json` for structured, unset for text | No |
| `RUST_LOG` | Log level filter override | No |
| `SHUTDOWN_TIMEOUT` | Graceful shutdown timeout in seconds (default: 90) | No |
| `HEALTH_DB_LATENCY_THRESHOLD_MS` | Health check DB latency threshold (default: 200) | No |
| `RESEND_API_KEY` | Resend API key for email delivery | No |
| `NUCLEI_BINARY_PATH` | Path to Nuclei binary | No |
| `TESTSSL_BINARY_PATH` | Path to testssl.sh binary | No |
| `SHIPSECURE_TEMPLATES_DIR` | Custom Nuclei templates directory | No |

### Frontend Configuration

Copy `frontend/.env.example` and set the following:

| Variable | Description | Required |
|----------|-------------|----------|
| `NEXT_PUBLIC_BACKEND_URL` | Backend API URL | Yes |
| `BACKEND_URL` | Backend URL (server-side) | Yes |
| `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` | Clerk publishable key | Yes |
| `CLERK_SECRET_KEY` | Clerk secret key | Yes |
| `NEXT_PUBLIC_CLERK_SIGN_IN_URL` | Sign-in page path (default: `/sign-in`) | No |
| `NEXT_PUBLIC_CLERK_SIGN_UP_URL` | Sign-up page path (default: `/sign-up`) | No |

## API

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `GET` | `/health` | No | Liveness check |
| `GET` | `/health/ready` | No | Readiness check (DB, scan capacity) |
| `GET` | `/metrics` | No (localhost) | Prometheus metrics |
| `POST` | `/api/v1/scans` | Optional | Submit a scan (auth upgrades tier) |
| `GET` | `/api/v1/scans/{id}` | No | Get scan status and findings |
| `GET` | `/api/v1/results/{token}` | Optional | Get results by capability token (auth ungates findings) |
| `GET` | `/api/v1/results/{token}/download` | Optional | Download results as markdown |
| `GET` | `/api/v1/quota` | Required | Get scan quota usage |
| `GET` | `/api/v1/stats/scan-count` | No | Get total scan count |
| `POST` | `/api/v1/webhooks/clerk` | Svix | Clerk user sync webhook |
| `POST` | `/api/v1/domains/verify-start` | Required | Start domain verification |
| `POST` | `/api/v1/domains/verify-confirm` | Required | Confirm domain verification |
| `POST` | `/api/v1/domains/verify-check` | Required | Check domain verification status |
| `GET` | `/api/v1/domains` | Required | List verified domains |
| `GET` | `/api/v1/users/me/scans` | Required | Get user's scan history |

## Project Structure

```
src/
  api/           # HTTP handlers (scans, results, auth, domains, users, webhooks, health, metrics)
  scanners/      # Security scanners (headers, TLS, secrets, vibecode, exposed files, detector)
  orchestrator/  # Scan job management with concurrency control and graceful shutdown
  models/        # Data structures (scan, finding, detection, domain)
  db/            # Database operations (scans, findings, domains)
  email/         # Resend email integration
  rate_limit/    # Database-backed rate limiting (IP + user tiers)
  ssrf/          # SSRF protection (blocks private IPs, cloud metadata)
  metrics/       # Prometheus metrics (HTTP, scan, rate limit counters)
  cleanup.rs     # Hourly data retention (24h anonymous, 30d Developer)
frontend/
  app/           # Next.js pages (landing, scan, results, dashboard, verify-domain, sign-in/up)
  components/    # UI components (scan form, results dashboard, scan history, auth gate, header)
  middleware.ts  # Clerk auth middleware (protects /dashboard, /verify-domain)
  __tests__/     # Vitest unit tests and Playwright E2E tests
migrations/      # PostgreSQL schema migrations
infrastructure/  # Ansible playbooks, templates, and deployment automation
```

## Architecture Highlights

- **Scan orchestrator** — Semaphore-based concurrency (configurable max), database-as-queue
- **Tiered scanning** — Anonymous gets light config (20 JS files, 180s timeout); authenticated gets full config (30 JS files, 300s timeout)
- **Results gating** — Server-side: high/critical findings stripped for non-owners. Frontend: lock overlay with signup CTA
- **Domain verification** — Meta tag verification with shared-hosting TLD blocklist, 30-day TTL
- **SSRF protection** — Blocks private IPs, cloud metadata endpoints (AWS/GCP/Azure), validates DNS resolution
- **Auth** — Clerk JWT verification via cached JWKS public keys (no per-request API calls), webhook user sync
- **Capability tokens** — 256-bit unguessable tokens for results sharing, auth optional for full access
- **Rate limiting** — 1 scan/IP/24h anonymous, 5 scans/user/month Developer tier, with `429 + resets_at`
- **Data retention** — Hourly cleanup: 24h expiry for anonymous scans, 30d for Developer, 24h grace period
- **Observability** — Structured JSON logging, request correlation IDs, Prometheus metrics, health checks (liveness + readiness)
- **Graceful shutdown** — SIGTERM drains in-flight scans with configurable timeout via TaskTracker/CancellationToken

## Testing

### Frontend

```bash
cd frontend

# Unit and component tests (Vitest + React Testing Library)
npm test

# Coverage report
npm run test:coverage

# E2E tests (Playwright)
npm run build && npx playwright test
```

Coverage thresholds enforced at 80% lines / 80% functions / 75% branches.

### CI/CD

GitHub Actions runs unit tests and E2E tests on every push and PR. Branch protection on `main` requires all checks to pass.

## License

[Mozilla Public License 2.0](LICENSE)
