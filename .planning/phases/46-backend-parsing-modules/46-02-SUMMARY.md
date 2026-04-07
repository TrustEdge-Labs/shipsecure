---
phase: 46-backend-parsing-modules
plan: 02
subsystem: api
tags: [osv, supply-chain, vulnerability-scanning, reqwest, wiremock]

requires:
  - phase: 46-backend-parsing-modules/01
    provides: lockfile parser types (ParsedDep, DepSource, SupplyChainError, SupplyChainScanResult)
provides:
  - OsvClient HTTP client with batch query, retry, and hydration
  - scan_lockfile() orchestrator wiring parser -> OSV -> categorization -> result
  - categorize_finding() for tier assignment (Infected/Vulnerable/Advisory)
affects: [47-database-storage, 48-api-endpoints]

tech-stack:
  added: [wiremock (dev)]
  patterns: [chunked-parallel-http, retry-on-5xx, positional-response-alignment, tier-categorization]

key-files:
  created:
    - src/scanners/osv_client.rs
    - src/scanners/supply_chain.rs
  modified:
    - Cargo.toml

key-decisions:
  - "Used Option B for CVSS: skip vector parsing (no cvss crate), rely on database_specific.severity which is reliably present for GHSA/npm entries"
  - "Added wiremock dev-dependency for proper HTTP-level testing of retry and chunking logic"

patterns-established:
  - "Chunked parallel HTTP: batch large dep lists into 1000-item chunks, fire via join_all"
  - "Retry on 5xx: single retry with ChunkFailure abort on double failure"
  - "Tier categorization: MAL- prefix -> Infected, HIGH/CRITICAL -> Vulnerable, other -> Advisory"

requirements-completed: [OSV-01, OSV-02, OSV-03, OSV-04]

duration: 5min
completed: 2026-04-07
---

# Phase 46 Plan 02: OSV Client and Supply Chain Orchestrator Summary

**OSV.dev HTTP client with chunked batch query, retry logic, and tier-based categorization orchestrator producing serializable SupplyChainScanResult**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-07T01:07:40Z
- **Completed:** 2026-04-07T01:12:17Z
- **Tasks:** 2/2
- **Files modified:** 3

## Accomplishments

- Implemented OsvClient with batch querying (1000 dep chunks, parallel via join_all), single retry on 5xx, 10s timeout, and parallel vulnerability hydration
- Implemented scan_lockfile() orchestrator: parse -> partition -> query -> hydrate -> categorize -> SupplyChainScanResult
- Full tier categorization: MAL- prefix -> Infected (no hydration), database_specific.severity HIGH/CRITICAL -> Vulnerable, other OSV match -> Advisory, no match -> NoKnownIssues, non-registry -> Unscanned
- 22 unit tests across both modules covering all categorization tiers, retry logic, chunking, positional alignment, and end-to-end flow

## Task Commits

Each task was committed atomically:

1. **Task 1: OSV.dev client with batch query, retry, and hydration** - `2b11cc7` (feat)
2. **Task 2: Supply chain orchestrator with categorization logic** - `19d1dc0` (feat)

## Files Created/Modified

- `src/scanners/osv_client.rs` - OSV.dev HTTP client: OsvClient struct, query_batch (chunking + parallel), hydrate_vulns, retry on 5xx, 10 tests
- `src/scanners/supply_chain.rs` - Orchestrator: scan_lockfile(), categorize_finding(), tier assignment logic, 12 tests
- `Cargo.toml` - Added wiremock dev-dependency for HTTP mock testing

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added wiremock dev-dependency**
- **Found during:** Task 1
- **Issue:** No HTTP mocking library available to test retry logic and chunk failure paths
- **Fix:** Added `wiremock = "0.6"` as dev-dependency
- **Files modified:** Cargo.toml

**2. [Rule 2 - Missing Critical] CVSS vector parsing skipped (Option B)**
- **Found during:** Task 2
- **Issue:** No `cvss` crate in Cargo.toml; plan explicitly allows Option B
- **Fix:** Rely on database_specific.severity field (reliably present for GHSA/npm entries) instead of CVSS vector parsing
- **Files modified:** src/scanners/supply_chain.rs
- **Tradeoff:** Vulns with only CVSS vectors and no database_specific.severity will be categorized as Advisory instead of Vulnerable. Acceptable per plan guidance.

## Threat Mitigations Applied

| Threat ID | Mitigation |
|-----------|------------|
| T-46-04 | OSV base URL hardcoded to api.osv.dev; HTTPS enforced by reqwest default |
| T-46-05 | 10-second timeout per request; single retry cap; ChunkFailure aborts scan |
| T-46-06 | All OSV response fields use #[serde(default)]; missing severity treated as Advisory |
| T-46-08 | Non-registry deps skipped from OSV queries; hydration parallel with timeout |

## Self-Check: PASSED

All files exist and all commits verified.
