# Phase 05: Codebase Preparation - Context

**Gathered:** 2026-02-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Make the existing v1.0 codebase deployment-ready by removing all Render references, switching Nuclei and testssl.sh from Docker containers (bollard) to native subprocesses, externalizing all configuration/secrets, and setting up Docker Compose for both local development and production deployment. Zero changes to scanning logic or features.

</domain>

<decisions>
## Implementation Decisions

### Nuclei subprocess design
- Binary located via `NUCLEI_BINARY_PATH` env var with fallback to PATH lookup
- If Nuclei binary not found at startup: log warning and continue (other scanners still work). Good for dev environments.
- Custom vibe-code templates bundled in repo (e.g., `templates/nuclei/`), with optional override via `NUCLEI_TEMPLATES_DIR` env var for additional/custom templates
- Scan output captured via JSON temp file (`-o` flag), not stdout. Avoids buffering issues.

### testssl.sh subprocess design
- Also moves to subprocess execution (bollard removed entirely)
- Same pattern as Nuclei: binary path via env var, graceful skip if missing
- Claude's discretion on specific invocation details

### Configuration structure
- Single `.env` file for all variables
- `.env.example` committed to repo with all variable names and placeholder/description comments (no real values)
- Fail fast at startup: validate ALL required env vars on boot, crash with clear error listing which vars are missing
- All env vars must be explicitly set — no hidden defaults. PORT, LOG_LEVEL, MAX_CONCURRENT_SCANS, etc. all require explicit values in .env
- What you see in .env.example is exactly what the app needs

### Render cleanup scope
- Thorough cleanup: delete render.yaml, strip all Render-specific env var names, remove Render mentions from docs/comments, refactor any code that assumed Render's environment
- Remove bollard crate entirely (both Nuclei and testssl.sh move to subprocess)
- `/health` endpoint stays (standard, not Render-specific)
- Claude should audit codebase for any other Render-specific assumptions during research/planning
- Update README and docs in this phase to reflect new setup — remove Render references, add new local dev instructions

### Dev/prod parity
- Docker Compose for both local development AND production deployment
- Full stack in Docker Compose: Rust backend + Next.js frontend + PostgreSQL + scanner binaries
- One `docker compose up` to run everything locally
- Production uses same docker-compose.yml with prod overrides (e.g., docker-compose.prod.yml)
- Systemd manages `docker compose up` on the droplet
- Multi-stage Dockerfiles: build stage (compile Rust, build Next.js) + slim runtime stage

### Claude's Discretion
- testssl.sh subprocess invocation details
- Exact temp file handling for Nuclei JSON output
- Docker Compose service naming and networking
- .env.example variable descriptions and grouping
- Dockerfile base images and runtime dependencies

</decisions>

<specifics>
## Specific Ideas

- User said `/health` endpoint is probably standard and should stay
- User wants Claude to audit the codebase for Render-specific code during research/planning (not sure what all is there beyond render.yaml and env vars)
- Docker Compose in prod means Phase 06's systemd units just need to manage `docker compose up/down`, not individual containers

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-codebase-preparation*
*Context gathered: 2026-02-06*
