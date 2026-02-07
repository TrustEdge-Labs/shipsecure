# Roadmap: TrustEdge Audit v1.1

**Milestone:** v1.1 DigitalOcean Deployment
**Created:** 2026-02-06
**Phases:** 3 (continuing from v1.0 phase 04)
**Coverage:** 8/8 requirements mapped (100%)

---

## Phase 05: Codebase Preparation

**Goal:** Application code is deployment-ready with Render references removed, Nuclei running as subprocess, and production configuration externalized.

**Requirements:**
- **CLEAN-01**: All Render-specific configuration, environment variables, and documentation references removed
- **INFRA-02**: Nuclei installed as native binary and executed as subprocess (no Docker-in-Docker)
- **INFRA-03**: Production environment variables and secrets managed securely (not in code/git)

**Success Criteria:**
1. Zero references to "Render" exist in codebase (code, config files, documentation)
2. Nuclei scanner executes as subprocess using installed binary (not Docker container)
3. Application starts successfully with environment variables loaded from external file
4. All secrets and API keys are externalized (no hardcoded values in source)

**Dependencies:** None (can start immediately)

**Plans:** 4 plans

Plans:
- [x] 05-01-PLAN.md — Refactor scanners from Docker to native binary subprocess execution
- [x] 05-02-PLAN.md — Externalize configuration with fail-fast validation and comprehensive .env.example
- [x] 05-03-PLAN.md — Update Docker Compose and Dockerfiles for full-stack dev/prod deployment
- [x] 05-04-PLAN.md — Update README and clean up Render hosting references in documentation

**Phase directory:** `.planning/phases/05-codebase-preparation/`

---

## Phase 06: Deployment Infrastructure

**Goal:** Production infrastructure is provisioned and configured on DigitalOcean with reverse proxy, SSL, process management, and firewall hardening.

**Requirements:**
- **INFRA-01**: Single DigitalOcean droplet provisioned with Ubuntu, Docker, and PostgreSQL
- **PROXY-01**: Nginx configured as reverse proxy routing to Rust backend and Next.js frontend
- **PROXY-02**: Let's Encrypt SSL certificate provisioned and auto-renewed via certbot
- **PROC-01**: Systemd service units for backend and frontend with auto-start on boot and restart on failure
- **SEC-01**: UFW firewall configured allowing only ports 22, 80, and 443

**Success Criteria:**
1. DigitalOcean droplet is running with Docker, PostgreSQL, and Nuclei binary installed
2. Nginx reverse proxy routes HTTPS requests to correct backend/frontend services
3. Let's Encrypt SSL certificate is active and configured for automatic renewal
4. Backend and frontend services auto-start on system boot and restart on crash
5. UFW firewall blocks all ports except 22 (SSH), 80 (HTTP), and 443 (HTTPS)

**Dependencies:** Phase 05 (requires Render-free codebase and subprocess Nuclei)

**Plans:** 4 plans

Plans:
- [ ] 06-01-PLAN.md — Update docker-compose.prod.yml and scaffold Ansible infrastructure project
- [ ] 06-02-PLAN.md — Ansible tasks for droplet provisioning, SSH hardening, UFW firewall, Docker installation
- [ ] 06-03-PLAN.md — Ansible tasks for Nginx reverse proxy, Certbot SSL, app deployment, systemd service
- [ ] 06-04-PLAN.md — Configure secrets, run playbook, verify production infrastructure

**Phase directory:** `.planning/phases/06-deployment-infrastructure/`

---

## Phase 07: Production Validation

**Goal:** Deployed application is verified working end-to-end in production environment with all critical workflows tested.

**Requirements:**
None (validation phase - verifies all previous requirements work together)

**Success Criteria:**
1. User can access application via HTTPS with valid SSL certificate (no browser warnings)
2. Free scan completes successfully and delivers results via email
3. Paid scan checkout flow completes and delivers PDF report via email
4. All five scanners (headers, TLS, secrets, files, vibe-code) execute and return findings
5. Services automatically recover after manual restart or simulated crash

**Dependencies:** Phase 06 (requires fully deployed infrastructure)

**Phase directory:** `.planning/phases/07-production-validation/`

---

## Progress

| Phase | Status | Plans | Requirements |
|-------|--------|-------|--------------|
| 05 - Codebase Preparation | Complete | 4/4 | CLEAN-01, INFRA-02, INFRA-03 |
| 06 - Deployment Infrastructure | Planned | 4 plans | INFRA-01, PROXY-01, PROXY-02, PROC-01, SEC-01 |
| 07 - Production Validation | Pending | 0/0 | (verification phase) |

---

## Coverage

| Requirement | Phase | Description |
|-------------|-------|-------------|
| CLEAN-01 | Phase 05 | Remove all Render references from codebase and docs |
| INFRA-02 | Phase 05 | Refactor Nuclei from Docker container to subprocess execution |
| INFRA-03 | Phase 05 | Externalize environment variables and secrets |
| INFRA-01 | Phase 06 | Provision DigitalOcean droplet with dependencies |
| PROXY-01 | Phase 06 | Configure Nginx reverse proxy |
| PROXY-02 | Phase 06 | Provision and auto-renew Let's Encrypt SSL |
| PROC-01 | Phase 06 | Create systemd service units |
| SEC-01 | Phase 06 | Configure UFW firewall |

**Coverage:** 8/8 requirements mapped (100%) ✓

---

## Notes

**Phase ordering rationale:**
- Phase 05 prepares codebase for deployment (removes Render, enables subprocess Nuclei)
- Phase 06 provisions and configures all infrastructure components
- Phase 07 validates end-to-end functionality in production

**Depth calibration:**
- Config setting: "quick" (3-5 phases)
- Applied: 3 phases for 8 requirements
- Rationale: Infrastructure work naturally groups into preparation → provisioning → validation

**No orphaned requirements:** All 8 v1.1 requirements mapped to exactly one phase.

---

*Created: 2026-02-06*
*Milestone: v1.1 DigitalOcean Deployment*
