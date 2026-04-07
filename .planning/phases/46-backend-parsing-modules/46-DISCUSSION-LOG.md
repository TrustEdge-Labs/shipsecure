# Phase 46: Backend Parsing Modules - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 46-backend-parsing-modules
**Areas discussed:** Error type design, Dependency representation

---

## Error Type Design

| Option | Description | Selected |
|--------|-------------|----------|
| New SupplyChainError enum | Specific variants for each failure mode. API handler can return precise HTTP errors. | ✓ |
| Reuse ScannerError | Use Other(String) for supply chain specifics. Fewer types but less precise error handling. | |
| You decide | Claude picks the best approach based on the codebase patterns | |

**User's choice:** New SupplyChainError enum
**Notes:** Explicit over clever. Variants: LockfileParse, OsvQuery, GitHubFetch, ChunkFailure, DepCountExceeded, Timeout. Enables Phase 47 to pattern-match for HTTP status codes.

---

## Dependency Representation

| Option | Description | Selected |
|--------|-------------|----------|
| Rich struct (source type + dev flag) | Future-proof for Phase 2. More data in results. Slightly more parsing work. | ✓ |
| Minimal struct (name + version + scannable) | Narrowest wedge. Just what OSV needs + unscanned tracking. | |
| You decide | Claude picks based on implementation complexity | |

**User's choice:** Rich struct with source type + dev flag
**Notes:** ParsedDep with name, version, source (DepSource enum: Registry/Git/File/Link/Tarball), is_dev bool. Only Registry deps sent to OSV. Future-proof for Phase 2 risk scoring without adding significant implementation cost.

---

## Claude's Discretion

- Module-internal struct names and helper function signatures
- OSV response deserialization approach
- Test fixture file format and organization

## Deferred Ideas

None — discussion stayed within phase scope.
