# Phase 47: API Handler & Database - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 47-api-handler-database
**Areas discussed:** Error-to-HTTP mapping, Query audit scope

---

## Error-to-HTTP Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| Looks good | Use the proposed 6-variant mapping as-is | ✓ |
| Adjust | Change some mappings | |
| You decide | Claude picks based on existing patterns | |

**User's choice:** Looks good (proposed mapping accepted)
**Notes:** LockfileParse→400, OsvQuery→502, GitHubFetch→502, ChunkFailure→502, DepCountExceeded→400, Timeout→504. All use RFC 7807 Problem Details JSON format matching existing ApiError.

---

## Query Audit Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Critical paths only (~5 queries) | Dashboard list, cleanup, quota, per-target cache. Minimal diff. | ✓ |
| Filter everything (~12 queries) | Every SELECT/UPDATE gets kind awareness. Maximum safety. | |
| Database view | web_app_scans view. Existing code unchanged. | |

**User's choice:** Critical paths only (~5 queries)
**Notes:** Dashboard scan list, cleanup task, per-user quota, per-target cache, stats endpoint. UUID-based lookups are safe without filtering.

---

## Claude's Discretion

- GitHub URL regex pattern specifics
- SupplyChainError integration approach (extend ApiError or keep separate)
- Migration file naming
- Multipart field names and validation

## Deferred Ideas

None — discussion stayed within phase scope.
