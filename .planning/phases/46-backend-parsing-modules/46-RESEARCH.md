# Phase 46: Backend Parsing Modules - Research

**Researched:** 2026-04-06
**Domain:** Rust — lockfile parsing, OSV.dev API integration, supply chain orchestration
**Confidence:** HIGH (codebase verified directly; OSV schema verified via official source)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** New `SupplyChainError` enum — variants: `LockfileParse`, `OsvQuery`, `GitHubFetch`, `ChunkFailure`, `DepCountExceeded`, `Timeout`. Do NOT reuse existing `ScannerError`.
- **D-02:** `ParsedDep` struct with `name: String`, `version: String`, `source: DepSource` (Registry | Git | File | Link | Tarball), `is_dev: bool`. Only `Registry` deps sent to OSV.
- **D-03:** Parser handles lockfileVersion 1/2 (nested `dependencies` key) and v3 (flat `packages` key with `node_modules/` path prefix stripping). Version detected from `lockfileVersion` field.
- **D-04:** Deduplication by (name, version) tuple. Multiple paths to same package in v3 → one entry.
- **D-05:** Non-registry deps identified by `resolved` field pattern: `git+`, `file:`, `link:` prefixes, or missing `resolved` field.
- **D-06:** POST to `/v1/querybatch` with ecosystem "npm". Chunk at 1000 packages per request.
- **D-07:** Parallel chunk requests via `futures::join_all`. No bounded concurrency for MVP.
- **D-08:** Single retry per chunk on 5xx or timeout. Any chunk failing after retry → `SupplyChainError::ChunkFailure`.
- **D-09:** Per-chunk timeout: 10 seconds via reqwest client timeout.
- **D-10:** Tier rules: `MAL-` prefix → Infected; `severity[].score` CVSS >= 7.0 → Vulnerable; `database_specific.severity` CRITICAL or HIGH (when severity[] absent) → Vulnerable; other match → Advisory; no match → No Known Issues; non-registry dep → Unscanned.
- Three modules: `src/scanners/lockfile_parser.rs`, `src/scanners/osv_client.rs`, `src/scanners/supply_chain.rs`
- Results struct must be serde-serializable for JSONB storage (Phase 47 consumes it)
- Register new modules in `src/scanners/mod.rs`

### Claude's Discretion

- Module-internal struct names and helper function signatures
- OSV response deserialization approach (serde structs vs serde_json::Value)
- Test fixture file format and organization

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LOCK-01 | User can submit a package-lock.json (v1/v2/v3) and get all dependencies extracted | lockfileVersion field detection; `dependencies` vs `packages` key parsing |
| LOCK-02 | Parser handles lockfileVersion 1/2 (nested dependencies key) and v3 (flat packages key) | Exact JSON structure documented in this research |
| LOCK-03 | Parser deduplicates packages appearing at multiple paths in v3 format | HashMap<(name, version), ParsedDep> dedup approach |
| LOCK-04 | Non-npm deps (git:, file:, link:, tarball) counted as "unscanned" | `resolved` field pattern matching documented |
| OSV-01 | All extracted deps checked against OSV.dev /v1/querybatch | API schema fully documented here |
| OSV-02 | Deps chunked at 1000/batch, queried in parallel via futures::join_all | Already in use in js_secrets.rs; pattern documented |
| OSV-03 | OSV results categorized: MAL- → Infected, CVSS>=7/HIGH/CRITICAL → Vulnerable, other → Advisory | Severity field variants documented; categorization logic specified |
| OSV-04 | Any OSV chunk failing after 1 retry → entire scan fails with clear error | Retry + ChunkFailure variant documented |

</phase_requirements>

---

## Summary

Phase 46 delivers three Rust modules: a lockfile parser, an OSV.dev HTTP client, and an orchestrator. No HTTP endpoint, no database writes — pure computation returning a serializable result struct. The phase is self-contained and everything it needs (`reqwest`, `serde_json`, `futures`) is already in `Cargo.toml`.

The hardest correctness surface is the lockfile format variation. v3 is structurally different from v1/v2 and the `node_modules/` path prefix stripping is a common implementation gap. The OSV API is straightforward but its severity field has two paths (CVSS vector string vs string label) that both must be handled to avoid silent miscategorization of HIGH/CRITICAL advisories as low-tier Advisory findings.

