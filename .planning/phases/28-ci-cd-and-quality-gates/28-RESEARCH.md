# Phase 28: CI/CD and Quality Gates - Research

**Researched:** 2026-02-17
**Domain:** GitHub Actions CI pipeline, Vitest coverage enforcement, Playwright E2E in CI, branch protection
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Workflow structure
- Single workflow file with parallel jobs (not separate workflow files)
- Trigger events: `pull_request` and `push` to `main`
- E2E job depends on unit test job passing first (not fully parallel)
- Production build (`npm run build`) runs inside the E2E job (not a separate artifact-passing job)
- Node.js 22 LTS

#### Caching & performance
- Cache `node_modules` directory (full cache, not just npm global store)
- Playwright browsers installed fresh each run (no caching)
- Clean Next.js build each time (no .next/cache persistence)

#### Coverage thresholds
- Keep roadmap-specified thresholds: 80% lines, 80% functions, 75% branches
- Enforce via Vitest config only (thresholds in vitest.config.ts, CI fails on non-zero exit)
- No coverage report upload as artifact
- Apply coverage to everything in frontend/ (not just components + lib)

#### Branch protection
- Require all CI checks to pass before merge (both unit and E2E jobs)
- Automate branch protection setup via `gh` CLI commands
- No admin bypass — strict enforcement, no exceptions
- CI checks only for merge — no review approval required (sole developer)

### Claude's Discretion
- Exact workflow job naming and step organization
- Cache key strategy (hash of package-lock.json, etc.)
- Playwright test artifact upload format for failures (screenshots + traces)
- Vitest reporter configuration for CI output

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CI-01 | GitHub Actions workflow running Vitest unit/component tests on every PR and push to main | GitHub Actions trigger syntax, `vitest run --coverage` in CI job |
| CI-02 | GitHub Actions workflow running Playwright E2E tests on every PR and push to main | Playwright CI docs, `npx playwright test` after build in same job |
| CI-03 | Vitest and Playwright jobs run in parallel for faster CI feedback | E2E job uses `needs: [unit-tests]` — sequenced, NOT parallel per decision |
| CI-04 | npm dependency caching and Playwright browser caching configured for CI performance | actions/cache@v5 for node_modules; no Playwright browser caching per decision |
| CI-05 | Playwright test artifacts (screenshots, traces) uploaded on test failure for debugging | actions/upload-artifact@v6 with `if: failure()` |
| CI-06 | PR merges blocked when any test job fails | `gh api` PUT branch protection with required status checks |
| QUAL-01 | Code coverage thresholds enforced: 80% lines, 80% functions, 75% branches | `coverage.thresholds` in vitest.config.ts; non-zero exit fails CI |
| QUAL-02 | Coverage reports generated in HTML and lcov formats | Already configured: `reporter: ['text', 'html', 'lcov']` in vitest.config.ts |
| QUAL-03 | CI fails when coverage drops below configured thresholds | Vitest exits non-zero on threshold failure; `test:ci` script propagates exit code |
</phase_requirements>

---

## Summary

This phase adds a GitHub Actions CI workflow and coverage enforcement to the existing ShipSecure frontend. The project already has Vitest and Playwright configured from phases 25–27. The CI work is primarily configuration — one YAML workflow file and additions to `vitest.config.ts`.

The key architectural decision is a single workflow file with two jobs: `unit-tests` (Vitest with coverage) and `e2e-tests` (build + Playwright), where e2e-tests depends on unit-tests passing first. This is NOT fully parallel (per user decision), but the unit tests complete fast and unblock e2e quickly. The full `node_modules` directory is cached using `actions/cache@v5` with an exact key on `package-lock.json` hash, skipping `npm ci` on cache hit. Playwright browsers are installed fresh every run per user decision (Playwright itself recommends against caching browsers).

Branch protection is set up via `gh api` (REST API) to require both job names as status checks with `enforce_admins: true`. This must be run once (manually or in a setup script), not as part of the CI workflow itself.

**Primary recommendation:** Write one workflow YAML file, add coverage thresholds to `vitest.config.ts`, and run `gh api` once to enforce branch protection.

---

## Standard Stack

