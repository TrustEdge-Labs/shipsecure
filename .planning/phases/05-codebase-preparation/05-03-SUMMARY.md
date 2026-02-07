---
phase: 05-codebase-preparation
plan: 03
subsystem: infra
tags: [docker, docker-compose, nuclei, testssl, multi-stage-build, debian, deployment]

# Dependency graph
requires:
  - phase: 05-01
    provides: Scanner native binary execution via subprocess
  - phase: 05-02
    provides: Environment configuration validation
provides:
  - Multi-stage Dockerfile with Nuclei and testssl.sh binaries installed
  - Development docker-compose.yml with full stack (backend + frontend + PostgreSQL)
  - Production docker-compose.prod.yml overrides for deployment
  - Non-root container execution (trustedge user)
  - Template hot-reload support for development
affects: [06-infrastructure-setup, deployment, production]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Multi-stage Docker builds for slim runtime images
    - Docker Compose override pattern for environment-specific config
    - Non-root container execution for security
    - Volume mount for development hot-reload

key-files:
  created:
    - docker-compose.prod.yml
  modified:
    - Dockerfile
    - docker-compose.yml

key-decisions:
  - "Install scanner binaries directly in Docker image (not Docker-in-Docker)"
  - "Dynamic Nuclei version resolution from GitHub API at build time"
  - "testssl.sh cloned from git (no versioned releases)"
  - "Production removes port mappings (proxied via Nginx)"
  - "Development mounts templates as volume for hot-reload"
  - "Resource limits: backend 2 CPU/2G RAM, frontend/db 1 CPU/1G RAM"

patterns-established:
  - "Docker Compose override files for environment-specific configuration"
  - "Multi-stage builds: builder stage → slim runtime stage"
  - "Non-root user execution inside containers (trustedge:1000)"
  - "Environment variable substitution in docker-compose.prod.yml"

# Metrics
duration: 1m 17s
completed: 2026-02-07
---

# Phase 05 Plan 03: Docker Configuration Summary

**Multi-stage Docker build with Nuclei and testssl.sh binaries installed natively, development and production Docker Compose configurations with template hot-reload and resource limits**

## Performance

- **Duration:** 1m 17s
- **Started:** 2026-02-07T02:08:50Z
- **Completed:** 2026-02-07T02:10:07Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Backend Docker image includes Nuclei (latest from GitHub) and testssl.sh binaries with all runtime dependencies
- Full-stack development environment via `docker compose up` with PostgreSQL, backend, frontend
- Production overrides remove port mappings, add restart policies, resource limits, and log rotation
- Templates mounted as read-only volume in development for hot-reload during vibe-code intelligence development
- Non-root container execution (trustedge user with UID 1000) for security best practices

## Task Commits

Each task was committed atomically:

1. **Task 1: Update backend Dockerfile with scanner binaries** - `b746cc0` (feat)
2. **Task 2: Update Docker Compose for full-stack development and production** - `2563a28` (feat)

**Plan metadata:** (to be committed after SUMMARY creation)

## Files Created/Modified

- `Dockerfile` - Multi-stage build with builder (Rust 1.88) and runtime (Debian Bookworm slim) stages. Runtime installs Nuclei binary dynamically from GitHub releases API, clones testssl.sh from git, creates non-root user, copies scanner templates and fonts, sets environment variables for binary paths.
- `docker-compose.yml` - Development stack with PostgreSQL 16, backend (with templates volume mount), and frontend. Backend depends on healthy database. All ports exposed for local access.
- `docker-compose.prod.yml` - Production overrides that remove port mappings, add restart policies (unless-stopped), configure resource limits (backend: 2 CPU/2G RAM, others: 1 CPU/1G), enable JSON logging with rotation, use environment variable substitution for secrets (DB_PASSWORD, NEXT_PUBLIC_BACKEND_URL), remove template volume mount.

## Decisions Made

**Dynamic Nuclei version resolution:** Dockerfile uses `curl` to GitHub API to get latest Nuclei version tag, then downloads corresponding binary. This ensures fresh builds get the latest scanner without manual version updates. If fragile, can pin specific version.

**testssl.sh from git clone:** testssl.sh has no versioned binary releases, so cloning the repo (`--depth 1` for minimal size) is the standard installation method. Symlinked to `/usr/local/bin/testssl.sh` for PATH access.

**Production port removal via override:** docker-compose.prod.yml sets `ports: []` to remove all port mappings from base config. Services communicate internally via Docker network. Nginx will proxy external traffic to backend (port 3000) and frontend (port 3001) inside the network.

**Template hot-reload in dev only:** Development mounts `./templates:/app/templates:ro` so changes to Nuclei templates are immediately available without rebuilding the image. Production uses templates baked into the image (from COPY instruction) for immutability.

**Resource limits for production:** Backend gets 2 CPU cores and 2GB RAM (scanner-intensive). Frontend and database get 1 CPU core and 1GB RAM each. Limits prevent resource exhaustion on shared droplet.

**Environment variable substitution:** Production uses `${DB_PASSWORD}` and `${NEXT_PUBLIC_BACKEND_URL}` for secrets and deployment-specific values. Phase 06 systemd service will load these from environment file.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all Docker Compose configurations validated successfully. Dynamic Nuclei version resolution pattern works with GitHub API rate limits for anonymous requests (60/hour should be sufficient for image builds).

## User Setup Required

None - no external service configuration required. Phase 06 will create environment file with required variables (DB_PASSWORD, NEXT_PUBLIC_BACKEND_URL) for production deployment.

## Next Phase Readiness

**Ready for Phase 06 (Infrastructure Setup):**
- Docker images can be built locally and on DigitalOcean droplet
- Development workflow: `docker compose up` starts full stack with hot-reload
- Production workflow: `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d` starts services with overrides
- Phase 06 systemd units will manage `docker compose` lifecycle
- Nginx configuration needed to proxy traffic to backend:3000 and frontend:3001

**Verified:**
- Multi-stage build pattern preserves dependency caching
- Scanner binaries installed in runtime image
- Non-root user execution configured
- No Docker socket mounts (scanners run as native binaries)
- YAML syntax valid, merged config validated with `docker compose config`

**Blockers:** None

---
*Phase: 05-codebase-preparation*
*Completed: 2026-02-07*

## Self-Check: PASSED

All key files verified on disk:
- ✓ docker-compose.prod.yml
- ✓ Dockerfile
- ✓ docker-compose.yml

All commits verified in git history:
- ✓ b746cc0 (Task 1: install scanner binaries)
- ✓ 2563a28 (Task 2: Docker Compose configuration)
