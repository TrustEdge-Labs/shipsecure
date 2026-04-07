---
phase: 46-backend-parsing-modules
verified: 2026-04-07T01:15:44Z
status: passed
score: 4/4 must-haves verified
re_verification: false
---

# Phase 46: Backend Parsing Modules Verification Report

**Phase Goal:** Rust can parse any package-lock.json (v1/v2/v3) and query OSV.dev for all extracted dependencies, producing categorized findings
**Verified:** 2026-04-07T01:15:44Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | A package-lock.json with lockfileVersion 1, 2, or 3 yields the correct deduplicated dependency list | VERIFIED | `parse()` dispatches on `lockfileVersion`; 12 lockfile_parser tests cover v1/v2/v3, dedup, root-skip, nested paths — all 23 tests pass |
| 2 | Git/file/link/tarball dependencies appear in findings as "Unscanned" rather than crashing or being silently dropped | VERIFIED | `supply_chain.rs` partitions deps by source; non-registry deps land in `unscanned` field of `SupplyChainScanResult`; `scan_lockfile_result_structure` test asserts `unscanned.len() == 1` for a git dep |
| 3 | All extracted npm packages are checked against OSV.dev in parallel batches; a package with a MAL- advisory is returned as "Infected", CVSS>=7 as "Vulnerable", and any other match as "Advisory" | VERIFIED* | `query_batch` chunks at 1000 and fires via `join_all`; `categorize_finding` checks MAL- prefix (Infected), database_specific.severity HIGH/CRITICAL (Vulnerable), else Advisory. Note: CVSS vector score parsing skipped per plan Option B — Vulnerable classification relies on `database_specific.severity` field; functionally correct for npm/GHSA entries |
| 4 | If any OSV batch fails after one retry, the entire scan returns a clear error rather than silently returning partial results | VERIFIED | `query_chunk` retries once on 5xx; `query_batch_5xx_retry_then_fail` test asserts `ChunkFailure` returned on double failure; wiremock confirms exactly 2 HTTP calls made |

**Score:** 4/4 truths verified

*SC3 note: The roadmap criterion mentions "CVSS>=7 as Vulnerable". The implementation skips CVSS vector parsing (no `cvss` crate) and instead uses `database_specific.severity` HIGH/CRITICAL. This is an explicitly accepted tradeoff documented in the plan (Option B) and the summary. The behavior is correct for GHSA/npm entries where `database_specific.severity` is reliably present. No later phase overrides this decision.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/scanners/lockfile_parser.rs` | Shared types + parse function | VERIFIED | 570 lines; exports `SupplyChainError`, `ParsedDep`, `DepSource`, `DepTier`, `DepFinding`, `SupplyChainScanResult`, `parse()`; 23 unit tests pass |
| `src/scanners/osv_client.rs` | OSV HTTP client with batch query, retry, hydration | VERIFIED | 611 lines; exports `OsvClient`, `OsvVulnDetail`, `OsvSeverity`, `DepVulnMatch`; 10 unit tests using wiremock pass |
| `src/scanners/supply_chain.rs` | Orchestrator wiring parser -> OSV -> result | VERIFIED | 378 lines; exports `scan_lockfile`, `scan_lockfile_with_client`, `categorize_finding`; 12 unit tests pass |
| `src/scanners/mod.rs` | Module registration for all three supply chain modules | VERIFIED | Contains `pub mod lockfile_parser`, `pub mod osv_client`, `pub mod supply_chain` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/scanners/lockfile_parser.rs` | `src/scanners/mod.rs` | `pub mod lockfile_parser` | WIRED | Line 5 of mod.rs |
| `src/scanners/osv_client.rs` | `https://api.osv.dev/v1/querybatch` | reqwest POST | WIRED | `DEFAULT_BASE_URL = "https://api.osv.dev"`; URL built via `format!("{}/v1/querybatch", self.base_url)` at line 188 |
| `src/scanners/osv_client.rs` | `https://api.osv.dev/v1/vulns/{id}` | reqwest GET | WIRED | `format!("{}/v1/vulns/{}", self.base_url, id)` at line 290 |
| `src/scanners/supply_chain.rs` | `src/scanners/lockfile_parser.rs` | `lockfile_parser::parse` | WIRED | `crate::scanners::lockfile_parser::parse(content)?` at line 26 |
| `src/scanners/supply_chain.rs` | `src/scanners/osv_client.rs` | `OsvClient` | WIRED | `OsvClient::new()` at line 16; `osv_client.query_batch()` at line 35; `osv_client.hydrate_vulns()` at line 52 |