### Core (already in project)

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| GitHub Actions | N/A (platform) | CI orchestration | Repo is on GitHub; no external CI needed |
| `actions/checkout` | v5 | Clone repo in CI | Already used in build-push.yml |
| `actions/setup-node` | v6 | Install Node.js | Latest major version; supports Node 22 LTS |
| `actions/cache` | v5 | Cache node_modules | Latest; runs on Node 24; requires runner ≥2.327.1 |
| `actions/upload-artifact` | v6 | Upload Playwright reports | Latest; runs on Node 24 |
| `gh` CLI | bundled in ubuntu-latest | Branch protection setup | Native to GitHub; handles REST API calls |

### No New npm Packages Needed

All npm packages are already installed from phases 25–27:
- `vitest`, `@vitest/coverage-v8` — unit testing + coverage
- `@playwright/test` — E2E testing
- Playwright v1.58.2 is the installed version

---

## Architecture Patterns

### Recommended Workflow File Location

```
.github/
└── workflows/
    ├── build-push.yml       # Existing Docker build + deploy
    └── ci.yml               # NEW: Test CI pipeline
```

### Pattern 1: Single Workflow File with Two Jobs

**What:** One `.github/workflows/ci.yml` with `unit-tests` and `e2e-tests` jobs. `e2e-tests` has `needs: [unit-tests]`.

**Why:** User decided single file; E2E depends on unit tests to avoid wasting resources.

```yaml
# Source: Derived from GitHub Actions docs + project decisions
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: frontend

    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v6
        with:
          node-version: '22'

      - name: Cache node_modules
        id: cache
        uses: actions/cache@v5
        with:
          path: frontend/node_modules
          key: ${{ runner.os }}-node22-${{ hashFiles('frontend/package-lock.json') }}

      - name: Install dependencies
        if: steps.cache.outputs.cache-hit != 'true'
        run: npm ci

      - name: Run unit tests with coverage
        run: npm run test:ci

  e2e-tests:
    name: E2E Tests
    runs-on: ubuntu-latest
    needs: [unit-tests]
    defaults:
      run:
        working-directory: frontend

    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v6
        with:
          node-version: '22'

      - name: Cache node_modules
        id: cache
        uses: actions/cache@v5
        with:
          path: frontend/node_modules
          key: ${{ runner.os }}-node22-${{ hashFiles('frontend/package-lock.json') }}

      - name: Install dependencies
        if: steps.cache.outputs.cache-hit != 'true'
        run: npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium

      - name: Build production app
        run: npm run build

      - name: Run E2E tests
        run: npm run test:e2e
        env:
          CI: true

      - name: Upload test artifacts on failure
        if: failure()
        uses: actions/upload-artifact@v6
        with:
          name: playwright-artifacts-${{ github.run_id }}
          path: |
            frontend/playwright-report/
            frontend/test-results/
          retention-days: 7
```

### Pattern 2: node_modules Cache with Exact Key (No Restore Keys)

**What:** Cache the full `node_modules` directory. Use only an exact key (no `restore-keys` fallback). Skip `npm ci` entirely on cache hit.

**Why the user wants this:** Avoids re-downloading all packages when `package-lock.json` hasn't changed. This is the fastest approach but requires NOT using `npm ci` when the cache hits (because `npm ci` deletes node_modules first).

**Critical:** Do NOT add `restore-keys` — a partial cache would give a stale node_modules with no install step, causing broken builds. Exact key only.

```yaml
- name: Cache node_modules
  id: cache
  uses: actions/cache@v5
  with:
    path: frontend/node_modules
    key: ${{ runner.os }}-node22-${{ hashFiles('frontend/package-lock.json') }}
    # No restore-keys — exact match only

- name: Install dependencies
  if: steps.cache.outputs.cache-hit != 'true'
  run: npm ci
```

**Cache invalidation:** Any change to `frontend/package-lock.json` produces a new hash, misses the cache, and runs `npm ci` fresh.

### Pattern 3: Playwright Artifacts on Failure

**What:** Upload `playwright-report/` and `test-results/` when e2e job fails.

