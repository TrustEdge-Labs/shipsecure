---
phase: 03-vibe-code-intelligence
plan: 01
subsystem: scanner-engine
tags: [rust, scraper, detection, framework-fingerprinting, nextjs, vite, sveltekit, nuxt, vercel, netlify, railway]

# Dependency graph
requires:
  - phase: 02-free-tier-mvp
    provides: Scanner infrastructure, Finding and Scan models, database schema
provides:
  - Multi-signal framework detection engine (Next.js, Vite/React, SvelteKit, Nuxt)
  - Platform detection from response headers (Vercel, Netlify, Railway)
  - Detection model types (Framework, Platform, DetectionResult)
  - Extended Finding model with vibe_code boolean tag
  - Extended Scan model with detection columns and new stage tracking
  - Database migration for detection persistence
affects: [03-02-vibe-code-scanner, 03-03-remediation-engine, 04-monetization]

# Tech tracking
tech-stack:
  added: [scraper v0.22]
  patterns:
    - Weighted scoring for multi-signal detection (60+ confidence threshold)
    - Framework disambiguation (Vite/React excluded when Next.js detected)
    - Definitive platform detection from response headers

key-files:
  created:
    - src/models/detection.rs
    - src/scanners/detector.rs
    - migrations/20260205100001_add_detection_and_vibecode.sql
  modified:
    - src/models/finding.rs
    - src/models/scan.rs
    - src/models/mod.rs
    - src/db/scans.rs
    - src/db/findings.rs
    - src/scanners/mod.rs
    - All scanner files (security_headers, tls, exposed_files, js_secrets, container)

key-decisions:
  - "60+ confidence threshold requires 2+ signals for framework detection"
  - "Weighted scoring: STRONG signals 40 pts, MEDIUM 20-30 pts, LOW 10 pts"
  - "Vite/React detection disabled when Next.js scores above threshold (disambiguation)"
  - "Platform detection definitive (100 confidence) from x-vercel-id, x-nf-request-id, x-railway-request-id headers"
  - "vibe_code boolean tag on Finding for UI badge display"
  - "Detection columns stored as VARCHAR(50) for framework/platform names"

patterns-established:
  - "Multi-signal detection with weighted scoring pattern"
  - "Framework disambiguation logic to prevent false positives"
  - "HTML parsing with scraper crate for DOM signal extraction"
  - "Atomic Finding struct extensions require updating all constructors"

# Metrics
duration: 7min
completed: 2026-02-06
---

# Phase 3 Plan 1: Framework and Platform Detection Engine

**Multi-signal framework detection engine with weighted scoring (Next.js, Vite/React, SvelteKit, Nuxt) and definitive platform detection from response headers**

## Performance

- **Duration:** 7 minutes 27 seconds
- **Started:** 2026-02-06T04:06:13Z
- **Completed:** 2026-02-06T04:13:40Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments
- Detection model types with Framework/Platform enums and serialization
- Multi-signal detector scanner with 60+ confidence threshold prevents false positives
- Weighted scoring system: STRONG (40 pts), MEDIUM (20-30 pts), LOW (10 pts)
- Framework disambiguation logic (Vite/React excluded when Next.js detected)
- Platform detection from definitive response headers (100% confidence)
- Finding model extended with vibe_code tag for UI badges
- Scan model extended with detection columns and stage tracking
- All existing Finding constructors updated with vibe_code: false
- Database migration ready for detection persistence
- 5 unit tests passing for detection scenarios

## Task Commits

Each task was committed atomically:

1. **Task 1: Detection models, finding extension, and database migration** - `a16bb0e` (feat)
2. **Task 2: Multi-signal framework and platform detector** - `aebce53` (feat)

## Files Created/Modified

**Created:**
- `src/models/detection.rs` - Framework, Platform, DetectionResult enums with Display/serialization
- `src/scanners/detector.rs` - Multi-signal detection engine with weighted scoring
- `migrations/20260205100001_add_detection_and_vibecode.sql` - Detection columns and vibe_code tag

