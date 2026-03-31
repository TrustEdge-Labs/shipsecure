# Phase 44: Content Routes - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Create /blog route with MDX rendering and /check/{platform} landing pages for Lovable, Bolt, and v0. These serve as inbound marketing surfaces for CVE-driven traffic. No backend changes.

</domain>

<decisions>
## Implementation Decisions

### Blog Infrastructure
- **D-01:** MDX files live in `content/blog/` at the frontend project root (separate from code). Not inside `app/blog/`. Cleaner git diffs when adding posts.
- **D-02:** Blog frontmatter fields: `title`, `date` (YYYY-MM-DD), `slug`, `description` (for meta tags), `published` (boolean, allows drafts).
- **D-03:** Blog index (`/blog`) lists published posts sorted by date descending. When no published posts exist, show a "coming soon" page with: "Security research and vibe-code vulnerability analysis. Coming soon." and a scan CTA linking to `/`.
- **D-04:** Individual post route: `/blog/[slug]`. Use @next/mdx for compilation. Render with Geist typography from DESIGN.md. Include article `<time>` element with "Mar 29, 2026" format.
- **D-05:** No RSS feed, no CMS, no comments in v1. Minimal MDX.

### /check/{platform} Landing Pages
- **D-06:** Data-driven config approach. One shared `CheckPlatformPage` component reads from a platform config object: `{ name, slug, accent, placeholder, heroTitle, cveContext, cveLink }`.
- **D-07:** Platform configs:
  - **Lovable:** accent=#e11d48 (rose-600), placeholder="https://your-app.lovable.app", CVE: "CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys."
  - **Bolt:** accent=#3b82f6 (blue-500), placeholder="https://your-project.bolt.new", CVE: "45% of AI-generated code contains security vulnerabilities. Bolt apps ship fast but skip security defaults."
  - **v0:** accent=#fafafa (near-white on dark), placeholder="https://your-app.vercel.app", CVE: "Vercel/v0 apps inherit Next.js defaults but often miss security headers, exposed .env files, and CSP configuration."
- **D-08:** Each /check page pre-fills the scan form URL placeholder with the platform-specific URL. The scan form component needs to accept an optional `defaultUrl` prop.
- **D-09:** Dynamic route: `/check/[platform]/page.tsx`. Unknown platforms return 404 via Next.js `notFound()`.
- **D-10:** Platform-specific SEO: each page gets its own `generateMetadata` with platform name in title and description. Example: "Is your Lovable app secure? | ShipSecure", description: "CVE-2025-48757 exposed 170+ Lovable apps. Scan yours free in 30 seconds."

### Claude's Discretion
- MDX config approach (@next/mdx vs next-mdx-remote vs contentlayer)
- Whether to inline platform configs or put them in a separate `lib/platforms.ts` file
- Blog post typography styling (prose classes vs custom CSS)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend config
- `frontend/next.config.ts` -- Next.js config, CSP headers, standalone mode. Needs MDX plugin config.
- `frontend/app/page.tsx` -- Landing page with ScanForm component (reference for hero pattern)
- `frontend/components/scan-form.tsx` -- Scan form, needs optional defaultUrl prop for /check pages

### Design system
- `DESIGN.md` -- Geist typography, color palette, component styles, spacing scale
- `docs/designs/customer-acquisition-v1.md` -- CEO plan with platform-specific visual treatment decision

### Prior phase context
- `.planning/phases/42-funnel-unlock/42-CONTEXT.md` -- Scan form now accepts any URL (no lockdown)
- `.planning/phases/43-share-results-ux/43-CONTEXT.md` -- OG meta pattern for generateMetadata

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ScanForm` component: accepts `isAuthenticated` prop. Needs new `defaultUrl` prop for /check pages.
- `PageContainer` component: shared layout with max-width 1100px. Use for blog and /check pages.
- `generateMetadata` pattern in results page: server-side metadata generation. Reuse for /check pages.
- Existing OKLch/semantic token system in CSS. Platform accents can override `--accent-*` variables.

### Established Patterns
- App Router file conventions: `page.tsx` for routes, `layout.tsx` for shared layout
- Server components by default, "use client" only when needed
- Lucide React for icons

### Integration Points
- `frontend/app/` directory for new routes (/blog, /check)
- `frontend/next.config.ts` for MDX plugin registration
- Navigation: header links may need "Blog" and/or "Check" additions (or defer)

</code_context>

<specifics>
## Specific Ideas

- The "coming soon" blog page should feel intentional, not placeholder. "Security research and vibe-code vulnerability analysis." with a direct link to scan.
- Platform accent colors should be visible in the hero area (H1 color or accent bar) but not overwhelm the page. The scan form and results stay in ShipSecure's green accent.
- Blog post typography should feel like a good technical blog (readable, spacious, code blocks styled with Geist Mono).

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 44-content-routes*
*Context gathered: 2026-03-31*