**Condition options:**
- `if: failure()` — only on failure (most common, what we want)
- `if: ${{ !cancelled() }}` — on failure OR success (not just failure)

**Decision:** Use `if: failure()` since user only wants artifacts for debugging failures.

```yaml
- name: Upload test artifacts on failure
  if: failure()
  uses: actions/upload-artifact@v6
  with:
    name: playwright-artifacts-${{ github.run_id }}
    path: |
      frontend/playwright-report/
      frontend/test-results/
    retention-days: 7
```

**Playwright config already captures traces on retry** (`trace: 'on-first-retry'` in playwright.config.ts). In CI, `retries: 1` is set, so failing tests get a trace. Screenshots are NOT automatically captured by default — to get screenshots on failure, add `screenshot: 'only-on-failure'` to the `use:` block in playwright.config.ts.

### Pattern 4: Coverage Thresholds in vitest.config.ts

**What:** Add `thresholds` block to `coverage` section.

**Current state:** `vitest.config.ts` already has `provider: 'v8'` and `reporter: ['text', 'html', 'lcov']`. Thresholds are missing.

**Required addition:**

```typescript
// In vitest.config.ts, inside test.coverage:
thresholds: {
  lines: 80,
  functions: 80,
  branches: 75,
},
```

**Behavior:** When coverage is below threshold, `vitest run` exits with a non-zero exit code, printing:
```
ERROR: Coverage for lines (XX%) does not meet global threshold (80%)
```
This causes the CI job to fail automatically. No additional CI-level enforcement needed.

**QUAL-02 already met:** The existing `reporter: ['text', 'html', 'lcov']` satisfies the HTML and lcov format requirement.

### Pattern 5: Branch Protection via gh CLI

**What:** Configure GitHub branch protection on `main` to require both CI jobs to pass.

**Command (run once by developer, not in CI):**

```bash
gh api repos/TrustEdge-Labs/shipsecure/branches/main/protection \
  --method PUT \
  --header "Accept: application/vnd.github+json" \
  -f "required_status_checks[strict]=false" \
  -f "required_status_checks[checks][][context]=Unit Tests" \
  -f "required_status_checks[checks][][context]=E2E Tests" \
  -F "enforce_admins=true" \
  -F "required_pull_request_reviews=null" \
  -F "restrictions=null"
```

**Critical detail:** The `context` string in `checks` must exactly match the `name:` field of the GitHub Actions job. If the job is named `Unit Tests` in the YAML, then `context` must be `Unit Tests`. GitHub surfaces job names (not workflow names) as status check names.

**enforce_admins: true** — No admin bypass per user decision. Even pushing directly as repo owner will be blocked if CI fails.

**`strict: false`** — Branch does not need to be up-to-date before merging (not required by user; avoids forcing rebases).

**When to run:** After the CI workflow has run at least once (so GitHub knows the check names), then run the `gh api` command. Status checks only appear in the branch protection UI after they've been observed.

### Anti-Patterns to Avoid

- **Using `restore-keys` with node_modules cache:** A partial cache would restore a stale node_modules but the `if: steps.cache.outputs.cache-hit != 'true'` condition would skip install. Use exact key only.
- **Caching Playwright browsers:** Playwright explicitly recommends against it — restore time equals download time. User decision also says no.
- **Uploading coverage as artifact:** User decision says no. Don't add an artifact upload step for coverage HTML.
- **Running npm ci when cache hits:** `npm ci` deletes node_modules first. Only run when `cache-hit != 'true'`.
- **Using `if: always()` for artifact upload:** This uploads even on success, wasting storage. Use `if: failure()`.
- **Setting branch protection before workflow has run:** Required checks must appear in GitHub's check list before being settable. Workflow must run at least once first.
- **Using `actions/setup-node` cache instead of `actions/cache`:** `actions/setup-node` caches the npm global store (~/.npm), not node_modules. User explicitly wants node_modules cached.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Node.js version management | Custom nvm/fnm setup steps | `actions/setup-node@v6` | Handles PATH, version pinning, built-in |
| Cache management | Custom upload/download scripts | `actions/cache@v5` | Handles key matching, expiry, LFS |
| Artifact storage | Custom S3/external upload | `actions/upload-artifact@v6` | Built-in GitHub UI, no secrets needed |
| Branch protection | GitHub UI clicks or custom scripts | `gh api` REST call | Repeatable, documentable as code |

