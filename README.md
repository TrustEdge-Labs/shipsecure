# ShipSecure

Security scanning platform for vibe-coded apps. Paste a URL, get an A-F security grade with actionable fixes — no security expertise required.

Built for developers shipping fast with AI code generators (Cursor, Bolt, Lovable, etc.) who need to know if their app is leaking secrets, missing headers, or exposing admin panels.

## Why This Exists

- 45% of AI-generated code contains security flaws
- 86% of AI tools fail XSS defenses; 88% fail log injection tests
- Lovable's built-in scanner catches vulnerabilities only 66% of the time; Bolt's fails entirely
- CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys

## What It Scans

**Free tier** (no signup, just a URL and email):

- **Security headers** — CSP, HSTS, X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
- **TLS/SSL** — Certificate validity, cipher strength, protocol versions, known vulnerabilities (via SSL Labs API)
- **Exposed files** — `.env`, `.git`, `/debug`, `/admin`, source maps, API docs, `phpinfo.php`, `server-status`
- **Client-side secrets** — AWS keys, Stripe keys, Supabase/Firebase credentials, API tokens leaked in JavaScript bundles
- **Framework detection** — Auto-detects Next.js, Vite, React, SvelteKit, Nuxt
- **Platform detection** — Auto-detects Vercel, Netlify, Railway, Render, Supabase, Firebase

**Paid audit** ($49 one-time):

- Everything in free, plus containerized Nuclei scanning
- **Vibe-code checks** — Supabase RLS misconfigurations, Firebase security rules, Vercel env leaks, Netlify function exposure, Railway/Render debug endpoints
- **Remediation playbooks** — Framework-specific copy-paste fixes
- **PDF report** — Professional report with executive summary, findings by severity, remediation roadmap

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust (Axum) |
| Frontend | Next.js (App Router), React, Tailwind CSS |
| Database | PostgreSQL |
| Scanners | Nuclei (native binary), testssl.sh (native binary), custom probes |
| Observability | tracing (structured logging), metrics-exporter-prometheus, tower-http |
| Payments | Stripe Checkout |
| Email | Resend API |
| Reports | genpdf (PDF generation) |
| Infrastructure | Ansible, Docker Compose, Nginx, systemd |

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
npm install
npm run dev  # Runs on http://localhost:3001
```

### Configuration

Copy `.env.example` and set the following:

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `PORT` | Backend HTTP port | Yes |
| `SHIPSECURE_BASE_URL` | Base URL for email links | Yes |
| `FRONTEND_URL` | Frontend URL for redirects | Yes |
| `MAX_CONCURRENT_SCANS` | Maximum parallel scans | Yes |
| `LOG_FORMAT` | Log format: `json` for structured, unset for text | No (defaults to text) |
| `RUST_LOG` | Log level filter override | No (sensible defaults) |
| `SHUTDOWN_TIMEOUT` | Graceful shutdown timeout in seconds | No (defaults to 90) |
| `RESEND_API_KEY` | Resend API key for email delivery | No (email disabled) |
| `STRIPE_SECRET_KEY` | Stripe secret key | No (checkout disabled) |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook validation secret | No (webhooks disabled) |
| `NUCLEI_BINARY_PATH` | Path to Nuclei binary | No (auto-detected) |
| `TESTSSL_BINARY_PATH` | Path to testssl.sh binary | No (auto-detected) |
| `SHIPSECURE_TEMPLATES_DIR` | Custom Nuclei templates directory | No (uses bundled) |

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Liveness check (returns "ok") |
| `GET` | `/health/ready` | Readiness check (DB connectivity, scan capacity) |
| `GET` | `/metrics` | Prometheus metrics (localhost only) |
| `POST` | `/api/v1/scans` | Submit a scan |
| `GET` | `/api/v1/scans/{id}` | Get scan status and findings |
| `GET` | `/api/v1/results/{token}` | Get results by capability token |
| `GET` | `/api/v1/results/{token}/download` | Download results as markdown |
| `GET` | `/api/v1/stats/scan-count` | Get total scan count |
| `POST` | `/api/v1/checkout` | Create Stripe checkout session |
| `POST` | `/api/v1/webhooks/stripe` | Stripe webhook handler |

## Project Structure

```
src/
  api/           # HTTP handlers (scans, results, checkout, webhooks, health)
  scanners/      # Security scanners (headers, TLS, secrets, vibecode, etc.)
  orchestrator/  # Scan job management with concurrency control and graceful shutdown
  models/        # Data structures (scan, finding, detection, paid_audit)
  db/            # Database operations
  email/         # Resend email integration
  pdf.rs         # PDF report generation
  rate_limit/    # Database-backed rate limiting
  ssrf/          # SSRF protection (blocks private IPs, cloud metadata)
  metrics/       # Prometheus metrics (HTTP, scan, rate limit counters)
frontend/
  app/           # Next.js pages (landing, scan, results, payment)
  components/    # Reusable UI components
  actions/       # Server Actions
migrations/      # PostgreSQL schema migrations
infrastructure/  # Ansible playbooks, templates, and deployment automation
```

## Architecture Highlights

- **Scan orchestrator** — Semaphore-based concurrency (max 5 concurrent scans), database-as-queue
- **SSRF protection** — Blocks private IPs, cloud metadata endpoints (AWS/GCP/Azure), validates DNS resolution
- **Scanner execution** — Nuclei and testssl.sh run as native binaries with configurable paths and graceful degradation
- **Capability tokens** — 256-bit unguessable tokens for results access, no auth required
- **Graceful degradation** — Scanner unavailable? Empty findings with warning. Email fails? PDF saved to disk.
- **Observability** — Structured JSON logging, request tracing with correlation IDs, Prometheus metrics, health checks (liveness + readiness)
- **Graceful shutdown** — SIGTERM drains in-flight scans with configurable timeout, tracked via TaskTracker/CancellationToken

## Testing

```bash
# Run the end-to-end test suite
PORT=8888 cargo run &
./test-e2e.sh
```

## License

[Mozilla Public License 2.0](LICENSE)
