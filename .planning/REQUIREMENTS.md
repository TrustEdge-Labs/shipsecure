# Requirements: TrustEdge Audit

**Defined:** 2026-02-16
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.4 Requirements

Requirements for the Observability milestone. Each maps to roadmap phases.

### Logging

- [ ] **LOG-01**: Backend emits structured JSON logs when LOG_FORMAT=json (text mode for development)
- [ ] **LOG-02**: All log events include structured fields (timestamp, level, target, span context)
- [ ] **LOG-03**: Scan lifecycle events include scan_id, target_url, tier, scanner name
- [ ] **LOG-04**: Panic handler outputs structured JSON with backtrace

### Tracing

- [ ] **TRC-01**: Every HTTP request gets a unique correlation ID (request_id) via tower-http TraceLayer
- [ ] **TRC-02**: Request/response logs include method, URI, status code, and latency
- [ ] **TRC-03**: Background scan tasks inherit request span via .instrument()

### Metrics

- [ ] **MET-01**: Prometheus /metrics endpoint exposes metrics in OpenMetrics format
- [ ] **MET-02**: HTTP request counter (http_requests_total) with method, endpoint, status labels
- [ ] **MET-03**: HTTP request duration histogram (http_request_duration_seconds) with method, endpoint labels
- [ ] **MET-04**: Scan duration histogram (scan_duration_seconds) with tier, status labels and domain-appropriate buckets
- [ ] **MET-05**: Active scans gauge (active_scans) tracking in-flight scan count
- [ ] **MET-06**: Scan queue depth gauge (scan_queue_depth)
- [ ] **MET-07**: Scanner success/failure counters (scanner_results_total) with scanner, status labels
- [ ] **MET-08**: Rate limit counters (rate_limit_total) with limiter, action labels

### Health

- [ ] **HLT-01**: GET /health returns 200 "ok" instantly (shallow liveness check, unchanged)
- [ ] **HLT-02**: GET /health/ready returns JSON with DB connectivity, scan capacity, and overall status
- [ ] **HLT-03**: /health/ready returns 503 when database is unreachable

### Shutdown

- [ ] **SHD-01**: Backend handles SIGTERM and SIGINT signals for graceful shutdown
- [ ] **SHD-02**: In-flight scans complete before process exits (with configurable timeout)
- [ ] **SHD-03**: Background tasks tracked via TaskTracker replacing fire-and-forget tokio::spawn

### Infrastructure

- [ ] **INF-01**: DigitalOcean metrics agent installed via Ansible playbook
- [ ] **INF-02**: Nginx restricts /metrics endpoint to localhost only (deny all external)
- [ ] **INF-03**: Docker Compose configured with STOPSIGNAL, stop_grace_period, and JSON log rotation
- [ ] **INF-04**: systemd TimeoutStopSec extended to accommodate scan drain timeout
- [ ] **INF-05**: LOG_FORMAT=json set in production environment configuration

## Future Requirements

### Distributed Tracing

- **DTRC-01**: OpenTelemetry integration with OTLP exporter for cross-service tracing
- **DTRC-02**: Trace context propagation to external API calls (Stripe, SSL Labs, Resend)

### Alerting

- **ALT-01**: Prometheus AlertManager rules for error rate, queue depth, DB connections
- **ALT-02**: Slack/email notifications for critical alerts

### Log Aggregation

- **LAGR-01**: Log shipping to centralized aggregation system (Grafana Loki, Datadog, ELK)
- **LAGR-02**: Log-based alerting for error patterns

### Advanced Metrics

- **AMET-01**: Payment flow metrics (Stripe webhook counters, paid scan initiated)
- **AMET-02**: Email delivery metrics (sent/failed counters by type)
- **AMET-03**: Process-level metrics (CPU, memory, open connections via prometheus process collector)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Self-hosted Prometheus server | Overkill for single droplet. Expose /metrics, add Grafana Cloud later. |
| OpenTelemetry / distributed tracing | No external collector (Jaeger/Tempo). Add when multi-service. |
| In-app dashboards | Use Grafana when ready. Don't build custom visualization. |
| Custom metrics format | Prometheus is industry standard. |
| Logging to files | Container environments expect stdout/stderr. |
| tokio-console | Dev-only tool, not production observability. |
| Vendor-specific APM (Sentry, Datadog) | Structured logs + Prometheus is vendor-neutral. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| LOG-01 | — | Pending |
| LOG-02 | — | Pending |
| LOG-03 | — | Pending |
| LOG-04 | — | Pending |
| TRC-01 | — | Pending |
| TRC-02 | — | Pending |
| TRC-03 | — | Pending |
| MET-01 | — | Pending |
| MET-02 | — | Pending |
| MET-03 | — | Pending |
| MET-04 | — | Pending |
| MET-05 | — | Pending |
| MET-06 | — | Pending |
| MET-07 | — | Pending |
| MET-08 | — | Pending |
| HLT-01 | — | Pending |
| HLT-02 | — | Pending |
| HLT-03 | — | Pending |
| SHD-01 | — | Pending |
| SHD-02 | — | Pending |
| SHD-03 | — | Pending |
| INF-01 | — | Pending |
| INF-02 | — | Pending |
| INF-03 | — | Pending |
| INF-04 | — | Pending |
| INF-05 | — | Pending |

**Coverage:**
- v1.4 requirements: 26 total
- Mapped to phases: 0
- Unmapped: 26 ⚠️

---
*Requirements defined: 2026-02-16*
*Last updated: 2026-02-16 after initial definition*
