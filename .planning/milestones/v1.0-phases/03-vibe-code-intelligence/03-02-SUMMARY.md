---
phase: 03-vibe-code-intelligence
plan: 02
subsystem: scanner-engine
tags: [nuclei, vibe-code, docker, templates, framework-detection]

requires:
  - phase: 02
    provides: Docker container scanner infrastructure
  - phase: 01
    provides: Finding model and scanner architecture

provides:
  - Custom Nuclei templates for vibe-code vulnerabilities
  - Framework-aware vibe-code scanner module
  - Template selection based on detected framework/platform
  - False positive filtering for safe publishable keys

affects:
  - phase: 03
    plan: 04
    reason: Orchestrator will wire vibecode scanner with detector

tech-stack:
  added:
    - projectdiscovery/nuclei Docker image
  patterns:
    - Custom Nuclei template authoring
    - Framework-aware template selection
    - Docker volume mounting for templates
    - vibe_code tagging for UI differentiation

key-files:
  created:
    - templates/nuclei/supabase-rls.yaml
    - templates/nuclei/firebase-rules.yaml
    - templates/nuclei/nextjs-env-leak.yaml
    - templates/nuclei/unprotected-api-routes.yaml
    - templates/nuclei/env-in-build-output.yaml
    - templates/nuclei/netlify-function-exposure.yaml
    - templates/nuclei/vercel-env-leak.yaml
    - src/scanners/vibecode.rs
  modified:
    - src/scanners/mod.rs
    - src/scanners/container.rs

decisions:
  - id: custom-nuclei-templates
    what: Create 7 custom templates instead of relying on community templates
    why: Vibe-code vulnerabilities are specific to AI-generated apps and not well-covered by generic templates
    impact: TrustEdge differentiator - catches what other scanners miss

  - id: framework-aware-selection
    what: Select templates based on detected framework/platform
    why: Reduces noise and scan time - only run relevant templates
    impact: Faster scans, fewer false positives

  - id: vibe-code-tagging
    what: Tag all findings from this scanner with vibe_code=true
    why: UI can highlight vibe-code specific findings distinctly
    impact: Better user experience - users see AI-specific vulnerabilities clearly

  - id: safe-key-whitelist
    what: Filter NEXT_PUBLIC_SUPABASE_URL and NEXT_PUBLIC_SUPABASE_ANON_KEY from env leak findings
    why: These are safe publishable keys meant to be in client bundles per Supabase docs
    impact: Reduces false positives that would hurt credibility

  - id: docker-volume-mount
    what: Mount templates directory as read-only volume
    why: Security (read-only) + flexibility (can update templates without rebuilding image)
    impact: Templates can be version-controlled separately from scanner code

  - id: universal-vs-specific
    what: Always run universal templates (Supabase, Firebase, .env) plus framework-specific
    why: BaaS and env exposure are common across all vibe-coded apps
    impact: Comprehensive coverage even for unknown frameworks

metrics:
  duration: 335s
  completed: 2026-02-06
---

# Phase 03 Plan 02: Custom Nuclei Templates & Vibe-Code Scanner Summary

**One-liner:** Framework-aware vibe-code vulnerability scanner with 7 custom Nuclei templates for BaaS misconfigurations, env leaks, and unprotected API routes.

## Objective Achieved

Created TrustEdge's differentiator: custom Nuclei templates that detect vulnerabilities specific to AI-generated applications (vibe-coded apps). Implemented scanner module that orchestrates these templates with framework-aware selection, false positive filtering, and vibe_code tagging for UI highlighting.

## Task Commits

### Task 1: Custom Nuclei Templates (8cbfed4)
**Commit:** 8cbfed4
**Files:** 7 Nuclei YAML templates in templates/nuclei/

Created 7 custom Nuclei v3 templates covering vibe-code vulnerability patterns:

1. **supabase-rls.yaml** - Detects Supabase RLS misconfigurations
   - Extracts project URL and anon key from HTML/JS
   - Probes REST API with anon key for private data exposure
   - Severity: Critical

2. **firebase-rules.yaml** - Detects Firebase permissive security rules
   - Extracts Firebase project from HTML
   - Tests .json endpoint for unauthenticated read access
   - Severity: Critical

