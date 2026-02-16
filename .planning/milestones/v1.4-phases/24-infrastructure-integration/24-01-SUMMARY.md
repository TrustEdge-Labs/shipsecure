---
phase: 24-infrastructure-integration
plan: 01
subsystem: infrastructure
tags:
  - ansible
  - observability
  - deployment
  - metrics
  - graceful-shutdown
dependency_graph:
  requires:
    - "Phase 23: Graceful shutdown application logic"
    - "Phase 22: Prometheus metrics endpoint"
    - "Phase 19: JSON logging support"
  provides:
    - "INF-01: DigitalOcean metrics agent installation"
    - "INF-02: Nginx /metrics endpoint with localhost-only access"
    - "INF-03: Docker Compose graceful shutdown configuration"
    - "INF-04: systemd timeout coordination (95s)"
    - "INF-05: Production environment observability variables"
  affects:
    - "Infrastructure deployment playbooks"
    - "Production server configuration"
tech_stack:
  added:
    - "DigitalOcean metrics agent (do-agent)"
  patterns:
    - "Ansible idempotent task with args:creates"
    - "Nginx exact location match for security"
    - "Timeout coordination: app < Docker < systemd"
key_files:
  created:
    - "infrastructure/tasks/metrics-agent.yml"
  modified:
    - "infrastructure/templates/trustedge.nginx.conf.j2"
    - "infrastructure/templates/docker-compose.production.yml.j2"
    - "infrastructure/templates/trustedge.service.j2"
    - "infrastructure/templates/env.production.j2"
    - "infrastructure/playbooks/provision.yml"
    - "infrastructure/playbooks/resume-app.yml"
key_decisions:
  - decision: "Use exact location match (location = /metrics) not prefix match"
    rationale: "Prevents /metrics/anything from bypassing security restrictions"
    alternatives: ["location /metrics with trailing slash", "location ^~ /metrics prefix"]
    impact: "Tighter security — only exact /metrics endpoint accessible"
  - decision: "Frontend stop_grace_period 10s (not 90s like backend)"
    rationale: "Frontend is stateless Next.js — no long-running operations to drain"
    alternatives: ["Same 90s as backend", "Even shorter 5s"]
    impact: "Faster frontend restart without unnecessary delay"
  - decision: "systemd TimeoutStopSec 95s (Docker grace 90s + 5s buffer)"
    rationale: "Allows Docker Compose to gracefully stop containers before systemd kills the process"
    alternatives: ["Match exactly at 90s", "Larger buffer like 120s"]
    impact: "Graceful shutdown completes before systemd intervention"
  - decision: "Metrics agent in both provision.yml and resume-app.yml"
    rationale: "Ensures agent installed whether doing fresh provisioning or app-only updates"
    alternatives: ["Only in provision.yml", "Separate metrics playbook"]
    impact: "Agent always present after any playbook run"
metrics:
  duration_minutes: 1
  tasks_completed: 2
  files_created: 1
  files_modified: 6
  commits: 2
  completed_at: "2026-02-16T21:08:27Z"
---

# Phase 24 Plan 01: Ansible Template Configuration Summary

**One-liner:** Configure Ansible templates for production observability — JSON logging, localhost-restricted /metrics endpoint, graceful shutdown coordination (SIGTERM + 90s grace period), systemd timeout extension to 95s, and idempotent DigitalOcean metrics agent installation.

## What Was Built

Prepared all infrastructure-as-code changes for production observability deployment:

### Task 1: Ansible Template Updates

**Nginx template (trustedge.nginx.conf.j2):**
- Added `/metrics` location block with exact match (`location = /metrics`)
- Restricted access to localhost only: `allow 127.0.0.1`, `allow ::1`, `deny all`
- Configured reverse proxy to backend (127.0.0.1:3000/metrics)
- Disabled access logs to avoid high-frequency scraping noise

**Docker Compose template (docker-compose.production.yml.j2):**
- Added backend graceful shutdown: `stop_signal: SIGTERM`, `stop_grace_period: 90s`
- Added frontend graceful shutdown: `stop_signal: SIGTERM`, `stop_grace_period: 10s`
- Coordinated with application's SHUTDOWN_TIMEOUT=90 from Phase 23

**systemd service template (trustedge.service.j2):**
- Extended `TimeoutStopSec` from 30s to 95s
- Ensures systemd waits for Docker Compose graceful shutdown (90s) + buffer (5s)