---

## Common Pitfalls

### Pitfall 1: Job Name vs. Workflow Name in Status Checks

**What goes wrong:** Developer configures branch protection with the workflow name ("CI") instead of the job name ("Unit Tests"). The check never matches.

**Why it happens:** GitHub exposes job-level names, not workflow names, as status checks.

**How to avoid:** Use the exact `name:` field of the job (e.g., `Unit Tests`, `E2E Tests`) as the `context` in the branch protection API call.

**Warning signs:** Branch protection shows check as "never been run" even after CI passes.

### Pitfall 2: Cache Hit Skips Install but node_modules Is Stale

**What goes wrong:** Developer adds a new package, forgets to commit `package-lock.json`, cache hits on the old hash, CI runs with old node_modules.

**Why it happens:** Cache key is based on `package-lock.json` hash, so if lock file isn't committed, the hash matches old cache.

**How to avoid:** Always commit `package-lock.json` changes alongside `package.json` changes. The cache correctly invalidates when the lock file changes.

### Pitfall 3: Playwright webServer Times Out in CI

**What goes wrong:** `npm run build && npm run start` takes too long; webServer timeout (120s) exceeded; E2E tests fail before running.

**Why it happens:** First build in CI with cold node_modules can be slow. Also, Next.js 16 with `output: 'standalone'` may need extra time.

**How to avoid:** Cache node_modules to speed up the build (dependencies already installed). The `timeout: 120_000` in playwright.config.ts should be sufficient. Monitor first CI run.

**Warning signs:** E2E job fails with `webServer failed to start` or timeout message.

### Pitfall 4: Branch Protection Set Before CI Has Run

**What goes wrong:** `gh api` call succeeds but the check names don't appear — GitHub says no matching checks available.

**Why it happens:** GitHub only registers status check names after they've been observed at least once. A workflow that has never run has no known check names.

**How to avoid:** Push the workflow file, let it run once on main or a PR, then run the `gh api` branch protection command.

### Pitfall 5: Coverage Thresholds Applied to Wrong Scope

**What goes wrong:** Coverage thresholds pass in CI but miss important code because the `include` in vitest.config.ts is too narrow.