The existing codebase pattern in `js_secrets.rs` using `futures::future::join_all` is the exact pattern needed for parallel OSV chunk requests. The project already has all required dependencies — no Cargo.toml changes needed.

**Primary recommendation:** Implement `lockfile_parser.rs` first (pure functions, easily unit-tested with fixtures), then `osv_client.rs` (one reqwest client, one batch method, retry logic), then `supply_chain.rs` as the thin orchestrator. Keep OSV deserialization as typed serde structs rather than `serde_json::Value` — the severity paths are predictable enough to warrant typed structs, and it makes test assertions cleaner.

---

## Project Constraints (from CLAUDE.md)

| Directive | Applies To |
|-----------|------------|
| No force-push to main | Git workflow |
| Run full CI locally before pushing | Build + test |
| NEVER push to main without explicit user approval | Deploy gating |
| Always read DESIGN.md before visual/UI decisions | Not applicable this phase (backend only) |
| Testing: Vitest (unit) + Playwright (E2E) + MSW for API mocking | Frontend — not applicable to Rust modules |
| Rust testing: standard `#[cfg(test)]` inline modules | Inferred from existing scanner pattern |

---

## Standard Stack

### Core (all already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1 (features: derive) | Struct serialization/deserialization | Already in use throughout codebase |
| `serde_json` | 1 | JSON parsing for lockfile and OSV responses | Already in use throughout codebase |
| `reqwest` | 0.13.1 (features: json) | HTTP client for OSV.dev requests | Already used by all other scanners |
| `futures` | 0.3 | `futures::future::join_all` for parallel chunk requests | Already in Cargo.toml; used in js_secrets.rs |
| `tokio` | 1 (features: full) | Async runtime | Project-wide runtime |

[VERIFIED: /home/john/vault/projects/github.com/shipsecure/Cargo.toml — all packages confirmed present at listed versions]

**No new Cargo.toml entries required for this phase.**

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `futures::future::join_all` | `tokio::task::JoinSet` | JoinSet gives bounded concurrency; join_all is simpler and sufficient for 1-2 chunks per decision D-07 |
| Typed serde structs for OSV response | `serde_json::Value` | Value is more flexible but harder to test; structs make severity field access explicit |

---

## Architecture Patterns

### Recommended Project Structure

```
src/scanners/
├── lockfile_parser.rs   # Pure parsing: String → Vec<ParsedDep>
├── osv_client.rs        # HTTP: Vec<ParsedDep> → Vec<OsvResult>
├── supply_chain.rs      # Orchestrator: String → SupplyChainScanResult
└── mod.rs               # Add: pub mod lockfile_parser; pub mod osv_client; pub mod supply_chain;
```

### Pattern 1: lockfile_parser.rs — Version-Dispatched Parsing

**What:** Detect `lockfileVersion`, dispatch to the correct parsing path, return `Vec<ParsedDep>`.

**When to use:** Every entry into the parsing pipeline.

