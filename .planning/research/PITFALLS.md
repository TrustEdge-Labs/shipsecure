# Pitfalls Research: Brand Identity Integration

**Domain:** Adding brand identity to existing Next.js 16 + Tailwind CSS v4 SaaS
**Researched:** 2026-02-09
**Confidence:** HIGH (codebase-specific) / MEDIUM (general patterns)

## Critical Pitfalls

### Pitfall 1: Layout Shift Cascade from Header Addition

**What goes wrong:** Adding a sticky header to an app that was designed without one causes layout shift across ALL routes. Current layout uses `flex-col` + `min-h-screen` + `flex-1` pattern. Adding a 64px header pushes content down, breaks spacing, and causes CLS issues.

**Why it happens:** App was built header-less. All pages assume content starts at top of viewport. Hero sections, sticky elements, and scroll calculations don't account for header height.

**Current state:** layout.tsx uses `min-h-screen flex flex-col` with `flex-1` on main content. No header exists.

**Prevention:**
1. Snapshot all routes at 375px, 768px, 1024px BEFORE adding header
2. Define `--header-height: 64px` CSS variable, reference in spacing
3. Audit all `pt-*`, `mt-*`, `min-h-screen` usage before implementation
4. Add header to layout.tsx and fix spacing in same commit

**Phase assignment:** Foundation — layout structure must be fixed BEFORE header component

---

### Pitfall 2: Dark Mode Color Regression from Token Migration

**What goes wrong:** Migrating from raw Tailwind classes (`blue-600`, `gray-50`) to semantic tokens breaks dark mode. 17 components use `dark:` prefix classes. New tokens without equivalent dark mode mappings cause white-on-white text, invisible inputs, broken severity badges.

**Current color inventory:**
- **Severity:** red-100/700/900, orange-100/700/900, yellow-100/700/900, blue-100/700/900
- **State:** green-50/200/600/800/950, red-50/200/600/800/950
- **Neutrals:** gray-50 through gray-950, white, black
- **Brand:** blue-400/500/600/700/800

**Prevention:**
1. Inventory all `dark:` patterns before refactor: `rg "dark:(bg|text|border)-" frontend/`
2. Design semantic tokens with explicit dark mode overrides
3. Parallel implementation: add tokens WITHOUT removing existing classes
4. Migrate one component at a time with visual verification
5. Dark mode testing checklist per component

**Phase assignment:** Color System — requires careful staging, no partial states

---

### Pitfall 3: Logo SVG Breaks Dark Mode or SSR

**What goes wrong:** Inline SVG with hardcoded fill colors renders incorrectly in dark mode. Or Next.js SSR hydration mismatch causes flash of wrong-color logo.

**Prevention:**
1. Use `currentColor` for all fills — inherits from parent text color
2. Use `className="text-blue-600 dark:text-blue-400"` on logo wrapper
3. Specify width/height to prevent CLS
4. Don't read `window`/`document` during render (breaks SSR)

**Phase assignment:** Foundation — logo must work before header integration

---

### Pitfall 4: Favicon Cache Invalidation

**What goes wrong:** Branded favicon doesn't appear for users due to aggressive browser caching (up to 7 days). Users report "still seeing the old icon."

**Current state:** Default Next.js SVG files in `/public/` — no branded favicon exists.

**Prevention:**
1. Generate all formats: favicon.ico, favicon.svg, icon-192.png, icon-512.png, apple-touch-icon.png
2. Use Next.js Metadata API for proper `<link>` tags
3. SVG favicon with `prefers-color-scheme` media query for dark mode
4. Add `?v=2` cache bust param on first deploy
5. Verify all formats after deploy: `curl -I https://shipsecure.ai/favicon.ico`

**Phase assignment:** Favicon — can be deployed independently

---

### Pitfall 5: SVG Icon Inconsistency

**What goes wrong:** Icons render at different sizes across components. Colors don't follow dark mode. Missing accessibility labels. Inconsistent stroke weights.

**Current state:** Unicode emoji (page.tsx: lock, key, document, magnifying glass). Not scalable.

**Prevention:**
1. Use established library (Lucide/Heroicons) — not custom icons
2. Standardize sizing via Tailwind classes (`w-5 h-5`, `w-6 h-6`)
3. Always use `currentColor` for color inheritance
4. Decorative icons: `aria-hidden="true"`. Standalone: `aria-label="..."`.
5. Document icon usage pattern for consistency

**Phase assignment:** Icon System — depends on color system being complete

---

## Moderate Pitfalls

### Pitfall 6: Incomplete Color Migration

**What goes wrong:** Partially migrated codebase — some components use tokens, others use raw classes. Future color changes require touching two systems.

**Prevention:**
1. Track migration per component (spreadsheet or checklist)
2. Block phase completion until ALL components migrated
3. Add lint rule to flag raw color classes in new code

### Pitfall 7: Header Z-Index Conflicts

**What goes wrong:** Header z-index conflicts with future modals/dropdowns/tooltips.

**Prevention:** Define z-index scale in design tokens before header implementation.

### Pitfall 8: Mobile Header Navigation Confusion

**What goes wrong:** Desktop header works but mobile navigation pattern unclear.

**Current context:** ShipSecure is essentially single-page SaaS with minimal nav needs (Home, Privacy, Terms). May not need hamburger menu at all.

**Prevention:**
1. Mobile: Logo + CTA only (no menu needed for current nav)
2. All touch targets >= 44px
3. Test at 375px, 768px, 1024px

---

## Minor Pitfalls

### Pitfall 9: Logo File Proliferation
Keep single source SVG. Generate raster formats programmatically.

### Pitfall 10: OG Image Update Overlooked
When updating brand, update opengraph-image.tsx in same PR.

### Pitfall 11: No Design System Documentation
Document patterns DURING implementation, not after.

---

## Phase-Specific Warnings

| Phase | Likely Pitfall | Mitigation |
|-------|---------------|------------|
| Logo Component | SVG dark mode breaks | Use `currentColor`, test both modes |
| Color System | Incomplete migration | Track every component, block until 100% |
| Header | Layout shift cascade | Refactor layout FIRST, then add header |
| Favicon | Browser cache | Multiple formats + cache bust |
| Icon System | Inconsistent sizing | Standardized component interface |
| OG Image | Overlooked during brand update | Checklist item in final phase |

## Regression Prevention

**Pre-deploy checklist:**
- [ ] Test Chrome, Firefox, Safari
- [ ] Test light mode and dark mode
- [ ] Test at 375px, 768px, 1280px
- [ ] Verify all routes: `/`, `/results/[token]`, `/scan/[id]`, `/privacy`, `/terms`
- [ ] Severity badges render correctly
- [ ] Form inputs visible and functional
- [ ] Lighthouse score >= 90
- [ ] No hydration warnings in console
- [ ] No 404s for asset files

**Recovery:** If visual regression reaches production: `git revert` + force push to trigger redeploy.

---

## Sources

- ShipSecure codebase inspection (17 component files, layout.tsx, globals.css)
- Existing dark mode via `prefers-color-scheme` + `dark:` classes
- Current layout: flex-col + min-h-screen + flex-1 pattern
- Next.js 16 App Router metadata conventions
- Tailwind CSS v4 @theme directive
- Production auto-deploy CI/CD context