**Why it happens:** Current vitest.config.ts has `include: ['components/**', 'lib/**', 'app/**']` — this covers the right scope. But user decision says "apply to everything in frontend/ (not just components + lib)" — the app/** inclusion already covers this.

**How to avoid:** Verify the `include` paths in vitest.config.ts match the intention. The existing config already includes `app/**`, which covers server actions and pages.

### Pitfall 6: NEXT_PUBLIC_ env vars missing at build time in E2E job

**What goes wrong:** `npm run build` fails or produces a broken build because `NEXT_PUBLIC_BACKEND_URL` is not set.

**Why it happens:** NEXT_PUBLIC_ variables are inlined at build time. If not set, Next.js uses `undefined`, which may cause issues.

**How to avoid:** The playwright.config.ts webServer env sets `NEXT_PUBLIC_BACKEND_URL: E2E_BASE_URL` but this is for the server START, not for BUILD. The build step needs env vars too.

**Solution:** Add env vars to the build step in the E2E job:
```yaml
- name: Build production app
  run: npm run build
  env:
    NEXT_PUBLIC_BACKEND_URL: http://localhost:3001
    NEXT_PUBLIC_SITE_URL: https://shipsecure.io
```
Or create a `.env.production.local` file before building. The local fallback `|| 'http://localhost:3000'` in the code means the build won't fail, but the URL will be wrong.

---

## Code Examples

### Complete Workflow File

```yaml
# Source: GitHub Actions docs + Playwright CI docs + project decisions
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: frontend

    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v6
        with:
          node-version: '22'

      - name: Cache node_modules
        id: cache
        uses: actions/cache@v5
        with:
          path: frontend/node_modules
          key: ${{ runner.os }}-node22-${{ hashFiles('frontend/package-lock.json') }}

      - name: Install dependencies
        if: steps.cache.outputs.cache-hit != 'true'
        run: npm ci

      - name: Run unit tests with coverage
        run: npm run test:ci

  e2e-tests:
    name: E2E Tests
    runs-on: ubuntu-latest
    needs: [unit-tests]
    defaults:
      run:
        working-directory: frontend

    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v6
        with:
          node-version: '22'

      - name: Cache node_modules
        id: cache
        uses: actions/cache@v5
        with:
          path: frontend/node_modules
          key: ${{ runner.os }}-node22-${{ hashFiles('frontend/package-lock.json') }}

      - name: Install dependencies
        if: steps.cache.outputs.cache-hit != 'true'
        run: npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium

      - name: Build production app
        run: npm run build
        env:
          NEXT_PUBLIC_BACKEND_URL: http://localhost:3001
          NEXT_PUBLIC_SITE_URL: https://shipsecure.io

      - name: Run E2E tests
        run: npm run test:e2e
        env:
          CI: true

      - name: Upload test artifacts on failure
        if: failure()
        uses: actions/upload-artifact@v6
        with:
          name: playwright-artifacts-${{ github.run_id }}
          path: |
            frontend/playwright-report/
            frontend/test-results/
          retention-days: 7
```

### vitest.config.ts Addition (thresholds)

```typescript
// Source: https://vitest.dev/config/coverage
// Add inside test.coverage block:
thresholds: {
  lines: 80,
  functions: 80,
  branches: 75,
},
```

Full updated coverage block:

```typescript
coverage: {
  provider: 'v8',
  reporter: ['text', 'html', 'lcov'],
  include: ['components/**', 'lib/**', 'app/**'],
  exclude: [
    'node_modules/',
    '__tests__/',
    '*.config.*',
    '.next/',
    'app/**/layout.tsx',
    'app/**/loading.tsx',
    'app/**/error.tsx',
    'app/**/global-error.tsx',
    'app/**/opengraph-image.tsx',
    'app/robots.ts',
    'app/sitemap.ts',
  ],
  thresholds: {
    lines: 80,
    functions: 80,
    branches: 75,
  },
},
```

### Branch Protection Setup Command

```bash
# Run once after CI workflow has been observed by GitHub (i.e., after first CI run)
# Source: GitHub REST API docs for branch protection
gh api repos/TrustEdge-Labs/shipsecure/branches/main/protection \
  --method PUT \
  --header "Accept: application/vnd.github+json" \
  -f "required_status_checks[strict]=false" \
  -f "required_status_checks[checks][][context]=Unit Tests" \
  -f "required_status_checks[checks][][context]=E2E Tests" \
  -F "enforce_admins=true" \
  -F "required_pull_request_reviews=null" \
  -F "restrictions=null"
