# Roadmap: ShipSecure

## Milestones

- ✅ **v1.0 MVP** — Phases 01-04 (shipped 2026-02-06)
- ✅ **v1.1 DigitalOcean Deployment** — Phases 05-07 (shipped 2026-02-08)
- ✅ **v1.2 Launch Readiness** — Phases 08-12 (shipped 2026-02-10)
- ✅ **v1.3 Brand Identity** — Phases 13-18 (shipped 2026-02-11)
- 🚧 **v1.4 Observability** — Phases 19-24 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 01-04) — SHIPPED 2026-02-06</summary>

- [x] Phase 01: Foundation (5/5 plans) — Rust/Axum backend, Next.js frontend, PostgreSQL schema
- [x] Phase 02: Free Tier MVP (8/8 plans) — 5 scanners, email delivery, results dashboard
- [x] Phase 03: Vibe-Code Intelligence (5/5 plans) — Framework detection, Nuclei templates, remediation guidance
- [x] Phase 04: Monetization (5/5 plans) — Stripe checkout, PDF reports, paid audit flow

See: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 DigitalOcean Deployment (Phases 05-07) — SHIPPED 2026-02-08</summary>

- [x] Phase 05: Codebase Preparation (4/4 plans) — Native subprocesses, config externalization, Docker builds
- [x] Phase 06: Deployment Infrastructure (4/4 plans) — Ansible provisioning, Nginx + SSL, systemd, UFW
- [x] Phase 07: Production Validation (2/2 plans) — Scanner validation, email delivery, Stripe flow, resilience

See: `.planning/milestones/v1.1-ROADMAP.md`

</details>

<details>
<summary>✅ v1.2 Launch Readiness (Phases 08-12) — SHIPPED 2026-02-10</summary>

- [x] Phase 08: Analytics & Tracking (1/1 plan) — Plausible analytics, conversion events
- [x] Phase 09: SEO & Discoverability (2/2 plans) — Meta tags, OG image, JSON-LD, sitemap, robots.txt
- [x] Phase 10: Legal Compliance (2/2 plans) — Privacy Policy, TOS, CFAA consent checkbox
- [x] Phase 11: Mobile & UX Polish (3/3 plans) — Mobile responsive, loading states, error boundaries, Lighthouse
- [x] Phase 12: Landing Page Optimization (2/2 plans) — Developer-focused copy, methodology transparency, OSS attribution

See: `.planning/milestones/v1.2-ROADMAP.md`

</details>

<details>
<summary>✅ v1.3 Brand Identity (Phases 13-18) — SHIPPED 2026-02-11</summary>

- [x] Phase 13: Design Token System (3/3 plans) — OKLch primitives, semantic tokens, dark mode, WCAG AA
- [x] Phase 14: Logo Component (2/2 plans) — Shield logo, responsive variants, professional PNG
- [x] Phase 15: Layout Refactor (1/1 plan) — Header-height token, layout preparation
- [x] Phase 16: Header & Navigation (1/1 plan) — Sticky header, responsive logo, CTA, keyboard nav
- [x] Phase 17: Icon System & Migration (1/1 plan) — Lucide React SVG icons replacing emoji
- [x] Phase 18: Favicon & OG Image (2/2 plans) — Branded favicon (ICO+SVG), Apple touch icon, OG image

See: `.planning/milestones/v1.3-ROADMAP.md`

</details>

### 🚧 v1.4 Observability (In Progress)

**Milestone Goal:** Add production-grade observability for debugging, monitoring, and operational visibility

