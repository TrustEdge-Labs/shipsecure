# Project Research Summary

**Project:** ShipSecure — Brand Identity Integration
**Domain:** Visual brand identity for developer-focused security SaaS
**Researched:** 2026-02-09
**Confidence:** HIGH (stack/architecture), MEDIUM (features)

## Executive Summary

ShipSecure needs professional brand identity elements to transition from a working prototype to a credible security product. Research shows developer-focused SaaS products prioritize minimal, geometric logos that scale to favicon sizes, monochromatic color systems with single accent colors, and consistent SVG icon systems. The recommended approach leverages existing Next.js 16 and Tailwind CSS v4 capabilities with only one new dependency (lucide-react for icons).

The critical path involves: (1) establishing design tokens first to avoid color regression, (2) building logo component with proper dark mode support using `currentColor`, (3) refactoring layout structure before adding header to prevent layout shift cascade, then (4) systematic icon migration. This order prevents the three most severe pitfalls: dark mode breakage from token migration, layout shift from header insertion, and SSR hydration issues with logo rendering.

Key risk is incomplete color token migration causing a split codebase where some components use semantic tokens and others use raw Tailwind classes. Mitigation requires tracking migration progress per component and blocking phase completion until 100% migration is verified. The existing 17 components with dark mode classes must be migrated carefully to avoid white-on-white text, invisible inputs, or broken severity badges.

## Key Findings

### Recommended Stack

**Zero-dependency approach:** The existing stack (Next.js 16 + Tailwind CSS v4) already provides everything needed except icons. Native SVG rendering eliminates the need for `@svgr/webpack` or `react-svg-loader`. Tailwind v4's `@theme` directive replaces external design token libraries like Style Dictionary. Next.js Metadata API handles favicon generation without additional tools.

**Core technologies:**
- **Lucide React (^0.468.0)**: Icon component library — 1400+ tree-shakeable icons, 1-2KB per icon, TypeScript-native, consistent 24px design system
- **Tailwind CSS v4 @theme**: Design tokens — Native CSS custom properties, full IntelliSense, no additional dependencies
- **Native SVG components**: Logo rendering — Zero dependencies, full TypeScript support, React Server Components compatible
- **Next.js Metadata API**: Favicon generation — Built-in multi-format support using Sharp (already a Next.js dependency)

**What NOT to add:** `@svgr/webpack` (Next.js handles natively), `react-icons` (poor tree-shaking, 10-20KB per icon), `tailwindcss-themer` (replaced by v4 @theme), Font Awesome (legacy CSS approach, 50KB+ base), `styled-components`/`emotion` (conflicts with Tailwind).

### Expected Features

**Must have (table stakes):**
- Logo/Wordmark — Professional credibility, brand recognition
- Favicon (multi-size, dark mode) — Browser tab identification
- Consistent Icon System — Visual coherence using SVG instead of emoji
- Header/Navbar with Logo — Primary brand touchpoint
- Defined Color System — Design tokens for visual consistency and WCAG AA compliance
- Hover/Focus States — Accessibility and interactive feedback
- Responsive Logo Behavior — Icon-only at mobile, full wordmark at desktop

**Should have (competitive):**
- Branded OG Images — Add logo overlay to existing opengraph-image.tsx
- Geometric/Technical Logo — Communicates precision and security
- Monochromatic + Accent — Blue accent on neutral base, matches modern dev tools

**Defer (v2+):**
- Animated Logo — Adds complexity, not essential for credibility
- Custom Iconography — High effort, Lucide provides 1400+ icons
- SVG Scroll Animations — Polish feature, defer until core identity proven
- Gradient Accents — Can be added incrementally after token system established