3. **nextjs-env-leak.yaml** - Detects NEXT_PUBLIC_ secret exposure
   - Scans client bundle for secret-like NEXT_PUBLIC_ variables
   - Whitelists safe publishable keys (Supabase URL/anon key)
   - Severity: High

4. **unprotected-api-routes.yaml** - Detects unprotected API endpoints
   - Probes common Next.js API paths (/api/users, /api/admin, etc.)
   - Tests App Router patterns (/api/user/route)
   - Severity: High

5. **env-in-build-output.yaml** - Detects .env files in build output
   - Checks for .env, .env.local, .env.production
   - Validates content has env patterns (KEY=value)
   - Severity: Critical

6. **netlify-function-exposure.yaml** - Detects Netlify function exposure
   - Tests /.netlify/functions/ for directory listings
   - Checks for debug endpoints revealing function names
   - Severity: Medium

7. **vercel-env-leak.yaml** - Detects Vercel environment leaks
   - Scans for VERCEL_ system vars in client bundles
   - Identifies deployment info disclosure
   - Severity: Low

All templates:
- Use Nuclei v3 syntax with matchers-condition
- Tagged with `vibe-code` for identification
- Authored by `trustedge-audit`
- Use passive + light active probing only (per Phase 3 scope)
- Include references to relevant documentation

### Task 2: Vibe-Code Scanner Module (c291ec3)
**Commit:** c291ec3
**Files:** src/scanners/vibecode.rs, src/scanners/mod.rs, src/scanners/container.rs

Implemented vibe-code scanner that orchestrates custom Nuclei templates:

**Core functionality:**
- `scan_vibecode(target_url, framework, platform)` - Main scanner entry point
- Takes optional framework/platform strings from detector (plan 03-01)
- Returns findings tagged with `vibe_code: true` and `scanner_name: "vibecode"`

**Framework-aware template selection:**
- Universal templates (always run): Supabase RLS, Firebase rules, env-in-build-output
- Next.js specific: nextjs-env-leak, unprotected-api-routes
- Vercel specific: vercel-env-leak
- Netlify specific: netlify-function-exposure
- Unknown framework: Run ALL templates (comprehensive coverage)

**Docker execution:**
- Uses projectdiscovery/nuclei:latest image
- Mounts templates directory as read-only volume
- CIS security hardening (8 mandatory flags from Phase 2 pattern)
- 120-second timeout for scan execution
- 30-second per-request timeout

**False positive filtering:**
- Whitelists NEXT_PUBLIC_SUPABASE_URL (safe publishable URL)
- Whitelists NEXT_PUBLIC_SUPABASE_ANON_KEY (safe publishable key)
- Filters findings matching these patterns from results
- Logs filtered findings at debug level

**Graceful degradation:**
- Checks Docker availability before execution
- Returns empty vec with warning if Docker unavailable
- Follows project convention from Phase 2 container scanners

**Template directory resolution:**
- Reads TRUSTEDGE_TEMPLATES_DIR env var if set
- Defaults to templates/nuclei/ in current working directory
- Errors if directory doesn't exist (fail-fast for misconfiguration)

**Unit tests (8 tests, all passing):**
- test_template_selection_nextjs - Verifies Next.js specific templates included
- test_template_selection_none - Verifies all templates run when framework unknown
- test_template_selection_vercel - Verifies Vercel template added
- test_template_selection_netlify - Verifies Netlify template added
- test_finding_parsing - Verifies JSON parsing and severity mapping
- test_whitelist_filtering_safe_keys - Verifies safe keys filtered
- test_whitelist_filtering_actual_secret - Verifies secrets not filtered
- test_templates_dir_from_env - Verifies env var override works