### Data-Flow Trace (Level 4)

These are pure Rust library functions, not UI components rendering dynamic data. No data-flow trace (Level 4) is applicable — the functions take input, process it, and return output. The unit tests with wiremock confirm real data flows through the full parse -> query -> categorize pipeline.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All lockfile_parser tests pass | `cargo test scanners::lockfile_parser` | 23 passed, 0 failed | PASS |
| All osv_client tests pass | `cargo test scanners::osv_client` | 10 passed, 0 failed | PASS |
| All supply_chain tests pass | `cargo test scanners::supply_chain` | 12 passed, 0 failed | PASS |
| Clippy clean | `cargo clippy -- -D warnings` | No warnings | PASS |

**Total tests verified: 45 across all three modules**

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| LOCK-01 | 46-01 | User can submit a package-lock.json (v1/v2/v3) and get all dependencies extracted | SATISFIED | `parse()` function handles all three versions |
| LOCK-02 | 46-01 | Parser handles lockfileVersion 1/2 (nested deps) and v3 (flat packages key) | SATISFIED | `parse_v1_v2_nested()` + `parse_v3()` dispatch; tests `parse_v1`, `parse_v2_with_packages`, `parse_v3` |
| LOCK-03 | 46-01 | Parser deduplicates packages appearing at multiple paths in v3 format | SATISFIED | `HashSet<(String, String)>` dedup; `parse_v3_dedup` test confirms single entry for duplicate |
| LOCK-04 | 46-01 | Non-npm deps (git:, file:, link:, tarball) are counted as "unscanned" | SATISFIED | Partition in `supply_chain.rs` lines 30-32; `parse_non_registry_sources` + `scan_lockfile_result_structure` tests |
| OSV-01 | 46-02 | All extracted deps are checked against OSV.dev /v1/querybatch API | SATISFIED | `query_batch()` in `osv_client.rs` calls `/v1/querybatch`; orchestrator calls it for all registry deps |
| OSV-02 | 46-02 | Deps chunked at 1000/batch, queried in parallel via futures::join_all | SATISFIED | `CHUNK_SIZE = 1000`; `futures::future::join_all(futures)` at line 155; `query_batch_1500_deps_two_chunks` test |
| OSV-03 | 46-02 | Results categorized: MAL- -> Infected, CVSS>=7/HIGH/CRITICAL -> Vulnerable, other -> Advisory | SATISFIED | `categorize_finding()` implements rules; CVSS vector parsing uses database_specific.severity (Option B); 9 categorization tests |
| OSV-04 | 46-02 | If any OSV chunk fails after 1 retry, entire scan fails with clear error | SATISFIED | `query_chunk()` retries once on 5xx; `ChunkFailure` propagated; `query_batch_5xx_retry_then_fail` test with wiremock `.expect(2)` |

All 8 requirements for Phase 46 are satisfied. No orphaned requirements found.

### Anti-Patterns Found

No blockers or warnings found.

Scan of all three implementation files:
- No `TODO`, `FIXME`, `XXX`, `HACK`, or placeholder comments in implementation paths
- No `return null`/`return {}`/`return []` stubs in logic (empty `vec![]` returns are conditional, not default stubs)
- No hardcoded empty data passed to rendering
- `#[allow(dead_code)]` on `OsvVulnSummary.modified` — intentional (field deserialized from API but unused internally); informational only, not a stub

### Human Verification Required

None. All functionality is verifiable via automated tests. The OSV.dev HTTP calls are tested via wiremock — no live API calls needed to verify behavior.

### Gaps Summary

No gaps. All four roadmap success criteria are met by the actual implementation. All 8 requirement IDs (LOCK-01 through LOCK-04, OSV-01 through OSV-04) are satisfied with concrete implementation and passing tests. The one noted deviation (CVSS vector parsing via Option B) is an explicitly planned tradeoff accepted in the design document and does not block the phase goal.

---

_Verified: 2026-04-07T01:15:44Z_
_Verifier: Claude (gsd-verifier)_