**Anti-features to avoid:**
- Mascot/character logo (too playful for security)
- Multiple brand colors (cluttered, unfocused)
- Lowercase-only wordmark (reduces readability)
- Overly complex logo (won't scale to 16x16px favicon)
- Custom icon fonts (performance and accessibility issues)
- Serif typography (reduces technical feel)

### Architecture Approach

Incremental, non-breaking integration using Tailwind v4 `@theme` for design tokens, native SVG components with `currentColor` for theme support, and file-based favicon conventions. Color migration happens in parallel: add tokens WITHOUT removing existing classes first, migrate components one-by-one with dark mode verification, then remove legacy classes only after 100% migration confirmed.

**Major components:**

1. **Design Token System** — Tailwind v4 `@theme` in globals.css defining semantic tokens (brand-primary, surface-primary, text-secondary, border-subtle) with explicit dark mode overrides via `@media (prefers-color-scheme: dark)`

2. **Logo Component** — SVG component with variants (full/icon/wordmark), sizes (sm/md/lg/xl), uses `currentColor` for fills to inherit text color, includes aria-label and role="img", never uses `<img>` tag

3. **Header/Navbar** — Sticky positioned (z-index: 1020), fixed 64px height, logo left + nav center + CTA right, hides nav on mobile, defines `--header-height` CSS variable for spacing calculations

4. **Icon System** — Lucide React components with consistent sizing (w-5 h-5 for decorative, w-6 h-6 standalone), `currentColor` for theme support, proper aria attributes (aria-hidden for decorative, aria-label for standalone)

5. **Favicon Configuration** — Dynamic generation via app/icon.tsx using ImageResponse, SVG favicon with `prefers-color-scheme` media query for dark mode support, static apple-icon.png and favicon.ico fallbacks

**Key architectural decisions:**
- Layout refactor before header addition (prevents layout shift cascade)
- Parallel token migration (additive, not destructive)
- Single source SVG for logo (generate variants programmatically)
- Z-index scale defined upfront (base: 0, dropdown: 1000, sticky: 1020, fixed: 1030, modal: 1040-1050, tooltip: 1070)

### Critical Pitfalls

1. **Layout Shift Cascade from Header Addition** — App was built header-less with `min-h-screen flex flex-col` pattern. Adding 64px sticky header pushes content down, breaks spacing across all routes. **Prevention:** Define `--header-height: 64px` CSS variable, snapshot all routes before implementation, refactor layout structure FIRST before adding header component.

2. **Dark Mode Color Regression from Token Migration** — 17 components use `dark:` prefix classes for severity badges, state indicators, and form inputs. Migrating to semantic tokens without equivalent dark mode mappings causes white-on-white text, invisible inputs, broken severity badges. **Prevention:** Inventory all `dark:` patterns with `rg "dark:(bg|text|border)-" frontend/`, add tokens WITHOUT removing classes, migrate one component at a time, dark mode test each component.

3. **Logo SVG Breaks Dark Mode or SSR** — Inline SVG with hardcoded fill colors renders incorrectly in dark mode. Next.js SSR hydration mismatch causes flash of wrong-color logo. **Prevention:** Use `currentColor` for all fills (inherits parent text color), apply theme classes to wrapper (`text-blue-600 dark:text-blue-400`), specify width/height to prevent CLS, never read `window`/`document` during render.

4. **Favicon Cache Invalidation** — Browsers cache favicons aggressively (up to 7 days). Users report "still seeing old icon" after deployment. **Prevention:** Generate all formats (favicon.ico, favicon.svg, icon-192.png, icon-512.png, apple-touch-icon.png), use Next.js Metadata API for proper link tags, add `?v=2` cache bust param on first deploy, verify formats after deploy with curl.

5. **SVG Icon Inconsistency** — Icons render at different sizes, colors don't follow dark mode, missing accessibility labels, inconsistent stroke weights when mixing custom icons with Lucide. **Prevention:** Use established library (Lucide), standardize sizing via Tailwind classes, always use `currentColor`, decorative icons get `aria-hidden="true"`, standalone icons get `aria-label`.

**Moderate pitfalls:** Incomplete color migration causing split codebase, header z-index conflicts with future modals, mobile header navigation confusion (solve by logo + CTA only, no hamburger menu needed for current minimal nav).

## Implications for Roadmap

Based on research, suggested 6-phase structure:

### Phase 1: Design Token System
**Rationale:** Foundation layer that enables all other work. Non-breaking (additive only). Must be complete before any color migration to prevent regression. Defines semantic tokens with explicit dark mode overrides.

**Delivers:** Tailwind v4 `@theme` in globals.css with brand colors (primary, primary-hover), semantic tokens (surface-primary/secondary, text-primary/secondary, border-subtle/default), severity tokens (critical, success, danger, warning), and z-index scale.

**Addresses:** Color system definition (table stakes), prevents dark mode regression pitfall, establishes naming convention (--color-{category}-{variant}-{state}).

**Avoids:** Pitfall #2 (dark mode regression). Token system must be complete before migration begins.

### Phase 2: Logo Component
**Rationale:** Core brand asset needed for header, favicon, OG images. Must work in isolation before integration. Dark mode support is critical for developer audience.

**Delivers:** SVG logo component with variants (full/icon/wordmark), sizes (sm/md/lg/xl), `currentColor` fills, proper aria-label, optimized with SVGO.

**Uses:** Native React SVG (zero dependencies), Tailwind classes for sizing.

**Implements:** Logo component architecture pattern with theme support.

**Avoids:** Pitfall #3 (logo SSR/dark mode issues). Using `currentColor` prevents hardcoded color problems.

### Phase 3: Layout Refactor
**Rationale:** MUST happen before header addition. Current layout uses `min-h-screen flex flex-col` which will break when 64px header is added. Prevents layout shift cascade.

**Delivers:** Updated layout.tsx with `--header-height` variable, adjusted spacing on all routes, snapshots of current state for regression testing.

**Addresses:** Prevents critical Pitfall #1 (layout shift cascade). This is the most destructive pitfall if not handled properly.

**Avoids:** Breaking spacing across all 5 routes (/, /results/[token], /scan/[id], /privacy, /terms).

### Phase 4: Header & Navigation
**Rationale:** Now that layout structure is prepared, header can be safely added. Primary brand touchpoint.

**Delivers:** Sticky header component (64px, z-index 1020) with Logo, minimal nav (hidden on mobile), CTA button, integrated into layout.tsx.

**Uses:** Logo component from Phase 2, design tokens from Phase 1, layout structure from Phase 3.

**Implements:** Header architecture pattern with responsive behavior.

**Avoids:** Pitfall #7 (z-index conflicts) by using defined scale. Pitfall #8 (mobile confusion) by logo + CTA only.

### Phase 5: Icon System & Migration
**Rationale:** Replace emoji with professional SVG icons. Depends on color system being complete for proper theming. Systematic migration prevents inconsistency.

**Delivers:** lucide-react installed, icon barrel export (components/icons/index.ts), all emoji replaced on landing page, scan form, footer, grade summary (4 components migrated).

**Uses:** Lucide React library, design tokens for colors.

**Addresses:** Consistent icon system (table stakes), visual coherence.

**Avoids:** Pitfall #5 (icon inconsistency) by standardizing library and sizing patterns.

### Phase 6: Favicon & OG Image
**Rationale:** Can be done independently, low risk. Completes brand identity rollout.

**Delivers:** app/icon.tsx (dynamic generation), apple-icon.png, favicon.ico, favicon.svg with dark mode support, updated opengraph-image.tsx with logo overlay.

**Uses:** Logo component from Phase 2, Next.js Metadata API, ImageResponse.

**Addresses:** Favicon (table stakes), branded OG images (competitive feature).

**Avoids:** Pitfall #4 (cache invalidation) by generating all formats and using cache bust param.

### Phase Ordering Rationale

- **Foundation first:** Design tokens must exist before any component uses them (prevents regression)
- **Isolation before integration:** Logo built and tested standalone before header integration
- **Layout refactor before header:** Prevents layout shift cascade (most destructive pitfall)
- **Sequential dependencies:** Header needs logo, icons need color system, favicon needs logo
- **Risk management:** Non-breaking changes first (tokens, logo), structural changes second (layout), then additions (header, icons, favicon)

**Total estimated effort:** 10-12 hours across 6 phases (per ARCHITECTURE.md build order)

### Research Flags

**Phases with standard patterns (skip research):**
- **Phase 1 (Design Tokens):** Well-documented Tailwind v4 @theme, official docs verified
- **Phase 2 (Logo):** Standard React SVG component pattern
- **Phase 4 (Header):** Standard Next.js layout pattern
- **Phase 5 (Icons):** Lucide React has comprehensive docs
- **Phase 6 (Favicon):** Next.js Metadata API is well-documented

**No phases require `/gsd:research-phase`.** All patterns are established and documented in official sources.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Official Next.js 16 and Tailwind v4 docs verified. Lucide React actively maintained with clear docs. |
| Features | MEDIUM | Based on training data analysis of Linear, Vercel, Raycast, Stripe, 1Password. Not industry standards, but strong patterns. |
| Architecture | HIGH | Codebase-specific research. Current layout structure inspected, 17 components inventoried, dark mode patterns documented. |
| Pitfalls | HIGH | Derived from actual codebase state (layout pattern, dark mode classes, existing color usage). Patterns verified from Next.js/Tailwind best practices. |

**Overall confidence:** HIGH

Recommended stack is minimal and proven. Architecture patterns are standard. Pitfalls are derived from actual codebase inspection rather than speculation. Only uncertainty is feature prioritization (table stakes vs competitive), which is based on training data rather than user research.

### Gaps to Address

**Logo design itself:** Research covers technical integration, not visual design. Logo must be designed (geometric, works at 16x16px, conveys security). This is a design task, not development. **Handle during Phase 2 planning:** Source logo from designer or create placeholder SVG, validate it scales to favicon size before proceeding.

**Color palette refinement:** Research recommends "monochromatic + blue accent" but doesn't define exact color values. Existing codebase uses blue-400 through blue-900, severity colors (red/orange/yellow/green), and gray neutrals. **Handle during Phase 1:** Audit existing usage with `rg "blue-[0-9]" frontend/`, document current palette, define semantic tokens that map to existing usage to minimize visual changes.

**Mobile navigation pattern:** Research recommends "logo + CTA only" but doesn't validate this meets user needs. Current nav is minimal (Home, Privacy, Terms), but future features may need more nav items. **Handle during Phase 4:** Implement logo + CTA pattern, defer hamburger menu decision until Phase 4 validation. If nav grows, add menu in future phase.

**Accessibility validation:** Research identifies need for WCAG AA contrast and keyboard navigation but doesn't specify exact criteria. **Handle during Phase 6:** Add pre-deploy checklist item for Lighthouse accessibility audit, manual keyboard navigation test, contrast checker on all token combinations.

## Sources

### Primary (HIGH confidence)
- Next.js 16 Metadata API: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons
- Tailwind CSS v4 Theme: https://tailwindcss.com/docs/theme
- Lucide Icons: https://lucide.dev (library docs)
- ShipSecure codebase inspection: layout.tsx, globals.css, 17 component files, existing dark mode patterns

### Secondary (MEDIUM confidence)
- OKLch Color Space: https://oklch.com (perceptual uniformity rationale)
- WCAG 2.2 Contrast Guidelines: https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum
- Developer-focused brand patterns: Inferred from training data (Linear, Vercel, Raycast, Stripe, 1Password)

### Tertiary (LOW confidence)
- Feature prioritization: Training data analysis, not user research or industry standards
- Mobile navigation needs: Current minimal nav may not represent future state

---
**Research completed:** 2026-02-09
**Ready for roadmap:** Yes
**Total estimated effort:** 10-12 hours across 6 phases
