---
phase: 02-free-tier-mvp
plan: 03
subsystem: scanners
tags: [rust, docker, nuclei, testssl, javascript, secrets, regex, security]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: Finding model, ScannerError pattern, database schema
provides:
  - JavaScript secrets scanner with pattern-based detection
  - Containerized scanner execution wrapper with Docker security hardening
  - Nuclei and testssl.sh integration with Finding model mapping
affects: [02-04-orchestrator-integration, 02-05-frontend]

# Tech tracking
tech-stack:
  added: [regex, futures, lazy_static, Docker container execution]
  patterns: [Pattern-based secret detection, Docker security hardening with CIS flags, Graceful degradation for Docker unavailability]

key-files:
  created:
    - src/scanners/js_secrets.rs
    - src/scanners/container.rs
  modified:
    - Cargo.toml
    - src/scanners/mod.rs

key-decisions:
  - "Use lazy_static for compiled regex patterns to avoid repeated compilation overhead"
  - "Limit JS file scanning to 20 files max and 2MB per file to prevent abuse and memory issues"
  - "Apply false positive filtering for test keys, placeholders, and example values"
  - "All Docker containers run with full CIS Docker Security hardening flags"
  - "Graceful degradation when Docker unavailable (logs warning, returns empty findings)"
  - "Use Uuid::nil() as placeholder scan_id in Finding structs, will be set by orchestrator"

patterns-established:
  - "Secret pattern detection: HIGH/MEDIUM/LOW confidence levels with severity mapping"
  - "Docker security hardening: 8 mandatory flags for all container execution (--rm, --read-only, --cap-drop all, --user 1000:1000, --memory, --pids-limit, --cpu-shares, --no-new-privileges)"
  - "Scanner graceful degradation: Return empty Vec<Finding> with warning log on Docker unavailable, not error"

# Metrics
duration: 4min
completed: 2026-02-05
---

# Phase 02 Plan 03: JavaScript Secrets & Container Scanners Summary

**JavaScript secrets scanner detects 10+ secret types with confidence-based pattern matching, container wrapper executes Nuclei and testssl.sh with full CIS Docker security hardening**

## Performance

- **Duration:** 4 minutes
- **Started:** 2026-02-05T14:03:16Z
- **Completed:** 2026-02-05T14:07:44Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- JavaScript secrets scanner discovers and fetches JS bundles (max 20 files, 2MB each), detects AWS keys, Stripe keys, GitHub tokens, Firebase/Supabase keys with HIGH/MEDIUM/LOW confidence patterns
- False positive filtering skips test keys, placeholders, and example values (sk_test_, YOUR_KEY_HERE, etc.)
- Container execution wrapper runs Nuclei and testssl.sh in hardened Docker containers with all 8 CIS Docker Security flags
- Parses Nuclei JSONL and testssl JSON output into Finding structs with severity mapping
- Graceful degradation when Docker unavailable (development-friendly)

## Task Commits

Each task was committed atomically:

1. **Task 1: JavaScript secrets scanner** - `902791c` (feat)
2. **Task 2: Containerized scanner execution wrapper** - `de70a43` (feat)

## Files Created/Modified
- `src/scanners/js_secrets.rs` - Discovers JS bundles via script tags and common paths, fetches concurrently, scans for 10+ secret patterns (AWS, Stripe, GitHub, Slack, Twilio, Firebase, Supabase, generic API keys), false positive filtering, redacted evidence
- `src/scanners/container.rs` - Docker container execution with security hardening (--rm, --read-only, --cap-drop all, --user 1000:1000, --memory limits, --pids-limit, --cpu-shares, --no-new-privileges), run_nuclei and run_testssl functions, JSONL/JSON parsing, severity mapping, graceful Docker unavailability handling
- `Cargo.toml` - Added regex, futures, lazy_static dependencies
- `src/scanners/mod.rs` - Exported js_secrets and container modules

## Decisions Made

**Pattern-based secret detection confidence levels:**
- HIGH confidence: Format-validated patterns (AWS AKIA*, Stripe sk_live_*, GitHub ghp_*, etc.)
- MEDIUM confidence: Pattern + context (Firebase apiKey near "firebase", Supabase JWT-format keys)
- LOW confidence: Entropy-based detection (not implemented in this plan, reserved for future enhancement)

**Resource limits for container security:**
- Nuclei: 512MB memory, 120s timeout (scans can be thorough with many templates)
- testssl.sh: 100MB memory, 180s timeout (TLS handshake and cipher testing takes time)
- Both: 1000 process limit, 512 CPU shares, non-root user 1000:1000

**Docker availability handling:**
- Check `docker info` exit code to detect Docker installation
- If unavailable: Log warning, return empty Vec<Finding>, continue execution
- Rationale: Allows local development without Docker, production deployment requires Docker

**False positive filtering strategy:**
- Skip test/example patterns: "test", "example", "placeholder", "YOUR_KEY_HERE", "CHANGE_ME", "xxx", "000", "123"
- Skip Stripe test keys: sk_test_*, pk_test_*
- Skip repeated single characters (e.g., "aaaaaaa")
- Context-aware: Check surrounding text for "example" or "demo"

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## Next Phase Readiness

**Ready for integration:**
- Scanner modules compile successfully with only minor unused field warnings
- Both scanners return Vec<Finding> with proper Severity enum and optional raw_evidence
- Container scanner gracefully handles Docker unavailability
- All findings use Uuid::nil() placeholder for scan_id, ready for orchestrator to set

**Blockers:**
None

**Concerns:**
- Docker container images (projectdiscovery/nuclei:latest, drwetter/testssl.sh:latest) need to be pulled on first run - will be slow initially but cached afterward
- Need to verify Docker is installed in Render production environment (or document Docker requirement)

---
*Phase: 02-free-tier-mvp*
*Completed: 2026-02-05*
