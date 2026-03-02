# Phase 40: Docker Healthchecks & Docs - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Production Docker containers (backend and frontend) self-report health to Docker via HEALTHCHECK, and the root README accurately describes the tech stack (Next.js 16, not 15). No new scanning features, no monitoring dashboards, no alerting.

</domain>

<decisions>
## Implementation Decisions

### Healthcheck placement
- Define healthchecks in `docker-compose.prod.yml` only (not in Dockerfiles)
- Keeps Dockerfiles portable, lets intervals be tuned per environment
- Consistent with how all other config lives in this project (compose-level, not image-level)

### Backend healthcheck
- Poll `/health/ready` (readiness endpoint) — checks DB connectivity, scan capacity, and cache
- Returns 503 during graceful shutdown, which makes Docker mark the container unhealthy before it stops
- Use `curl` (already installed in the backend runtime image)
- Conservative timing: 30s interval, 30s timeout, 3 retries, 60s start period (backend runs scans that can stress resources)

### Frontend healthcheck
- Add a lightweight `/api/health` Next.js API route that returns 200 OK
- Probe with `wget` (already available in node:20-alpine — no extra packages)
- Avoids rendering a full page on every check

### Unhealthy visibility
- Docker's built-in health status column (`docker ps` shows `(healthy)`/`(unhealthy)`) satisfies success criterion 3
- Defining HEALTHCHECK in compose inherently makes unhealthy containers distinguishable without reading logs
- Upgrade `depends_on` condition from `service_started` to `service_healthy` so frontend waits for backend to be genuinely ready

### README updates
- Fix "Next.js 15" to "Next.js 16" in the root README tech stack table
- Fix "middleware.ts" to "proxy.ts" in the Project Structure section (already proxy.ts in the codebase)
- Root README only — leave frontend/README.md boilerplate as-is (harmless)

### Claude's Discretion
- Exact healthcheck timing for frontend container
- Whether to add any additional README accuracy fixes discovered during implementation
- Frontend `/api/health` response body format (simple JSON or plain text)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Backend `/health` (liveness) and `/health/ready` (readiness) endpoints already exist in `src/api/health.rs`
- `curl` is installed in the backend runtime image (`debian:bookworm-slim`)
- `wget` is available in `node:20-alpine` (frontend runtime image)

### Established Patterns
- All production config lives in `docker-compose.prod.yml` (standalone, not a dev override)
- Backend health endpoints are excluded from metrics tracking (`src/metrics/middleware.rs`)
- Frontend uses Next.js App Router with `app/` directory structure

### Integration Points
- `docker-compose.prod.yml` — add `healthcheck:` blocks to both services
- `docker-compose.prod.yml` — change `depends_on.backend.condition` from `service_started` to `service_healthy`
- `frontend/app/api/health/route.ts` — new API route file for frontend health endpoint
- `README.md` — tech stack table and project structure section

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard Docker healthcheck patterns apply.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 40-docker-healthchecks-docs*
*Context gathered: 2026-03-01*
