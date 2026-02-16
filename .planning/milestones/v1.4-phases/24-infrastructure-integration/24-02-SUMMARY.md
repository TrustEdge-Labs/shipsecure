---
phase: 24-infrastructure-integration
plan: 02
subsystem: infrastructure
tags:
  - deployment
  - verification
  - production
  - observability
dependency_graph:
  requires:
    - "24-01: Ansible template configuration"
  provides:
    - "INF-01 through INF-05: All observability requirements verified in production"
  affects:
    - "Production server configuration"
key_files:
  modified:
    - "src/api/metrics.rs"
key_decisions:
  - decision: "Remove app-level IP check on /metrics endpoint"
    rationale: "Docker networking makes is_loopback() unreliable — containers see bridge IP, not 127.0.0.1. Nginx + Docker port binding provide sufficient access control."
    alternatives: ["Allow Docker bridge subnet", "Use X-Forwarded-For header"]
    impact: "/metrics accessible from within Docker network, secured by Nginx deny-all and Docker localhost-only port binding"
metrics:
  duration_minutes: 30
  tasks_completed: 2
  files_created: 0
  files_modified: 1
  commits: 1
  completed_at: "2026-02-16T23:15:00Z"
---

# Phase 24 Plan 02: Production Deployment & Verification Summary

**One-liner:** Deployed observability infrastructure to production via Ansible, fixed Docker-incompatible metrics IP check, and verified all 7 INF requirements pass on the live server.

## What Was Done

### Task 1: Production Deployment

Ansible playbook deployed all Phase 24 configuration changes:
- Nginx config with `/metrics` localhost restriction
- Docker Compose with SIGTERM + 90s/10s grace periods + JSON log rotation
- systemd service with TimeoutStopSec=95s
- Production .env with LOG_FORMAT=json and SHUTDOWN_TIMEOUT=90
- DigitalOcean metrics agent (do-agent)

### Bug Fix: Metrics Endpoint IP Check

**Problem:** `src/api/metrics.rs` used `ConnectInfo<SocketAddr>` with `addr.ip().is_loopback()` to restrict access. Under Docker networking, the container sees the Docker bridge IP (172.18.0.x), not 127.0.0.1, causing all requests to return 403 Forbidden — even from the host's localhost.

**Fix:** Removed the application-level IP check entirely. Access control is handled by two infrastructure layers:
1. Docker: port 3000 bound to `127.0.0.1` only (inaccessible externally)
2. Nginx: `/metrics` location with `deny all` except `127.0.0.1` and `::1`

### Task 2: Production Verification

All 7 success criteria verified on the live production server:

| # | Requirement | Verification | Result |
|---|-------------|-------------|--------|
| 1 | INF-01: Metrics agent | `systemctl is-active do-agent` → `active` | PASS |
| 2 | INF-02: /metrics blocked externally | `curl https://shipsecure.ai/metrics` → `403` | PASS |
| 3 | INF-02: /metrics works locally | `curl http://127.0.0.1:3000/metrics` → Prometheus text | PASS |
| 4 | INF-03: Docker SIGTERM | `docker inspect` → `SIGTERM` | PASS |
| 5 | INF-03: Docker log rotation | `{json-file map[max-file:3 max-size:10m]}` | PASS |
| 6 | INF-03: Grace periods | `90s` backend, `10s` frontend | PASS |
| 7 | INF-04: systemd timeout | `TimeoutStopUSec=1min 35s` (95s) | PASS |
| 8 | INF-05: LOG_FORMAT=json | Confirmed in .env + structured JSON log output | PASS |
| 9 | Health liveness | `{"status":"ok"}` with 200 | PASS |
| 10 | Health readiness | `db_connected: true`, scan capacity 0/10 | PASS |

## Deviations from Plan

1. **Metrics endpoint fix required** — The application-level `is_loopback()` check was incompatible with Docker networking. Removed in commit f0731c4 and redeployed via CI.

## Requirements Satisfied

- **INF-01:** DigitalOcean metrics agent active and reporting ✓
- **INF-02:** /metrics returns 403 externally, Prometheus text locally ✓
- **INF-03:** Docker Compose SIGTERM + 90s grace + JSON log rotation ✓
- **INF-04:** systemd TimeoutStopSec=95s ✓
- **INF-05:** LOG_FORMAT=json producing structured JSON logs ✓

## Commits

| Task | Commit | Message |
|------|--------|---------|
| Fix | f0731c4 | fix: remove app-level IP check on /metrics endpoint |

## v1.4 Observability Milestone: COMPLETE

All 6 phases (19-24) are now verified in production:
- Phase 19: Structured JSON logging ✓
- Phase 20: Request tracing with correlation IDs ✓
- Phase 21: Health check endpoints ✓
- Phase 22: Prometheus metrics ✓
- Phase 23: Graceful shutdown ✓
- Phase 24: Infrastructure integration ✓
