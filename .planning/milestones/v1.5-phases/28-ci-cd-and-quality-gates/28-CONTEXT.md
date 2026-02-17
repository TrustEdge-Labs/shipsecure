# Phase 28: CI/CD and Quality Gates - Context

**Gathered:** 2026-02-17
**Status:** Ready for planning

<domain>
## Phase Boundary

GitHub Actions CI pipeline that runs Vitest unit/component tests and Playwright E2E tests on every PR and push to main, with coverage enforcement and branch protection preventing merges on failure.

</domain>

<decisions>
## Implementation Decisions

### Workflow structure
- Single workflow file with parallel jobs (not separate workflow files)
- Trigger events: `pull_request` and `push` to `main`
- E2E job depends on unit test job passing first (not fully parallel)
- Production build (`npm run build`) runs inside the E2E job (not a separate artifact-passing job)

### Caching & performance
- Cache `node_modules` directory (full cache, not just npm global store)
- Playwright browsers installed fresh each run (no caching)
- Clean Next.js build each time (no .next/cache persistence)
- Node.js 22 LTS

### Coverage thresholds
- Keep roadmap-specified thresholds: 80% lines, 80% functions, 75% branches
- Enforce via Vitest config only (thresholds in vitest.config.ts, CI fails on non-zero exit)
- No coverage report upload as artifact
- Apply coverage to everything in frontend/ (not just components + lib)

### Branch protection
- Require all CI checks to pass before merge (both unit and E2E jobs)
- Automate branch protection setup via `gh` CLI commands
- No admin bypass — strict enforcement, no exceptions
- CI checks only for merge — no review approval required (sole developer)

### Claude's Discretion
- Exact workflow job naming and step organization
- Cache key strategy (hash of package-lock.json, etc.)
- Playwright test artifact upload format for failures (screenshots + traces)
- Vitest reporter configuration for CI output

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches for GitHub Actions CI configuration.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 28-ci-cd-and-quality-gates*
*Context gathered: 2026-02-17*
