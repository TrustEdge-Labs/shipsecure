# Phase 46: Backend Parsing Modules - Context

**Gathered:** 2026-04-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Rust modules that parse package-lock.json (v1/v2/v3) and query OSV.dev for vulnerability/malware data, producing categorized findings. Three modules: lockfile_parser.rs, osv_client.rs, supply_chain.rs (orchestrator). This phase delivers the backend engine only — no HTTP endpoint, no database, no frontend.

</domain>

<decisions>
## Implementation Decisions

### Error Handling
- **D-01:** New `SupplyChainError` enum with specific variants: `LockfileParse`, `OsvQuery`, `GitHubFetch`, `ChunkFailure`, `DepCountExceeded`, `Timeout`. Do NOT reuse existing `ScannerError`. The API handler in Phase 47 pattern-matches on these to return precise HTTP status codes and user-facing error messages.

### Dependency Representation
- **D-02:** Rich struct with source type tracking:
  ```rust
  struct ParsedDep {
    name: String,
    version: String,
    source: DepSource, // Registry | Git | File | Link | Tarball
    is_dev: bool,
  }
  ```
  Only `Registry` deps are sent to OSV.dev. All others counted as "Unscanned" in results.

### Lockfile Parsing
- **D-03:** Parser handles lockfileVersion 1/2 (nested `dependencies` key) and v3 (flat `packages` key with `node_modules/` path prefix stripping). Version detected from `lockfileVersion` field.
- **D-04:** Deduplication by (name, version) tuple. Multiple paths to same package in v3 format → one entry.
- **D-05:** Non-registry deps identified by `resolved` field pattern: `git+`, `file:`, `link:` prefixes, or missing `resolved` field.

### OSV.dev Integration
- **D-06:** POST to `/v1/querybatch` with ecosystem "npm". Chunk at 1000 packages per request.
- **D-07:** Parallel chunk requests via `futures::join_all`. No bounded concurrency needed for MVP (typical: 1-2 chunks).
- **D-08:** Single retry per chunk on 5xx or timeout. If any chunk fails after retry, entire scan fails with `SupplyChainError::ChunkFailure`.
- **D-09:** Per-chunk timeout: 10 seconds via reqwest client timeout.

### Result Categorization
- **D-10:** Tier assignment rules:
  - **Infected:** OSV ID has `MAL-` prefix
  - **Vulnerable:** `severity[].score` >= 7.0 where present. If absent, `database_specific.severity` string is CRITICAL or HIGH.
  - **Advisory:** Any other OSV match (MEDIUM, LOW, or no severity data and no MAL- prefix)
  - **No Known Issues:** No OSV match for this dep (registry dep with clean record)
  - **Unscanned:** Non-registry dep (git, file, link, tarball)

### Claude's Discretion
- Module-internal struct names and helper function signatures
- OSV response deserialization approach (serde structs vs serde_json::Value)
- Test fixture file format and organization

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design & Architecture
- `~/.gstack/projects/TrustEdge-Labs-shipsecure/john-main-design-20260406-133756.md` — Approved supply chain MVP design doc with full implementation scope, data flow, and tier definitions
- `~/.gstack/projects/TrustEdge-Labs-shipsecure/john-main-eng-review-test-plan-20260406-141500.md` — Test plan with affected routes, edge cases, critical paths

### Existing Scanner Pattern
- `src/scanners/security_headers.rs` — Reference for scanner module structure, ScannerError pattern, reqwest usage
- `src/scanners/mod.rs` — Scanner module registration

### API Reference
- OSV.dev API: POST https://api.osv.dev/v1/querybatch — batch vulnerability query endpoint

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest` (v0.13.1 with json feature) already in Cargo.toml — use for OSV.dev and GitHub API calls
- `ScannerError` pattern in `src/scanners/security_headers.rs` — reference for error handling structure (but creating new SupplyChainError per D-01)
- `serde` + `serde_json` already dependencies — use for lockfile and OSV response parsing

### Established Patterns
- Scanners live in `src/scanners/` as individual modules
- Scanners return `Vec<Finding>` but supply chain results use JSONB (different pattern for this phase)
- `uuid::Uuid` for IDs, `chrono` for timestamps — consistent with existing models

### Integration Points
- New modules register in `src/scanners/mod.rs`
- `supply_chain.rs` orchestrator will be called by the API handler in Phase 47
- Results struct must be serde-serializable for JSONB storage in Phase 47

</code_context>

<specifics>
## Specific Ideas

No specific requirements — implementation follows the approved design doc and eng review decisions. The design doc's wireframe at `/tmp/gstack-sketch-1775496985.html` shows the expected results display (consumed by Phase 48, not this phase).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 46-backend-parsing-modules*
*Context gathered: 2026-04-06*