**Production environment template (env.production.j2):**
- Added OBSERVABILITY CONFIGURATION section
- Set `LOG_FORMAT=json` for structured logging (Phase 19 integration)
- Set `SHUTDOWN_TIMEOUT=90` matching Docker's stop_grace_period

### Task 2: Metrics Agent Installation

**Created infrastructure/tasks/metrics-agent.yml:**
- Idempotent DigitalOcean metrics agent installation
- Checks if do-agent is already running before installation
- Uses `args: creates:` for shell task idempotency
- Verifies agent is active after installation

**Updated playbooks:**
- Added metrics-agent import to `provision.yml` (after systemd tasks)
- Added metrics-agent import to `resume-app.yml` (after systemd tasks)

## Requirements Satisfied

- **INF-01:** DigitalOcean metrics agent task file with idempotent installation ✓
- **INF-02:** Nginx /metrics location block restricted to localhost ✓
- **INF-03:** Docker Compose graceful shutdown (SIGTERM + grace periods) ✓
- **INF-04:** systemd TimeoutStopSec extended to 95s ✓
- **INF-05:** Production env template with LOG_FORMAT=json and SHUTDOWN_TIMEOUT=90 ✓

## Timeout Coordination Chain

```
Application SHUTDOWN_TIMEOUT: 90s
    ↓ (matches)
Docker stop_grace_period: 90s
    ↓ (+ 5s buffer)
systemd TimeoutStopSec: 95s
```

This ensures clean shutdown flow:
1. systemd sends SIGTERM to docker compose (via ExecStop)
2. Docker sends SIGTERM to backend container
3. Backend begins draining in-flight scans (up to 90s)
4. Docker waits 90s before sending SIGKILL
5. systemd waits 95s before killing docker compose process

## Deviations from Plan

None - plan executed exactly as written.

## Testing Notes

**Not tested in this plan (infrastructure changes only):**
- Ansible playbook syntax validation (Plan 02 will deploy and test)
- Nginx configuration reload after template application
- Docker Compose graceful shutdown behavior under load
- DigitalOcean metrics agent reporting to control panel

**Plan 02 deployment testing required:**
- Verify Nginx blocks external /metrics access (curl from external IP should 403)
- Verify localhost /metrics access works (curl from server should 200)
- Test graceful shutdown: start scan, docker stop, verify scan completes
- Confirm systemd doesn't kill docker compose before 95s timeout
- Verify do-agent reports metrics to DigitalOcean dashboard

## Key Files Changed

### Created (1)
- `infrastructure/tasks/metrics-agent.yml` - Idempotent do-agent installation

### Modified (6)
- `infrastructure/templates/trustedge.nginx.conf.j2` - /metrics endpoint with localhost restriction
- `infrastructure/templates/docker-compose.production.yml.j2` - Graceful shutdown signals and grace periods
- `infrastructure/templates/trustedge.service.j2` - Extended systemd timeout to 95s
- `infrastructure/templates/env.production.j2` - Added LOG_FORMAT and SHUTDOWN_TIMEOUT
- `infrastructure/playbooks/provision.yml` - Import metrics-agent tasks
- `infrastructure/playbooks/resume-app.yml` - Import metrics-agent tasks

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | 52cdbab | feat(24-01): configure Ansible templates for observability |
| 2 | bd0a1b2 | feat(24-01): add DigitalOcean metrics agent installation |

## What's Next

**Plan 02: Deploy observability infrastructure**
- Deploy to staging environment first
- Test /metrics security (external access blocked, localhost works)
- Validate graceful shutdown under load
- Verify DigitalOcean metrics agent reporting
- Deploy to production after staging validation

## Self-Check

Verifying all claims in this summary:

**Files created:**
- infrastructure/tasks/metrics-agent.yml ✓

**Files modified:**
- infrastructure/templates/trustedge.nginx.conf.j2 ✓
- infrastructure/templates/docker-compose.production.yml.j2 ✓
- infrastructure/templates/trustedge.service.j2 ✓
- infrastructure/templates/env.production.j2 ✓
- infrastructure/playbooks/provision.yml ✓
- infrastructure/playbooks/resume-app.yml ✓

**Commits exist:**
- 52cdbab ✓
- bd0a1b2 ✓

**Must-have truths verified:**
- Nginx /metrics has deny all ✓
- Docker backend has stop_grace_period 90s ✓
- systemd has TimeoutStopSec=95s ✓
- env.production has LOG_FORMAT=json ✓
- metrics-agent.yml exists with do-agent ✓
- Both playbooks import metrics-agent ✓

## Self-Check: PASSED
