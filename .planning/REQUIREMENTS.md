# Requirements: ShipSecure

**Defined:** 2026-03-01
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.8 Requirements

Requirements for CI & Quality Hardening milestone.

### CI Pipeline

- [x] **CI-01**: Backend tests (cargo test) run on every push and PR to main
- [x] **CI-02**: Cargo clippy runs with zero warnings on every push and PR
- [x] **CI-03**: Cargo fmt --check enforces formatting on every push and PR
- [x] **CI-04**: Backend test coverage is reported in CI (cargo llvm-cov or tarpaulin)

### Infrastructure

- [x] **INFRA-01**: Docker healthcheck on backend container validates /health endpoint
- [x] **INFRA-02**: Docker healthcheck on frontend container validates HTTP response

### Test Coverage

- [x] **TEST-01**: Unit tests for domain-badge component
- [x] **TEST-02**: Unit tests for meta-tag-snippet component
- [x] **TEST-03**: Unit tests for scan-history-table component

### Documentation

- [x] **DOC-01**: README reflects correct Next.js version (16, not 15)

## Future Requirements

None — this is a hardening milestone.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Skill scanning | Separate v2.0 milestone after user feedback |
| New scanners or detections | Hardening only, no new features |
| Backend coverage thresholds | Report first, set thresholds after baseline established |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CI-01 | Phase 39 | Complete |
| CI-02 | Phase 39 | Complete |
| CI-03 | Phase 39 | Complete |
| CI-04 | Phase 39 | Complete |
| INFRA-01 | Phase 40 | Complete |
| INFRA-02 | Phase 40 | Complete |
| DOC-01 | Phase 40 | Complete |
| TEST-01 | Phase 41 | Complete |
| TEST-02 | Phase 41 | Complete |
| TEST-03 | Phase 41 | Complete |

**Coverage:**
- v1.8 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-03-01*
*Last updated: 2026-03-01 after roadmap creation — all 10 requirements mapped*
