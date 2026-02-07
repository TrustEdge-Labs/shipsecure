---
phase: 06-deployment-infrastructure
plan: 02
subsystem: infra
tags: [ansible, digitalocean, docker, ufw, ssh, security]

# Dependency graph
requires:
  - phase: 06-01
    provides: Ansible playbook structure with 3-play SSH port transition pattern
provides:
  - DigitalOcean droplet provisioning with idempotent creation
  - SSH hardening (port 2222, key-only auth, no root login)
  - UFW firewall configuration (SSH/HTTP/HTTPS only)
  - Docker CE and Compose v2 plugin installation
  - Deploy user with sudo and docker group membership
affects: [06-03, deployment, security]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "UFW SSH allow rule MUST precede firewall enable to prevent lockout"
    - "Ansible handlers in playbook (not imported task files) for SSH restart"
    - "SSH hardening with sshd -t validation before applying changes"

key-files:
  created:
    - infrastructure/tasks/droplet.yml
    - infrastructure/tasks/security.yml
    - infrastructure/tasks/docker.yml
  modified:
    - infrastructure/playbooks/provision.yml

key-decisions:
  - "SSH restart handler placed in provision.yml Play 2, not in security.yml task file (Ansible import_tasks limitation)"
  - "UFW SSH allow rule positioned before firewall enable to prevent lockout (critical safety measure)"

patterns-established:
  - "Task files contain only tasks, handlers live in playbooks"
  - "Security-critical ordering: UFW SSH allow → UFW enable"
  - "SSH config changes validated with 'sshd -t -f %s' before applying"

# Metrics
duration: 1.5min
completed: 2026-02-07
---

# Phase 06 Plan 02: Ansible Task Files Summary

**DigitalOcean droplet provisioning with idempotent creation, SSH hardening on port 2222 with key-only auth, UFW firewall allowing SSH/HTTP/HTTPS, and Docker CE with Compose v2 plugin**

## Performance

- **Duration:** 1.5 min
- **Started:** 2026-02-07T04:36:50Z
- **Completed:** 2026-02-07T04:38:25Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- DigitalOcean droplet provisioning with unique_name idempotence prevents duplicate droplet creation
- SSH hardened with non-standard port (2222), key-only authentication, MaxAuthTries=3, LoginGraceTime=20s
- UFW firewall configured with SSH rule before enable (lockout prevention)
- Docker CE and Docker Compose v2 plugin installed with deploy user in docker group

## Task Commits

Each task was committed atomically:

1. **Task 1: Create droplet provisioning task file** - `b32d69d` (feat)
2. **Task 2: Create security hardening and Docker installation task files** - `962aa3c` (feat)

## Files Created/Modified
- `infrastructure/tasks/droplet.yml` - DigitalOcean droplet creation with SSH key lookup, IP extraction, inventory addition, SSH wait
- `infrastructure/tasks/security.yml` - Deploy user creation, SSH hardening (port/auth/root), UFW firewall rules (SSH/HTTP/HTTPS)
- `infrastructure/tasks/docker.yml` - Docker CE + Compose v2 plugin installation, deploy user docker group membership, service enablement
- `infrastructure/playbooks/provision.yml` - Added Restart SSH handler to Play 2 for security task notifications

## Decisions Made

**1. SSH restart handler placement**
- **Issue:** Ansible import_tasks doesn't support handlers in imported task files
- **Decision:** Placed "Restart SSH" handler in provision.yml Play 2, not in security.yml
- **Rationale:** Handlers must be defined in the playbook that imports the tasks
- **Impact:** All task files remain pure task lists, handlers centralized in playbook

**2. UFW SSH rule ordering**
- **Issue:** Enabling UFW before allowing SSH port causes immediate lockout
- **Decision:** UFW SSH allow rule positioned at line 74, UFW enable at line 105
- **Rationale:** Research Pitfall 1 warns about SSH configuration lockout risk
- **Impact:** Safe firewall deployment without manual recovery needed

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**1. YAML syntax error - handlers in task file**
- **Problem:** Initial security.yml included handlers section (lines 109-113), causing YAML parser error "expected <block end>, but found '?'"
- **Root cause:** Ansible import_tasks doesn't support handlers in imported task files
- **Resolution:** Removed handlers from security.yml, moved to provision.yml Play 2
- **Verification:** YAML syntax validation passed for all three files
- **Category:** Ansible structural requirement, not a deviation (fixed during Task 2 execution)

## User Setup Required

None - no external service configuration required at this stage.

## Next Phase Readiness

**Ready for 06-03 (Nginx/Certbot/systemd tasks):**
- Droplet provisioning tasks ready for execution
- Security hardening tasks configured with safe UFW ordering
- Docker installation tasks will provide container runtime for application

**No blockers.**

**Verification needed before first ansible-playbook run:**
- DigitalOcean API token in vault.yml
- SSH public key uploaded to DigitalOcean account
- Domain DNS A record prepared (will be set after droplet IP obtained)

---
*Phase: 06-deployment-infrastructure*
*Completed: 2026-02-07*

## Self-Check: PASSED

All created files verified on disk:
- infrastructure/tasks/droplet.yml
- infrastructure/tasks/security.yml

All commits verified in git history:
- b32d69d
- 962aa3c