**Phase Numbering:**
- Integer phases (19-24): Planned milestone work
- Decimal phases (19.1, 19.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 19: Structured JSON Logging** - Environment-driven JSON logs with structured fields (completed 2026-02-16)
- [x] **Phase 20: Request Tracing** - Correlation IDs and request/response logging (completed 2026-02-16)
- [x] **Phase 21: Health Checks** - Liveness and readiness endpoints with DB checks (completed 2026-02-16)
- [x] **Phase 22: Prometheus Metrics** - /metrics endpoint with HTTP and scan metrics (completed 2026-02-16)
- [ ] **Phase 23: Graceful Shutdown** - SIGTERM handling with scan drain coordination
- [ ] **Phase 24: Infrastructure Integration** - Deploy observability to production

## Phase Details

### Phase 19: Structured JSON Logging
**Goal**: Backend emits structured JSON logs in production with scan lifecycle context
**Depends on**: Nothing (first phase of v1.4)
**Requirements**: LOG-01, LOG-02, LOG-03, LOG-04
**Success Criteria** (what must be TRUE):
  1. Backend logs to stdout in JSON format when LOG_FORMAT=json
  2. Backend logs to stdout in human-readable text when LOG_FORMAT=text (development default)
  3. All log events include timestamp, level, target, and span context fields
  4. Scan lifecycle events include scan_id, target_url, tier, and scanner name in structured fields
  5. Panic handler outputs structured JSON with backtrace when JSON logging enabled
**Plans:** 2/2 plans complete

Plans:
- [ ] 19-01-PLAN.md — Logging foundation: deps, format switching, env filter, panic hook, startup banner
- [ ] 19-02-PLAN.md — Scan lifecycle instrumentation: structured events in orchestrator

### Phase 20: Request Tracing
**Goal**: Every HTTP request gets traced with correlation IDs propagated to background tasks
**Depends on**: Phase 19 (needs structured logging foundation)
**Requirements**: TRC-01, TRC-02, TRC-03
**Success Criteria** (what must be TRUE):
  1. Every HTTP request receives a unique request_id via tower-http TraceLayer
  2. Request and response logs include method, URI, status code, and latency_ms
  3. Background scan tasks inherit request span context via .instrument()
  4. Request_id appears in all logs associated with a single request lifecycle
**Plans:** 2/2 plans complete

Plans:
- [ ] 20-01-PLAN.md — TraceLayer middleware, health check filtering, database migration, Scan model update
- [ ] 20-02-PLAN.md — RequestId extension wiring through handlers, orchestrator propagation, webhook update

### Phase 21: Health Checks
**Goal**: Load balancers and monitoring systems can check service health with deep readiness validation
**Depends on**: Phase 19 (needs logging for health check events)
**Requirements**: HLT-01, HLT-02, HLT-03
**Success Criteria** (what must be TRUE):
  1. GET /health returns 200 "ok" in under 10ms (shallow liveness check)
  2. GET /health/ready returns JSON with db_connected, scan_capacity, and status fields
  3. GET /health/ready returns 503 when database is unreachable (tested via disconnected DB)
  4. GET /health/ready completes in under 100ms including DB connectivity check
**Plans:** 1/1 plans complete

Plans:
- [ ] 21-01-PLAN.md — Liveness and readiness health check endpoints with DB validation, scan capacity, cache

### Phase 22: Prometheus Metrics
**Goal**: Operational metrics exposed at /metrics endpoint for monitoring HTTP requests, scan performance, and queue depth
**Depends on**: Phase 19 (needs logging), Phase 21 (health patterns inform metrics design)
**Requirements**: MET-01, MET-02, MET-03, MET-04, MET-05, MET-06, MET-07, MET-08
**Success Criteria** (what must be TRUE):
  1. GET /metrics returns OpenMetrics format text (Prometheus can scrape)
  2. http_requests_total counter increments with method, endpoint, and status labels
  3. http_request_duration_seconds histogram records request latency with method and endpoint labels
  4. scan_duration_seconds histogram records scan execution time with tier and status labels
  5. active_scans gauge reflects current number of in-flight scans
  6. scan_queue_depth gauge reflects number of pending scans waiting to execute
  7. scanner_results_total counter tracks individual scanner success/failure with scanner and status labels
  8. rate_limit_total counter tracks rate limiting events with limiter and action labels
  9. /metrics endpoint is restricted to localhost only (external requests return 403 from Nginx)
**Plans:** 2/2 plans complete

Plans:
- [ ] 22-01-PLAN.md — Metrics infrastructure, HTTP request metrics middleware, /metrics endpoint with localhost access
- [ ] 22-02-PLAN.md — Scan metrics (duration, active, queue depth, scanner results) and rate limit counters

### Phase 23: Graceful Shutdown
**Goal**: Backend drains in-flight scans before exiting on SIGTERM/SIGINT to prevent data loss
**Depends on**: Phase 19 (logging shutdown events), Phase 22 (metrics inform shutdown behavior)
**Requirements**: SHD-01, SHD-02, SHD-03
**Success Criteria** (what must be TRUE):
  1. Backend receives SIGTERM and logs graceful shutdown initiation
  2. In-flight scans complete before process exits (tested via docker stop during active scan)
  3. Background tasks tracked via TaskTracker instead of fire-and-forget tokio::spawn
  4. Shutdown timeout respects configurable grace period (default 90s) before force termination
  5. New scan requests receive 503 Service Unavailable during shutdown drain period
**Plans:** 2 plans

Plans:
- [ ] 23-01-PLAN.md — TaskTracker and CancellationToken integration in ScanOrchestrator
- [ ] 23-02-PLAN.md — Signal handler, 503 middleware, health shutdown awareness, periodic logging, coordinated shutdown

### Phase 24: Infrastructure Integration
**Goal**: All observability components deployed to production with security hardening and monitoring agent
**Depends on**: Phase 19-23 (all code changes complete)
**Requirements**: INF-01, INF-02, INF-03, INF-04, INF-05
**Success Criteria** (what must be TRUE):
  1. DigitalOcean metrics agent installed and reporting infrastructure metrics (CPU, memory, disk)
  2. Nginx restricts /metrics endpoint to 127.0.0.1 (external curl returns 403)
  3. Docker Compose sets STOPSIGNAL SIGTERM and stop_grace_period 90s
  4. Docker Compose configures JSON log driver with rotation (max-size 10m, max-file 3)
  5. systemd service sets TimeoutStopSec=95s to accommodate graceful shutdown
  6. Production environment sets LOG_FORMAT=json
  7. All observability features verified working in production (logs, metrics, health, shutdown)
**Plans**: TBD

Plans:
- [ ] 24-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 01-18 (complete) → 19 → 20 → 21 → 22 → 23 → 24

| Phase | Milestone | Plans | Status | Completed |
|-------|-----------|-------|--------|-----------|
| 01 - Foundation | v1.0 | 5/5 | Complete | 2026-02-04 |
| 02 - Free Tier MVP | v1.0 | 8/8 | Complete | 2026-02-05 |
| 03 - Vibe-Code Intelligence | v1.0 | 5/5 | Complete | 2026-02-05 |
| 04 - Monetization | v1.0 | 5/5 | Complete | 2026-02-06 |
| 05 - Codebase Preparation | v1.1 | 4/4 | Complete | 2026-02-07 |
| 06 - Deployment Infrastructure | v1.1 | 4/4 | Complete | 2026-02-08 |
| 07 - Production Validation | v1.1 | 2/2 | Complete | 2026-02-08 |
| 08 - Analytics & Tracking | v1.2 | 1/1 | Complete | 2026-02-08 |
| 09 - SEO & Discoverability | v1.2 | 2/2 | Complete | 2026-02-08 |
| 10 - Legal Compliance | v1.2 | 2/2 | Complete | 2026-02-08 |
| 11 - Mobile & UX Polish | v1.2 | 3/3 | Complete | 2026-02-09 |
| 12 - Landing Page Optimization | v1.2 | 2/2 | Complete | 2026-02-09 |
| 13 - Design Token System | v1.3 | 3/3 | Complete | 2026-02-10 |
| 14 - Logo Component | v1.3 | 2/2 | Complete | 2026-02-11 |
| 15 - Layout Refactor | v1.3 | 1/1 | Complete | 2026-02-11 |
| 16 - Header & Navigation | v1.3 | 1/1 | Complete | 2026-02-11 |
| 17 - Icon System & Migration | v1.3 | 1/1 | Complete | 2026-02-11 |
| 18 - Favicon & OG Image | v1.3 | 2/2 | Complete | 2026-02-11 |
| 19 - Structured JSON Logging | v1.4 | Complete    | 2026-02-16 | - |
| 20 - Request Tracing | v1.4 | Complete    | 2026-02-16 | - |
| 21 - Health Checks | v1.4 | Complete    | 2026-02-16 | - |
| 22 - Prometheus Metrics | v1.4 | Complete    | 2026-02-16 | - |
| 23 - Graceful Shutdown | v1.4 | 0/2 | Not started | - |
| 24 - Infrastructure Integration | v1.4 | 0/? | Not started | - |

---
*Last updated: 2026-02-16*
