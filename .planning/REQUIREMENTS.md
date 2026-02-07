# Requirements: TrustEdge Audit

**Defined:** 2026-02-06
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.1 Requirements

Requirements for DigitalOcean deployment milestone. Each maps to roadmap phases.

### Infrastructure

- [ ] **INFRA-01**: Single DigitalOcean droplet provisioned with Ubuntu, Docker, and PostgreSQL
- [x] **INFRA-02**: Nuclei installed as native binary and executed as subprocess (no Docker-in-Docker)
- [x] **INFRA-03**: Production environment variables and secrets managed securely (not in code/git)

### Reverse Proxy

- [ ] **PROXY-01**: Nginx configured as reverse proxy routing to Rust backend and Next.js frontend
- [ ] **PROXY-02**: Let's Encrypt SSL certificate provisioned and auto-renewed via certbot

### Process Management

- [ ] **PROC-01**: Systemd service units for backend and frontend with auto-start on boot and restart on failure

### Security

- [ ] **SEC-01**: UFW firewall configured allowing only ports 22, 80, and 443

### Cleanup

- [x] **CLEAN-01**: All Render-specific configuration, environment variables, and documentation references removed

## Future Requirements

Deferred to later milestones. Tracked but not in current roadmap.

### Monitoring

- **MON-01**: Application health monitoring and alerting
- **MON-02**: Log aggregation and search

### Scaling

- **SCALE-01**: Separate worker droplet for scan execution
- **SCALE-02**: Load balancer for horizontal scaling

## Out of Scope

| Feature | Reason |
|---------|--------|
| Kubernetes/container orchestration | Single droplet sufficient for MVP scale |
| CI/CD pipeline | Manual deployment sufficient for now |
| Multi-region deployment | Single region sufficient for initial users |
| Managed database (DO Managed PostgreSQL) | Local PostgreSQL on droplet is simpler and cheaper for MVP |
| Docker Swarm/Compose for app services | Systemd is simpler for 2 services |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLEAN-01 | Phase 05 | Complete |
| INFRA-02 | Phase 05 | Complete |
| INFRA-03 | Phase 05 | Complete |
| INFRA-01 | Phase 06 | Pending |
| PROXY-01 | Phase 06 | Pending |
| PROXY-02 | Phase 06 | Pending |
| PROC-01 | Phase 06 | Pending |
| SEC-01 | Phase 06 | Pending |

**Coverage:**
- v1.1 requirements: 8 total
- Mapped to phases: 8
- Unmapped: 0

**100% coverage achieved** ✓

---
*Requirements defined: 2026-02-06*
*Last updated: 2026-02-07 after Phase 05 completion*