**Bug fix:**
- Fixed container.rs test assertions to use Severity enum instead of strings
- This was blocking compilation after vibe_code field addition in parallel plan 03-01

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed YAML syntax errors in Nuclei templates**
- **Found during:** Task 1, initial template validation
- **Issue:** Complex regex patterns with brackets and quotes caused YAML parsing errors
- **Fix:** Simplified regex patterns, used double quotes for consistency, split complex patterns into multiple simpler ones
- **Files modified:** firebase-rules.yaml, netlify-function-exposure.yaml, vercel-env-leak.yaml, nextjs-env-leak.yaml
- **Commit:** 8cbfed4 (included in initial commit after iterations)
- **Why safe:** YAML must be valid for Nuclei to parse templates - this was blocking basic functionality

**2. [Rule 3 - Blocking] Added vibe_code field to test Finding structs**
- **Found during:** Task 2, cargo test execution
- **Issue:** Parallel plan 03-01 added vibe_code field to Finding struct, breaking existing tests
- **Fix:** Added vibe_code: false to test Finding initializers in worker_pool.rs and fixed container.rs test assertions
- **Files modified:** src/orchestrator/worker_pool.rs, src/scanners/container.rs
- **Commit:** c291ec3
- **Why safe:** This is the same field plan 03-01 added - just fixing test compatibility

**3. [Rule 3 - Blocking] Added unsafe blocks for env var test**
- **Found during:** Task 2, cargo check
- **Issue:** std::env::set_var and remove_var require unsafe blocks in Rust
- **Fix:** Wrapped calls in unsafe {} blocks in test_templates_dir_from_env
- **Files modified:** src/scanners/vibecode.rs
- **Commit:** c291ec3
- **Why safe:** These are test-only functions, env var manipulation is inherently unsafe in multi-threaded contexts

## Integration Points

**Upstream dependencies:**
- Plan 03-01 (detector): Provides framework/platform detection results as Option<&str>
- Phase 02 container infrastructure: Reuses Docker execution patterns and security hardening

**Downstream consumers:**
- Plan 03-04 (orchestrator): Will call scan_vibecode() with detected framework/platform
- Frontend results dashboard: Will filter/highlight findings where vibe_code=true

**Parallel coordination:**
- Plan 03-01: Both modified src/scanners/mod.rs - clean merge (separate module declarations)
- Plan 03-03: Both modified src/scanners/mod.rs - clean merge (separate module declarations)
- Plan 03-01: Added vibe_code field to Finding struct - handled with test fixes

## Next Phase Readiness

**Blockers:** None

**Recommendations:**
1. Test templates against real vibe-coded apps before production
2. Monitor false positive rates and adjust whitelists as needed
3. Consider adding more safe key patterns as discovered
4. Update templates when Nuclei v3 syntax evolves

**Open questions:**
1. Should we version templates separately from scanner code?
2. How often should we update community templates?
3. Should we rate-limit Nuclei scans for very large sites?

## Performance Notes

**Execution time:** 335 seconds (5.6 minutes)
- Task 1: ~3 minutes (template creation + YAML syntax debugging)
- Task 2: ~2.6 minutes (scanner implementation + test fixes)

**Optimization opportunities:**
- Template selection logic could be data-driven (YAML config) vs hardcoded
- Could cache Docker image pull to speed up first-time execution

## Verification Evidence

```bash
# All 7 templates created
$ ls templates/nuclei/*.yaml | wc -l
7

# All tagged with vibe-code
$ grep -l "vibe-code" templates/nuclei/*.yaml | wc -l
7

# All findings tagged with vibe_code: true
$ grep "vibe_code: true" src/scanners/vibecode.rs
/// Vec of findings with `vibe_code: true` set on all results
            vibe_code: true,

# Tests pass
$ cargo test vibecode
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored

# Code compiles
$ cargo check
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Self-Check: PASSED

All created files exist:
✓ templates/nuclei/supabase-rls.yaml
✓ templates/nuclei/firebase-rules.yaml
✓ templates/nuclei/nextjs-env-leak.yaml
✓ templates/nuclei/unprotected-api-routes.yaml
✓ templates/nuclei/env-in-build-output.yaml
✓ templates/nuclei/netlify-function-exposure.yaml
✓ templates/nuclei/vercel-env-leak.yaml
✓ src/scanners/vibecode.rs

All commits exist:
✓ 8cbfed4 - Custom Nuclei templates
✓ c291ec3 - Vibe-code scanner module
