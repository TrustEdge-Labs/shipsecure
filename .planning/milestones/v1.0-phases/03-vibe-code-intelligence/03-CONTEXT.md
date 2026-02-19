# Phase 3: Vibe-Code Intelligence - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-detect frontend frameworks and deployment platforms from scan responses, run vibe-code-specific vulnerability checks (BaaS misconfigs, env leaks, missing auth, unprotected routes, default credentials), and deliver copy-paste remediation code tailored to the detected stack. This phase adds detection intelligence and framework-specific fixes to the existing free tier scan — no new scan tiers or payment flows.

</domain>

<decisions>
## Implementation Decisions

### Detection scope
- Detect 4 frontend frameworks: Next.js, Vite/React, SvelteKit, Nuxt
- Detect 3 deployment platforms: Vercel, Netlify, Railway
- High confidence only — require multiple signals before showing a framework badge (no guessing)
- Detection runs as a visible, separate scan stage ("Detecting framework...") before other scanners
- Framework detection results feed into downstream scanners to enable framework-specific checks

### Vulnerability coverage
- Full vibe-code checklist: BaaS misconfigs (Supabase RLS, Firebase rules), env variable leaks (NEXT_PUBLIC_ secrets, .env in build output), missing auth middleware, unprotected API routes, default admin credentials
- Custom Nuclei templates for vibe-code-specific checks + curated community templates for general web vulns
- Probing depth: passive + light active only (analyze responses, probe known BaaS endpoints). Reserve aggressive active probing for Phase 4 paid tier
- Findings use existing severity levels (Critical/High/Medium/Low) for A-F grading, plus a "vibe-code" tag to identify AI-generated-code issues

### Remediation format
- Targeted diffs: "In your next.config.js, add these lines" — not full file replacements
- Moderate explanation: code block + 1-2 sentence explanation of why the fix works and what it prevents
- Version-aware when it matters: default to framework-family-level fixes, split only when syntax genuinely differs (e.g., Next.js App Router vs Pages Router)
- No "verify your fix" sections — users rescan to verify

### Results presentation
- Framework + platform badge inline with grade circle: "B — Next.js on Vercel"
- Vibe-code findings get a small "Vibe-Code" tag/badge — subtle differentiation, not a separate section
- No vibe-code filter toggle — the tag is sufficient, keep UI simple
- When no framework detected: show "Framework: Not detected" next to grade (honest, signals feature exists)

### Claude's Discretion
- Exact HTML/JS fingerprinting patterns for each framework
- Nuclei template structure and naming conventions
- Number of detection signals required for "high confidence" threshold
- How to handle partial matches (e.g., React detected but not the meta-framework)

</decisions>

<specifics>
## Specific Ideas

- Detection stage should feel like the product "knows what you built" — a moment of intelligence before showing findings
- The "B — Next.js on Vercel" inline badge is the signature visual for this phase
- Vibe-code tag on findings connects back to the product positioning: "we catch what AI tools miss"
- CVE-2025-48757 (Lovable RLS misconfigs) is the canonical example of what this phase catches
- Free tier stays passive/light active — aggressive probing is the Phase 4 paid tier differentiator

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-vibe-code-intelligence*
*Context gathered: 2026-02-05*