```

### Playwright Config: Add Screenshot Capture on Failure

```typescript
// In playwright.config.ts, add to the `use:` block:
screenshot: 'only-on-failure',
```

This ensures screenshots are captured for upload when tests fail. Traces are already configured with `trace: 'on-first-retry'`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `actions/checkout@v3/v4` | `actions/checkout@v5` | 2024 | Node.js 24 runner; already used in project |
| `actions/setup-node@v3/v4` | `actions/setup-node@v6` | Oct 2024 | v6.0.0 limits auto-cache to npm only |
| `actions/cache@v3/v4` | `actions/cache@v5` | Late 2024 | Node.js 24; runner ≥ 2.327.1 required |
| `actions/upload-artifact@v3/v4` | `actions/upload-artifact@v6` | Late 2024 | Node.js 24 |
| Caching `~/.npm` store | Caching `node_modules` directly | N/A | Faster restore but requires exact key strategy |
| `vitest coverage` without thresholds | `coverage.thresholds` in config | Vitest 1.x+ | Declarative threshold enforcement |
| Branch protection via UI | `gh api` REST call | Available since 2021 | Repeatable, scriptable |

**Deprecated/outdated:**
- `microsoft/playwright-github-action`: Deprecated. Do not use. Use `npx playwright install` CLI directly.
- `setup-node` built-in `cache: 'npm'`: Caches `~/.npm` (global store), NOT `node_modules`. Not what user wants.

---

## Open Questions

1. **Coverage thresholds may fail on first CI run**
   - What we know: Current test suite exists from phases 25–27; coverage should be reasonable
   - What's unclear: Actual current coverage percentages — may be below 80%
   - Recommendation: Run `npm run test:ci` locally first and check current coverage numbers before adding thresholds. If below 80%, either fix coverage or temporarily lower thresholds.

2. **`actions/cache@v5` runner version requirement**
   - What we know: Requires Actions Runner ≥ 2.327.1; GitHub-hosted ubuntu-latest should meet this
   - What's unclear: Whether GitHub's ubuntu-latest runner version meets this requirement
   - Recommendation: Use `actions/cache@v5` — GitHub-hosted runners are typically up-to-date. If issues arise, fall back to `actions/cache@v4`.

3. **Playwright install time: `--with-deps` vs minimal deps**
   - What we know: `--with-deps chromium` installs Chromium + system dependencies. Some teams install only minimal deps for speed.
   - What's unclear: Actual install time on ubuntu-latest for just chromium
   - Recommendation: Use `npx playwright install --with-deps chromium` (official recommended approach). Only optimize if actual CI times are unacceptable.

4. **working-directory defaults and cache path**
   - What we know: Jobs use `defaults: run: working-directory: frontend`. But `actions/cache` `path` is relative to `$GITHUB_WORKSPACE` (repo root), not the working-directory.
   - What's unclear: None — this is confirmed behavior.
   - Recommendation: Cache path must be `frontend/node_modules` (repo-root-relative). The `run:` steps that use `npm ci` etc. will use the `frontend/` working directory correctly.

---

## Existing Project State (Critical Context)

### What already exists and does NOT need to be created:
- `frontend/vitest.config.ts` — exists, needs `thresholds` addition only
- `frontend/playwright.config.ts` — exists; may need `screenshot: 'only-on-failure'` added to `use:` block
- `frontend/package.json` scripts: `test:ci` (vitest run --coverage), `test:e2e` (playwright test) — both exist
- `.github/workflows/build-push.yml` — existing workflow; new workflow is additive, not replacing

### What needs to be created:
- `.github/workflows/ci.yml` — new file
- Branch protection setup (one-time `gh api` command, not a file)

### What needs to be modified:
- `frontend/vitest.config.ts` — add `thresholds` to coverage block
- `frontend/playwright.config.ts` — optionally add `screenshot: 'only-on-failure'` to `use:` block

---

## Sources

### Primary (HIGH confidence)
- GitHub Actions official docs — workflow syntax, trigger events, job dependencies
- https://playwright.dev/docs/ci — Playwright CI configuration, browser install, artifact upload
- https://playwright.dev/docs/ci-intro — Complete GitHub Actions workflow example
- https://vitest.dev/config/coverage — Coverage threshold configuration
- https://github.com/actions/cache — `actions/cache@v5` documentation, node_modules caching example
- https://github.com/actions/upload-artifact — `upload-artifact@v6`, `if: failure()` usage
- https://github.com/actions/setup-node — `setup-node@v6` documentation, v6.2.0 latest release
- https://docs.github.com/en/rest/branches/branch-protection — Branch protection REST API, checks array format
- `gh api repos/TrustEdge-Labs/shipsecure/branches/main/protection` — verified no existing protection

### Secondary (MEDIUM confidence)
- https://www.voorhoede.nl/en/blog/super-fast-npm-install-on-github-actions/ — node_modules caching with exact key + conditional install pattern

### Tertiary (LOW confidence)
- Various WebSearch results on Playwright CI optimization — not relied upon for recommendations

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified action versions from official repos; existing project tools confirmed
- Architecture: HIGH — official Playwright CI docs, GitHub Actions docs, verified API format
- Pitfalls: MEDIUM-HIGH — most verified from official docs; env var issue verified from code inspection
- Coverage thresholds: MEDIUM — Vitest docs confirmed threshold config; exact exit code behavior not shown in docs but behavior is well-known

**Research date:** 2026-02-17
**Valid until:** 2026-03-17 (stable tooling; action versions may update but patterns are stable)
