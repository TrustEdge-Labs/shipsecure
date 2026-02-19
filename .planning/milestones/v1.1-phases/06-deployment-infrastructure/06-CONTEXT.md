# Phase 06: Deployment Infrastructure - Context

**Gathered:** 2026-02-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Production infrastructure provisioned and configured on DigitalOcean with reverse proxy, SSL, process management, and firewall hardening. Covers INFRA-01, PROXY-01, PROXY-02, PROC-01, SEC-01. Does not include CI/CD pipelines, monitoring services, or application-level changes.

</domain>

<decisions>
## Implementation Decisions

### Provisioning approach
- Ansible playbook for all server configuration (idempotent, re-runnable)
- Playbook lives in this repo under `deploy/` or `infrastructure/` directory
- Playbook handles droplet creation via DigitalOcean API / Ansible module
- Droplet size: s-4vcpu-8gb ($48/mo) — 4 vCPUs, 8GB RAM

### Service architecture
- Docker Compose for running all application services (using docker-compose.prod.yml from Phase 05)
- PostgreSQL on DigitalOcean Managed Database (not containerized in production)
- Base docker-compose.yml keeps Postgres for local dev; production override excludes it and uses DATABASE_URL for managed DB
- Deploy workflow: SSH in, git pull, docker compose build & up (manual, simple)

### Domain & SSL
- User has a domain ready — playbook takes domain as a variable
- DNS managed at registrar — user updates A records manually
- SSL via Certbot with Nginx plugin (automatic cert placement and renewal)
- Nginx runs on the host (not containerized) — simpler certbot integration, directly manages ports 80/443
- Docker Compose services expose on localhost ports, Nginx reverse proxies to them

### SSH & access
- SSH key-only authentication, password auth disabled
- Non-standard SSH port (configurable, e.g., 2222) to reduce noise
- Root login disabled
- Playbook creates a non-root 'deploy' user with sudo — app runs under this user

### Firewall
- UFW configured allowing only SSH (non-standard port), 80, and 443

### Logging
- Docker's default logging driver + journald
- No extra log tooling — viewable with `docker logs` and `journalctl`

### Monitoring
- Skipped for this phase — handle manually until there's traffic

### Claude's Discretion
- Exact Ansible role/task structure and directory layout
- Nginx configuration details (worker processes, buffer sizes, etc.)
- Certbot renewal cron/timer setup specifics
- Systemd service unit details for Docker Compose
- DigitalOcean region selection
- Exact non-standard SSH port number

</decisions>

<specifics>
## Specific Ideas

- Managed Postgres removes DB maintenance burden and provides automated backups out of the box
- Host Nginx chosen specifically because certbot Nginx plugin integration is simpler than containerized cert management
- Non-standard SSH port is a pragmatic noise reducer, not a security measure — key-only auth is the real protection

</specifics>

<deferred>
## Deferred Ideas

- CI/CD pipeline (GitHub Actions) — future improvement after manual deploy workflow is validated
- Uptime monitoring (UptimeRobot, Betterstack, or custom) — add after there's real traffic
- Log aggregation / centralized logging — not needed at MVP scale

</deferred>

---

*Phase: 06-deployment-infrastructure*
*Context gathered: 2026-02-06*