**v3 `packages` key structure (verified from repo's own package-lock.json):**

```json
{
  "lockfileVersion": 3,
  "packages": {
    "": { "name": "frontend", "version": "0.1.0", "dependencies": {...}, "devDependencies": {...} },
    "node_modules/@adobe/css-tools": {
      "version": "4.4.4",
      "resolved": "https://registry.npmjs.org/@adobe/css-tools/-/css-tools-4.4.4.tgz",
      "integrity": "sha512-...",
      "dev": true,
      "license": "MIT"
    }
  }
}
```

Key observations from the project's own lockfile [VERIFIED: frontend/package-lock.json]:
- Root entry key is `""` (empty string) — must be skipped during dep extraction
- Package keys are `"node_modules/<name>"` — strip `"node_modules/"` prefix to get package name
- Scoped packages: `"node_modules/@scope/pkg"` → name is `"@scope/pkg"`
- `"dev": true` marks devDependencies — maps to `is_dev: true` in ParsedDep
- `"resolved"` is present and points to the npm registry tarball URL for all registry deps observed

**v1/v2 `dependencies` key structure (verified from npm spec and search results):**

```json
{
  "lockfileVersion": 1,
  "dependencies": {
    "express": {
      "version": "4.18.2",
      "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
      "integrity": "sha512-...",
      "dev": false,
      "dependencies": {
        "accepts": {
          "version": "1.3.8",
          ...
        }
      }
    }
  }
}
```

[CITED: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/]

- v1 has **nested** `dependencies` (package deps inside package deps) — must recurse
- v2 has **both** `dependencies` (for backwards compat) AND `packages` — parser should prefer `packages` when `lockfileVersion` is 2 to get a flat, authoritative view [ASSUMED — npm v7 behaviour: use packages key for v2 since it's the canonical representation in that version]
- `dev: false` is the default when absent — treat absent `dev` field as production dep

**Non-registry dep detection via `resolved` field (D-05):**

| Pattern | DepSource | Example |
|---------|-----------|---------|
| `resolved` starts with `git+` | Git | `"git+https://github.com/org/pkg.git#abc123"` |
| `resolved` starts with `file:` | File | `"file:../local-pkg"` |
| `resolved` starts with `link:` | Link | `"link:../symlinked-pkg"` |
| `resolved` field absent | Tarball or unknown | Package installed from local tarball |
| `resolved` is a `.tgz` URL not from registry | Tarball | `"https://example.com/pkg-1.0.0.tgz"` |

[CITED: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/ and search results for git+ format]

**Skeleton implementation approach:**

```rust
// Source: pattern matching verified against frontend/package-lock.json
pub fn parse(content: &str) -> Result<Vec<ParsedDep>, SupplyChainError> {
    let raw: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| SupplyChainError::LockfileParse(e.to_string()))?;

    let version = raw["lockfileVersion"].as_u64().unwrap_or(1);

    match version {
        3 => parse_v3(&raw),
        1 | 2 => parse_v1_v2(&raw),
        _ => Err(SupplyChainError::LockfileParse(format!("Unsupported lockfileVersion: {}", version))),
    }
}

fn parse_v3(raw: &serde_json::Value) -> Result<Vec<ParsedDep>, SupplyChainError> {
    let packages = raw["packages"].as_object()
        .ok_or_else(|| SupplyChainError::LockfileParse("Missing 'packages' key".into()))?;

    let mut seen: std::collections::HashSet<(String, String)> = HashSet::new();
    let mut deps = Vec::new();

    for (key, pkg) in packages {
        if key.is_empty() { continue; } // Skip root entry
        let name = key.strip_prefix("node_modules/").unwrap_or(key).to_string();
        let version = pkg["version"].as_str().unwrap_or("").to_string();
        if version.is_empty() { continue; }
        if !seen.insert((name.clone(), version.clone())) { continue; } // Dedup D-04

        let source = classify_source(pkg);
        let is_dev = pkg["dev"].as_bool().unwrap_or(false);
        deps.push(ParsedDep { name, version, source, is_dev });
    }
    Ok(deps)
}
```

### Pattern 2: osv_client.rs — Batch Query with Retry

**What:** Accept `Vec<ParsedDep>` (registry only), chunk at 1000, fire parallel requests, collect results.

**OSV.dev /v1/querybatch request schema** [VERIFIED: https://github.com/google/osv.dev/blob/master/docs/api/post-v1-querybatch.md]:

```json
POST https://api.osv.dev/v1/querybatch
Content-Type: application/json

{
  "queries": [
    {
      "package": {
        "name": "express",
        "ecosystem": "npm"
      },
      "version": "4.18.2"
    }
  ]
}
```

**OSV.dev /v1/querybatch response schema** [VERIFIED: official docs]:

```json
{
  "results": [
    {
      "vulns": [
        {
          "id": "GHSA-rv95-896h-c2vc",
          "modified": "2024-01-15T00:00:00Z"
        }
      ]
    },
    {
      "vulns": []
    }
  ]
}
```

Key behaviors:
- Response `results` array is **positionally aligned** with input `queries` array — result[i] corresponds to query[i]
- `vulns` array may be absent (not null, just missing) when there are no vulnerabilities — treat absent as empty
- Max 1000 queries per request [VERIFIED: web search cross-referenced with official docs]
- `next_page_token` appears in a result entry when that entry has >1000 vulns or total exceeds 3000 — for MVP, ignore pagination (malware packages rarely have 1000+ advisories)
- 5xx responses signal server error → trigger single retry per D-08

**Important:** querybatch returns only `id` and `modified` per vuln — NOT the full severity data. To get severity you need a separate `/v1/vulns/{id}` call OR use the batch to get IDs then hydrate. However, the CONTEXT.md D-10 severity rules reference `severity[].score` and `database_specific.severity`, which means the implementation must fetch full vuln detail.

**Resolution of this ambiguity:** The querybatch response intentionally omits full vuln data to keep response sizes manageable. The standard approach used by osv-scanner and similar tools is to use querybatch to get IDs, then call `/v1/vulns/{id}` (or the batch GET equivalent) to hydrate full records for categorization. [ASSUMED — based on osv-scanner Go source analysis in search results; needs implementation decision]

**Alternative approach:** Use `/v1/query` (singular) per package which returns full records, but this removes batching efficiency. **Recommended:** Use querybatch for IDs → group unique IDs → fetch full records via individual `/v1/vulns/{id}` calls in parallel. For MAL- prefix detection, the ID alone suffices without hydration.

### Pattern 3: supply_chain.rs — Orchestrator

**What:** Accept raw lockfile string, call parser, filter registry deps, call OSV client, categorize, return `SupplyChainScanResult`.

**Result struct (must be Serialize for JSONB storage in Phase 47):**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SupplyChainScanResult {
    pub total_deps: usize,
    pub infected: Vec<DepFinding>,
    pub vulnerable: Vec<DepFinding>,
    pub advisory: Vec<DepFinding>,
    pub no_known_issues: Vec<String>,   // just names/versions — no vuln detail
    pub unscanned: Vec<ParsedDep>,      // non-registry deps
    pub scanned_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepFinding {
    pub name: String,
    pub version: String,
    pub osv_id: String,
    pub description: String,
    pub tier: DepTier,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DepTier {
    Infected,
    Vulnerable,
    Advisory,
    NoKnownIssues,
    Unscanned,
}
```

### Anti-Patterns to Avoid

- **Parsing `dependencies` only in v2:** v2 lockfiles have BOTH `dependencies` and `packages`. The `packages` key is the authoritative source in v2 — using only `dependencies` misses packages installed in flat node_modules layout. Prefer `packages` when present.
- **Not skipping the root `""` key in v3:** The root entry (`""`) describes the project itself, not a dependency. Parsing it yields a dep with the project's own name and version, polluting OSV results.
- **Assuming `vulns` is always present in querybatch response:** When there are no vulnerabilities for a package, the result entry may have no `vulns` key at all (absent, not null). Use `unwrap_or_default()` when deserializing.
- **Using `serde` strict deserialization for the full OSV schema:** OSV schema has many optional fields and DB-specific extensions. Use `#[serde(default)]` liberally, or use `Option<>` for all fields beyond `id` and `modified`.
- **Treating `database_specific.severity` as a fixed enum:** Different source databases (GHSA, NVD, etc.) use different string labels. Normalize to uppercase before comparing to "HIGH" and "CRITICAL".

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client with timeout | Custom TCP code | `reqwest` (already present) | Handles TLS, connection pooling, timeout semantics |
| JSON parsing with optional fields | Manual string scanning | `serde_json` with `#[serde(default)]` | Handles absent fields, type coercion |
| Parallel async execution | Manual task spawning | `futures::future::join_all` | Already used in js_secrets.rs; correct semantics for collecting Vec<Result<>> |
| CVSS score extraction | CVSS string parser | Parse the numeric portion from the vector string or use a cvss crate | CVSS vector strings like `CVSS:3.1/AV:N/AC:H/...` don't directly encode a numeric base score — base score must be computed or found in a separate `score` field |

**Key insight:** The OSV schema has two severity representations that must both be handled. Neither replaces the other — a given advisory may have one, both, or neither.

---

## OSV Severity Field — Critical Detail

This is the highest-risk area for incorrect categorization.

### Two severity paths in the OSV schema

**Path 1: Top-level or affected-level `severity` array** [VERIFIED: ossf.github.io/osv-schema — confirmed via search]:

```json
"severity": [
  {
    "type": "CVSS_V3",
    "score": "CVSS:3.1/AV:N/AC:H/PR:N/UI:N/S:C/C:H/I:N/A:N"
  }
]
```

The `score` field is the **CVSS vector string**, NOT a numeric float. To extract the base score (0-10), you must:
- Either compute it from the vector (use the `cvss` crate: `docs.rs/cvss/latest/cvss/`) [CITED: https://docs.rs/cvss/latest/cvss/]
- Or check for a companion `baseScore` field that some databases add

**For MVP simplicity (within Claude's discretion):** Parse the CVSS base score from the vector string using the `cvss` crate, or alternatively fetch full vuln records which often include a numeric score in `database_specific`.

**Path 2: `database_specific.severity` string label** [VERIFIED: search results confirmed this field exists]:

```json
"database_specific": {
  "severity": "HIGH",
  "github_reviewed": true,
  "nvd_published_at": "2025-07-23T21:15:26Z"
}
```

Values observed: `"CRITICAL"`, `"HIGH"`, `"MODERATE"`, `"LOW"`. Normalize to uppercase.

### Categorization logic (D-10):

```
if osv_id starts with "MAL-" → Infected (check ID from querybatch response — no hydration needed)
else if severity[].score present:
    parse CVSS base score from vector string
    if base score >= 7.0 → Vulnerable
    else → Advisory
else if database_specific.severity in ["CRITICAL", "HIGH"]:
    → Vulnerable
else:
    → Advisory (MODERATE, LOW, or no severity data)
```

### MAL- prefix — confirmed real [VERIFIED: fetched osv.dev/vulnerability/MAL-2025-47423]:

MAL- prefixed IDs are real entries in OSV for malicious packages (e.g., `MAL-2025-47423`: "Malicious code in ng-imports-checker (npm)"). These entries typically have **no CVSS severity** — they rely solely on the ID prefix for categorization. The MAL- check must happen before the severity check.

---

## Common Pitfalls

### Pitfall 1: v3 Nested `node_modules` paths for scoped packages

**What goes wrong:** A scoped package like `@babel/core` appears as `"node_modules/@babel/core"`. Stripping `"node_modules/"` prefix gives `"@babel/core"` correctly. But a nested dep may appear as `"node_modules/parent/node_modules/@babel/core"` in some lockfile versions — stripping only the first prefix gives `"parent/node_modules/@babel/core"` which is wrong.

**Why it happens:** npm's legacy hoisting algorithm sometimes places packages in nested `node_modules` directories, and v2/v3 lockfiles record the full path.

**How to avoid:** After stripping the leading `"node_modules/"` prefix, check if the remaining string still contains `"node_modules/"`. If so, take everything after the last `"node_modules/"` occurrence to get the true package name. Dedup by (name, version) handles the case where the same package appears at multiple paths.

**Warning signs:** Package names in the extracted dep list containing slashes that aren't `@scope/` format.

### Pitfall 2: Missing `version` field in some lockfile entries

**What goes wrong:** Some entries in the `packages` object (particularly workspace packages or bundled deps) may lack a `version` field, causing `unwrap()` panics or empty version strings in ParsedDep.

**Why it happens:** Workspace packages (`"link": true`) have no `version` in the lockfile. Bundled deps that aren't installed have no registry metadata.

**How to avoid:** Always use `.as_str().unwrap_or("")` when reading version; skip entries with empty version string before building ParsedDep.

**Warning signs:** ParsedDep entries with `version: ""` reaching the OSV client.

### Pitfall 3: querybatch positional alignment breaks when chunks contain errors

**What goes wrong:** If chunk A has 1000 queries and receives a partial or error response, the positional mapping between query index and result index breaks for subsequent processing.

**Why it happens:** The querybatch API guarantees alignment — but only within a successful response. On retry, a new request is sent so alignment resets correctly.

**How to avoid:** Treat each chunk independently. Each chunk carries its own slice of deps; on success, zip results with the chunk's dep slice. On failure after retry, return `ChunkFailure` for the entire scan (D-08).

### Pitfall 4: CVSS vector string is not a numeric score

**What goes wrong:** Reading `severity[0].score` and comparing it as a float >= 7.0 against a string like `"CVSS:3.1/AV:N/..."` causes a type error or always-false comparison.

**Why it happens:** The OSV schema's `severity[].score` field contains the full CVSS vector string, not the numeric base score. This is a common misreading of the schema.

**How to avoid:** Either use the `cvss` crate to parse the vector and extract the base score, or check for a `baseScore` numeric field in `database_specific` (if present from that DB). Fall back to `database_specific.severity` string label if no numeric score can be extracted.

### Pitfall 5: reqwest client timeout applies per-request, not per-chunk-retry

**What goes wrong:** Setting a 10s timeout on the reqwest `Client` means each individual request times out at 10s. The retry is a second request, also with a 10s timeout. Total time for one chunk on worst case: 20s + latency. With 2 chunks in parallel: 20s worst case total.

**Why it happens:** reqwest `.timeout()` on the client builder applies to the entire response read, not the connection. This is the correct behavior.

**How to avoid:** This is correct per D-09 — document it clearly. The 10s timeout is per attempt; retry doesn't extend it.

---

## Code Examples

### join_all pattern (from codebase)

```rust
// Source: src/scanners/js_secrets.rs (VERIFIED)
let results = futures::future::join_all(scan_tasks).await;

for mut file_findings in results.into_iter().flatten() {
    findings.append(&mut file_findings);
}
```

For OSV chunks, collect `Vec<Result<OsvChunkResult, SupplyChainError>>` and check for any `Err`:

```rust
let chunk_results: Vec<Result<OsvChunkResult, SupplyChainError>> =
    futures::future::join_all(chunk_futures).await;

let mut all_results = Vec::new();
for result in chunk_results {
    match result {
        Ok(chunk) => all_results.extend(chunk.results),
        Err(e) => return Err(SupplyChainError::ChunkFailure(e.to_string())),
    }
}
```

### SupplyChainError enum structure

```rust
// Source: CONTEXT.md D-01 (locked decision)
#[derive(Debug)]
pub enum SupplyChainError {
    LockfileParse(String),
    OsvQuery(String),
    GitHubFetch(String),
    ChunkFailure(String),
    DepCountExceeded(usize),
    Timeout,
}

impl fmt::Display for SupplyChainError { /* ... */ }
impl std::error::Error for SupplyChainError {}
```

### serde structs for OSV response (recommended approach)

```rust
// Source: derived from VERIFIED OSV schema; use #[serde(default)] for optional fields
#[derive(Debug, Deserialize, Default)]
struct OsvQueryBatchResponse {
    #[serde(default)]
    results: Vec<OsvQueryResult>,
}

#[derive(Debug, Deserialize, Default)]
struct OsvQueryResult {
    #[serde(default)]
    vulns: Vec<OsvVulnSummary>,
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsvVulnSummary {
    pub id: String,
    pub modified: Option<String>,
}

// Full vuln record (for hydration calls):
#[derive(Debug, Deserialize)]
struct OsvVulnDetail {
    pub id: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub severity: Vec<OsvSeverity>,
    pub database_specific: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OsvSeverity {
    #[serde(rename = "type")]
    pub score_type: String,  // "CVSS_V3", "CVSS_V2", etc.
    pub score: String,        // Full vector string e.g. "CVSS:3.1/AV:N/..."
}
```

### reqwest client with timeout

```rust
// Source: pattern from src/scanners/security_headers.rs (VERIFIED)
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .user_agent("ShipSecure-Scanner/1.0")
    .build()
    .map_err(|e| SupplyChainError::OsvQuery(format!("Failed to build client: {}", e)))?;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Individual `/v1/query` per package | `/v1/querybatch` up to 1000/request | OSV API v1 (2022+) | 1000x fewer HTTP round-trips for large lockfiles |
| Parsing only `dependencies` key | Prefer `packages` key in v2/v3 | npm v7 (2021) | Flat representation more accurate than nested v1 format |
| Manual CVSS string parsing | `cvss` crate or numeric score from DB-specific | Ongoing | Avoids bug-prone manual parsing of CVSS vector strings |

**Deprecated/outdated:**
- lockfileVersion 1 (`npm <=6`): Still encountered in old projects; parser must support it but it is no longer generated by modern npm.
- `npm-shrinkwrap.json`: Same format as package-lock.json; if ever needed, the same parser handles it.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | lockfileVersion 2 should use the `packages` key (not `dependencies`) as the canonical source | Architecture Patterns > Pattern 1 | Parser falls back to `dependencies` key for v2 — would still work but may miss flat-layout packages |
| A2 | querybatch returns only `id`/`modified` per vuln — severity data requires separate `/v1/vulns/{id}` hydration call | Architecture Patterns > Pattern 2 | If full records are returned in some API version, the hydration step is unnecessary overhead |
| A3 | `database_specific.severity` string values are always uppercase ("HIGH", "CRITICAL") | OSV Severity Field section | If mixed case, comparison fails and HIGH/CRITICAL advisories fall through to Advisory tier |

---

## Open Questions

1. **CVSS base score extraction method**
   - What we know: `severity[].score` is a CVSS vector string, not a numeric float. The `cvss` crate can parse it.
   - What's unclear: Does adding the `cvss` crate to Cargo.toml require planner's attention, or is it Claude's discretion?
   - Recommendation: Add `cvss = "0.3"` to Cargo.toml as part of this phase's Wave 0. Alternative: parse the numeric base score from `database_specific` when present (GHSA records often include it), and fall back to string label when not.

2. **querybatch + hydration: two-step vs one-step**
   - What we know: querybatch gives IDs only. Full severity data requires `/v1/vulns/{id}` per advisory.
   - What's unclear: For typical npm lockfiles with few vulnerable deps, hydration adds minimal overhead. But a lockfile with many advisories could require many sequential hydration calls.
   - Recommendation: Use querybatch for IDs, then fire all unique advisory ID hydration calls in parallel via join_all. MAL- prefix detection happens at querybatch stage (no hydration needed). This is a Phase 46 implementation detail within Claude's discretion.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust / cargo | Build | ✓ | cargo 1.93.0, rustc 1.93.0 | — |
| reqwest 0.13.1 | OSV HTTP client | ✓ | 0.13.1 (in Cargo.toml) | — |
| serde_json 1 | Lockfile + OSV parsing | ✓ | 1 (in Cargo.toml) | — |
| futures 0.3 | join_all for parallel chunks | ✓ | 0.3 (in Cargo.toml) | — |
| tokio 1 | Async runtime | ✓ | 1 (in Cargo.toml) | — |
| cvss crate | CVSS vector → base score | ✗ | — | Parse numeric score from database_specific, or use string label only |
| api.osv.dev | OSV vulnerability data | External | — | Tests must mock this endpoint |
| Node.js | — | ✓ | v22.22.0 | — |

[VERIFIED: Cargo.toml read directly; cargo/rustc versions from shell]

**Missing dependencies with no fallback:** None blocking.

**Missing dependencies with fallback:**
- `cvss` crate: Not currently in Cargo.toml. Either add it (small dependency) or implement score extraction using the `database_specific.severity` string label as the CVSS fallback. Either approach satisfies D-10.

---

## Validation Architecture

> nyquist_validation key absent from config.json — treating as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none (inline `#[cfg(test)]` modules per existing codebase pattern) |
| Quick run command | `cargo test scanners::lockfile_parser` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LOCK-01 | v3 lockfile string → correct dep list | unit | `cargo test lockfile_parser::tests::test_parse_v3` | ❌ Wave 0 |
| LOCK-02 | v1/v2 lockfile → correct dep list | unit | `cargo test lockfile_parser::tests::test_parse_v1` / `test_parse_v2` | ❌ Wave 0 |
| LOCK-03 | v3 duplicate paths → single dep | unit | `cargo test lockfile_parser::tests::test_dedup_v3` | ❌ Wave 0 |
| LOCK-04 | git/file/link dep → DepSource::Git/File/Link | unit | `cargo test lockfile_parser::tests::test_non_registry_deps` | ❌ Wave 0 |
| OSV-01 | registry deps → OSV querybatch call (mocked) | unit | `cargo test osv_client::tests::test_batch_query` | ❌ Wave 0 |
| OSV-02 | 1500 deps → 2 chunks fired | unit | `cargo test osv_client::tests::test_chunking` | ❌ Wave 0 |
| OSV-03 | MAL- id → Infected; CVSS>=7 → Vulnerable; other → Advisory | unit | `cargo test supply_chain::tests::test_categorization` | ❌ Wave 0 |
| OSV-04 | chunk 5xx after retry → ChunkFailure | unit | `cargo test osv_client::tests::test_chunk_retry_failure` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test scanners` (unit tests for new scanner modules only)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** `cargo test && cargo clippy -- -D warnings` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src/scanners/lockfile_parser.rs` — create file with `#[cfg(test)]` module, fixture JSON strings for v1/v2/v3 lockfiles
- [ ] `src/scanners/osv_client.rs` — create file with `#[cfg(test)]` module; mock reqwest using `wiremock` or inline response structs
- [ ] `src/scanners/supply_chain.rs` — create file with `#[cfg(test)]` module
- [ ] Add `pub mod lockfile_parser; pub mod osv_client; pub mod supply_chain;` to `src/scanners/mod.rs`
- [ ] Test fixtures: inline JSON strings for v1, v2, v3 lockfiles (including git/file/link deps, duplicate paths)

**Note on mocking reqwest:** The project has no existing HTTP mocking crate for Rust tests. Options: (1) add `wiremock = "0.6"` to `[dev-dependencies]` for HTTP-level mocking, (2) inject the reqwest `Client` via parameter so tests can build a client pointing at a test server, or (3) abstract the HTTP call behind a trait. Approach (2) is the most idiomatic for this codebase and requires no new dependencies.

---

## Security Domain

> security_enforcement not explicitly false — section required.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Phase 46 has no auth layer |
| V3 Session Management | no | Stateless parsing module |
| V4 Access Control | no | No authz decisions |
| V5 Input Validation | yes | Validate lockfile is valid JSON; enforce max dep count cap (API-06, enforced in Phase 47 handler but parser should not panic on malformed input) |
| V6 Cryptography | no | No crypto operations |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed lockfile causing panic/OOM | Denial of Service | Return `Err(SupplyChainError::LockfileParse(...))` on any parse failure; never `unwrap()` on untrusted input |
| Malformed OSV response causing panic | Denial of Service | Use `#[serde(default)]` + `Option<>` for all OSV response fields; return `Err` on deserialization failure |
| SSRF via OSV API URL | Spoofing | OSV URL is hardcoded (`api.osv.dev`) — no user-controlled URL component; not a concern |
| Lockfile with crafted package names causing regex/string issues | Tampering | No regex on package names; serde string fields are safe |

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: Cargo.toml] — All dependency versions confirmed present in project
- [VERIFIED: src/scanners/js_secrets.rs] — `futures::future::join_all` usage pattern
- [VERIFIED: src/scanners/security_headers.rs] — reqwest client construction pattern, `ScannerError` enum structure
- [VERIFIED: src/scanners/mod.rs] — Module registration pattern
- [VERIFIED: frontend/package-lock.json] — Real v3 lockfile format; root `""` key; `node_modules/` prefix; `dev: true` field; `resolved` URL format
- [VERIFIED: osv.dev/vulnerability/MAL-2025-47423] — MAL- prefix confirmed real; no CVSS data on malware entries

### Secondary (MEDIUM confidence)
- [CITED: github.com/google/osv.dev/blob/master/docs/api/post-v1-querybatch.md] — querybatch request/response schema, positional alignment guarantee, 1000-query limit, pagination behavior
- [CITED: ossf.github.io/osv-schema] — severity array structure, database_specific field semantics
- [CITED: docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/] — lockfileVersion 1/2/3 structural differences

### Tertiary (LOW confidence)
- Web search results for v1/v2/v3 format differences — confirmed by multiple sources but official npm docs not directly fetched (timeout)

---

## Metadata

**Confidence breakdown:**
- Lockfile parsing (v3): HIGH — verified against real lockfile in repo
- Lockfile parsing (v1/v2): MEDIUM — documented in npm spec, couldn't directly load official page; confirmed by multiple credible sources
- OSV API schema: HIGH — official docs fetched and verified
- OSV severity fields: HIGH — schema confirmed via ossf.github.io source and real MAL- entry inspection
- futures pattern: HIGH — verified in codebase
- Cargo dependencies: HIGH — verified in Cargo.toml

**Research date:** 2026-04-06
**Valid until:** 2026-05-06 (OSV API is stable; npm lockfile format unlikely to change)