**Modified:**
- `src/models/finding.rs` - Added vibe_code: bool field
- `src/models/scan.rs` - Added detected_framework, detected_platform, stage_detection, stage_vibecode fields
- `src/models/mod.rs` - Re-exported detection types
- `src/db/scans.rs` - Updated all SQL queries for new columns
- `src/db/findings.rs` - Updated INSERT and SELECT queries for vibe_code
- `src/scanners/mod.rs` - Added detector module
- `src/scanners/security_headers.rs` - Added vibe_code: false to Finding constructor
- `src/scanners/tls.rs` - Added vibe_code: false to 10 Finding constructors
- `src/scanners/exposed_files.rs` - Added vibe_code: false to 2 Finding constructors
- `src/scanners/js_secrets.rs` - Added vibe_code: false to Finding constructor
- `src/scanners/container.rs` - Added vibe_code: false to 2 Finding constructors
- `src/scanners/aggregator.rs` - Added vibe_code: false to test Finding constructor
- `src/orchestrator/worker_pool.rs` - Added vibe_code: false to 2 test Finding constructors
- `Cargo.toml` - Added scraper v0.22 dependency

## Decisions Made

1. **60+ confidence threshold for framework detection** - Requires 2+ signals to prevent false positives from single indicators. High bar ensures only confident detections are shown to users.

2. **Weighted scoring system** - STRONG signals (40 pts): __NEXT_DATA__, __NUXT__, data-sveltekit. MEDIUM signals (20-30 pts): /_next/static, import.meta. LOW signals (10 pts): meta generator tags, x-powered-by headers. Reflects reliability of each indicator.

3. **Framework disambiguation** - Vite/React detection disabled when Next.js scores above threshold. Prevents false positive since Next.js uses React internally.

4. **Platform detection confidence levels** - Definitive headers (x-vercel-id, x-nf-request-id, x-railway-request-id) get 100% confidence. Server header fallback gets 80% confidence.

5. **vibe_code as boolean tag** - Simple boolean on Finding for UI badge display. False by default for all existing scanners, true only for vibe-code-specific findings.

6. **VARCHAR(50) for detection columns** - Framework/platform names stored as lowercase snake_case strings (nextjs, vite_react, sveltekit, nuxt, vercel, netlify, railway).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed missing vibe_code field in worker_pool.rs test constructors**
- **Found during:** Task 2 (Running detector tests)
- **Issue:** Compilation failed with E0063 error - missing vibe_code field in 2 test Finding constructors in worker_pool.rs
- **Fix:** Added vibe_code: false to test Finding constructors in test_compute_score function
- **Files modified:** src/orchestrator/worker_pool.rs
- **Verification:** cargo test detector passed all 5 tests
- **Committed in:** aebce53 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Auto-fix necessary to unblock compilation. The test file was not in the plan's scope but required update for consistency with Finding struct extension.

## Issues Encountered

None - plan executed smoothly. Parallel plans (03-02 and 03-03) added remediation.rs and vibecode.rs modules simultaneously, but no merge conflicts occurred as plan instructions specified to only add detector module line.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Detection engine complete and tested
- Framework detection: Next.js, Vite/React, SvelteKit, Nuxt with multi-signal scoring
- Platform detection: Vercel, Netlify, Railway from response headers
- Detection model types available for use in vibe-code scanner (plan 03-02)
- Finding vibe_code tag ready for UI badge display
- Scan detection columns ready for persistence
- Database migration ready to apply
- All existing scanners compatible (vibe_code: false set on all findings)
- Ready for framework-specific vulnerability scanning (plan 03-02)
- Ready for framework-aware remediation (plan 03-03)

---
*Phase: 03-vibe-code-intelligence*
*Completed: 2026-02-06*

## Self-Check: PASSED

All created files verified to exist.
All task commits verified in git history.
