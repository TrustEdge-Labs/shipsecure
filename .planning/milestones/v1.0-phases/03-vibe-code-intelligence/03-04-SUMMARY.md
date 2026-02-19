---
phase: 03-vibe-code-intelligence
plan: 04
subsystem: orchestrator
tags: [rust, axum, orchestrator, scanner-integration, detection, vibecode, remediation]

# Dependency graph
requires:
  - phase: 03-01
    provides: Framework/platform detection engine
  - phase: 03-02
    provides: Vibe-code scanner with Nuclei templates
  - phase: 03-03
    provides: Framework-specific remediation generation

provides:
  - 6-stage scan pipeline with detection feeding downstream scanners
  - Detection results stored in database
  - Vibe-code findings enhanced with framework-specific remediation
  - Graceful degradation when detection fails

affects: [03-05, 04-*]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Detection runs sequentially first, results feed parallel scanner stage
    - Detection failure is graceful (warns, continues with other scanners)
    - Per-scanner clones pattern maintained for parallel execution

key-files:
  created: []
  modified:
    - src/orchestrator/worker_pool.rs
    - src/db/scans.rs

key-decisions:
  - "Detection runs as first stage (sequential) before parallel scanners"
  - "Detection failure does NOT fail the scan"
  - "Framework/platform strings passed to downstream scanners"
  - "Remediation applied to vibe-code findings in orchestrator"

patterns-established:
  - "6-stage scan pipeline: Detection (stage 1) then Headers+TLS+Files+Secrets+VibCode (stages 2-6)"
  - "Detection results stored in database columns (detected_framework, detected_platform)"
  - "VibCode timeout increased to 180s (Nuclei can be slow)"

# Metrics
duration: 2min
completed: 2026-02-06
---

# Phase 03 Plan 04: Orchestrator Wiring Summary

**6-stage scan pipeline integrates detection, vibe-code scanning, and framework-specific remediation into orchestrator with graceful degradation**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-06T04:20:45Z
- **Completed:** 2026-02-06T04:22:18Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Detection engine runs as first stage, results feed downstream scanners
- Vibe-code scanner added as 5th parallel scanner with framework/platform context
- Remediation engine enhances vibe-code findings with framework-specific fixes
- Detection failure is graceful - warns but continues with other scanners
- All existing scanner behavior preserved

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire detection as first stage in scan pipeline** - `d5c9195` (feat)

## Files Created/Modified
- `src/orchestrator/worker_pool.rs` - Added detection stage, vibecode scanner, and remediation integration
- `src/db/scans.rs` - Added update_detected_framework and update_detected_platform helpers

## Decisions Made

**Detection runs as first stage (sequential) before parallel scanners**
- Framework/platform detection needs to complete before vibecode scanner runs
- Detection results feed template selection in vibecode scanner
- Sequential execution ensures detection completes before parallel stage

**Detection failure does NOT fail the scan**
- Detection is best-effort, not critical for scan completion
- If detection fails, scan continues with all scanners (including vibecode with framework=None)
- Logged as warning, stage marked complete, scan proceeds normally

**Framework/platform strings passed to downstream scanners**
- Extracted from detection result as Option<String>
- Passed to run_scanners, cloned for vibecode scanner task
- VibCode receives Option<&str> for framework/platform

**Remediation applied to vibe-code findings in orchestrator**
- Remediation engine called inline in vibecode scanner task
- Applies framework-specific remediation before findings returned
- Template ID extracted from raw_evidence, passed to remediation generator

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Orchestrator integration complete. All three Wave 1 modules (detection, vibecode, remediation) now wired into scan pipeline.

Next: Plan 03-05 (API/frontend integration) can expose detection results and vibe-code findings via API and dashboard.

Ready for:
- API endpoints to return detected framework/platform
- Frontend to display detection results
- Vibe-code badge on findings
- Framework-specific remediation in results view

## Self-Check: PASSED

All modified files exist:
- src/orchestrator/worker_pool.rs
- src/db/scans.rs

All commits verified:
- d5c9195

---
*Phase: 03-vibe-code-intelligence*
*Completed: 2026-02-06*
