# ShipSecure Retrospective

## Milestone: v1.8 — CI & Quality Hardening

**Shipped:** 2026-03-02
**Phases:** 3 | **Plans:** 3

### What Was Built
- Backend CI pipeline (cargo fmt, clippy, test, llvm-cov) in GitHub Actions
- Docker healthcheck directives on both production containers with service_healthy startup ordering
- 30 new unit tests for 3 previously excluded v1.6 components
- Coverage thresholds enforced across all active components (88.75/89.22/84.9%)

### What Worked
- Small, focused milestone (3 phases, 3 plans) completed in a single session
- All 3 phases had zero dependencies — could have run fully parallel
- Phase verification caught everything on first pass (no gap closure needed)
- 10/10 requirements satisfied cleanly with no rework

### What Was Inefficient
- v1.6 components were excluded from coverage when first added — should have been tested alongside the feature (phases 29-35)
- README inaccuracy (Next.js 15 → 16) could have been caught during the v1.5 testing milestone
- Minor: `-- --skip ignored` redundant flag in llvm-cov command (harmless but noisy)

### Patterns Established
- `fireEvent.click` + `vi.spyOn` for clipboard tests in happy-dom (not userEvent.click)
- `vi.useFakeTimers()` + `vi.setSystemTime()` for date-dependent component tests
- `makeScan()` factory with `Partial<T>` overrides for test data generation
- Healthchecks in docker-compose.prod.yml (not Dockerfiles) for environment-specific tuning
- Backend CI runs independently from frontend jobs (no cross-dependency)

### Key Lessons
- Test new components at the time of creation, not retroactively — coverage exclusions are tech debt
- Docker healthchecks should be standard practice from the first production deploy
- cargo fmt as first CI gate (fastest fail) saves resources

### Cost Observations
- Model mix: 100% sonnet (executor + verifier agents)
- Sessions: 1 (execute + audit + complete in single session)
- Notable: Smallest milestone yet — 3 plans completed in ~15 min total execution

---

## Cross-Milestone Trends

| Milestone | Phases | Plans | Days | Key Theme |
|-----------|--------|-------|------|-----------|
| v1.0 | 4 | 23 | 3 | Core product |
| v1.1 | 3 | 10 | 3 | Infrastructure |
| v1.2 | 5 | 10 | 2 | Polish |
| v1.3 | 6 | 10 | 7 | Design |
| v1.4 | 6 | 11 | 1 | Observability |
| v1.5 | 4 | 11 | 2 | Testing |
| v1.6 | 7 | 13 | 2 | Auth/access |
| v1.7 | 3 | 7 | 1 | UX polish |
| v1.8 | 3 | 3 | 1 | CI hardening |

**Velocity trend:** Milestones getting faster as the product matures. Infrastructure and quality work (v1.4-v1.8) ships in 1-2 days vs feature work (v1.0, v1.6) at 2-3 days.

**Quality trend:** Zero gap-closure phases needed since v1.4. Phase verifications consistently pass on first run.
