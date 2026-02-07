---
phase: 06-deployment-infrastructure
plan: 03
subsystem: infra
tags: [ansible, nginx, certbot, systemd, docker-compose, letsencrypt, reverse-proxy, ssl]

# Dependency graph
requires:
  - phase: 06-01
    provides: Ansible playbook structure with 3-play SSH port transition and group_vars configuration
provides:
  - Nginx reverse proxy routing /api/ to backend:3000 and / to frontend:3001
  - Let's Encrypt SSL certificate provisioning with automatic renewal
  - Systemd service managing Docker Compose lifecycle with auto-start on boot
  - Production environment file template with Ansible vault variable substitution
  - Nuclei binary installation to /usr/local/bin for system PATH availability
affects: [06-04-validation, deployment, security]

# Tech tracking
tech-stack:
  added: [certbot-snap, nginx, nuclei-binary]
  patterns: [http-only-to-https-progression, systemd-oneshot-compose-manager, ansible-jinja2-templating]

key-files:
  created:
    - infrastructure/tasks/nginx.yml
    - infrastructure/tasks/certbot.yml
    - infrastructure/tasks/app.yml
    - infrastructure/tasks/systemd.yml
    - infrastructure/templates/trustedge.nginx.conf.j2
    - infrastructure/templates/env.production.j2
    - infrastructure/templates/trustedge.service.j2
  modified: []

key-decisions:
  - "Deploy HTTP-only Nginx config first, then full HTTPS config after Certbot obtains certificate (solves chicken-and-egg problem)"
  - "Use systemd Type=oneshot with RemainAfterExit=yes for Docker Compose lifecycle (cleaner than forking)"
  - "Extended timeouts for scan endpoints (300s read timeout) to handle long-running Nuclei scans"
  - "Nuclei installed to host /usr/local/bin instead of Docker image for faster execution and system PATH availability"

patterns-established:
  - "Certbot ACME challenge pattern: initial HTTP-only config with /.well-known/acme-challenge/ location, then full HTTPS after cert obtained"
  - "Systemd EnvironmentFile pattern: load production .env for docker compose variable substitution"
  - "Health check pattern: wait up to 5 minutes for backend /health endpoint after service start"

# Metrics
duration: 2min
completed: 2026-02-07
---

# Phase 06 Plan 03: Nginx, Certbot, Application Deployment Summary

**Nginx reverse proxy with Let's Encrypt SSL, systemd Docker Compose manager, production environment templating, and Nuclei host-level installation**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-07T04:38:30Z
- **Completed:** 2026-02-07T04:40:24Z
- **Tasks:** 2
- **Files created:** 7

## Accomplishments

- Nginx reverse proxy with security headers (HSTS, X-Frame-Options, CSP) and Mozilla Intermediate SSL profile
- Certbot SSL certificate provisioning with idempotent check and automatic renewal timer (via snap)
- Systemd service unit managing Docker Compose with auto-start, restart on failure, and 10-minute first-build timeout
- Production environment template covering all 12 variables from .env.example with Ansible vault substitution
- Nuclei binary installed to /usr/local/bin for PATH availability (dynamically fetches latest version from GitHub API)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Nginx and Certbot task files with templates** - `11c69b4` (feat)
2. **Task 2: Create application deployment and systemd task files with templates** - `8daed9e` (feat)

**Plan metadata:** (to be committed after SUMMARY.md creation)

## Files Created/Modified

### Created
- `infrastructure/tasks/nginx.yml` - Nginx installation and HTTP-only initial config for ACME challenge
- `infrastructure/tasks/certbot.yml` - SSL certificate provisioning with idempotent check and full HTTPS config deployment
- `infrastructure/tasks/app.yml` - Application repository clone, production .env deployment, Nuclei binary installation
- `infrastructure/tasks/systemd.yml` - Systemd service unit deployment, enable/start, health checks
- `infrastructure/templates/trustedge.nginx.conf.j2` - Full HTTPS reverse proxy config with security headers
- `infrastructure/templates/env.production.j2` - Production environment file with all variables from .env.example
- `infrastructure/templates/trustedge.service.j2` - Systemd unit for Docker Compose lifecycle management

### Modified
- None

## Decisions Made

**1. HTTP-only to HTTPS progression for Certbot**
- Deploy initial HTTP-only Nginx config with ACME challenge support
- Certbot obtains certificate using Nginx plugin
- Replace with full HTTPS config after certificate exists
- Solves chicken-and-egg problem where full config references non-existent SSL certs

**2. Systemd oneshot service pattern**
- `Type=oneshot` with `RemainAfterExit=yes` tracks docker compose as active
- Cleaner than forking with PID tracking
- `ExecReload` allows zero-confusion updates via `systemctl reload trustedge`
- 10-minute `TimeoutStartSec` for first Docker image build

**3. Extended timeouts for scan endpoints**
- Backend `/api/` location has 300s read timeout (vs 60s for frontend)
- Nuclei scans can take 30s-3min depending on target
- Prevents proxy timeout killing long-running scans

**4. Nuclei installed on host (not in Docker image)**
- Installed to /usr/local/bin via Ansible task
- Available in system PATH for systemd services
- Faster execution (no Docker overhead)
- Dynamic version resolution from GitHub API ensures latest scanner

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tasks completed as specified.

## User Setup Required

None - no external service configuration required (Ansible vault variables handle all secrets).

## Next Phase Readiness

**Ready for deployment execution:**
- All Ansible task files created and validated
- All Jinja2 templates created with proper variable substitution
- Nginx configured with security headers and appropriate timeouts
- Certbot configured for automatic SSL certificate renewal
- Systemd service configured for auto-start and restart on failure
- Application deployment task handles git clone and environment setup
- Nuclei installation automated with version discovery

**Blockers:**
- None

**Concerns:**
- Ansible vault variables (vault_domain_name, vault_admin_email, etc.) must be populated before playbook execution
- DigitalOcean Managed PostgreSQL connection string must be available in vault_database_url
- Git repository must be accessible from droplet (public repo or deploy key configured)

## Self-Check: PASSED

All claimed files exist on disk:
- infrastructure/tasks/nginx.yml ✓
- infrastructure/tasks/certbot.yml ✓
- infrastructure/tasks/app.yml ✓
- infrastructure/tasks/systemd.yml ✓
- infrastructure/templates/trustedge.nginx.conf.j2 ✓
- infrastructure/templates/env.production.j2 ✓
- infrastructure/templates/trustedge.service.j2 ✓

All claimed commits exist in git history:
- 11c69b4 ✓
- 8daed9e ✓

---
*Phase: 06-deployment-infrastructure*
*Completed: 2026-02-07*
