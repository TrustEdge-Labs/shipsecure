# Phase 15: Layout Refactor - Research

**Researched:** 2026-02-10
**Domain:** CSS layout refactoring, CSS custom properties, layout shift prevention, Next.js App Router layout patterns
**Confidence:** HIGH

## Summary

Preparing a layout for header integration without causing layout shift requires three core practices: (1) defining a CSS custom property for header height before the header exists, (2) ensuring existing spacing remains unchanged when the variable is introduced, and (3) using CSS Grid or Flexbox patterns that accommodate the future sticky header without reflow. The project already uses Tailwind CSS v4's `@theme inline` directive for design tokens in `globals.css`, making it the natural location for the `--header-height` variable.

The current layout uses a flex column structure (`flex flex-col min-h-screen`) with footer at the bottom. Adding a `--header-height: 64px` token requires verification that no existing page spacing depends on viewport height calculations that would break when a 64px header consumes vertical space. The CSS Grid `grid-template-rows: auto 1fr auto` pattern is the modern standard for header/main/footer layouts that prevent layout shift, as it allocates precise space for each section.

Layout shift prevention depends on reserving space for the header before it renders. Since Phase 16 will add a sticky header, this phase must establish the variable and verify that all routes maintain current visual spacing. No pages currently use `min-h-screen` on main content that would conflict with a sticky header.

**Primary recommendation:** Define `--header-height: 64px` in `globals.css` using `@theme inline`, document the token in code comments, verify all routes maintain spacing unchanged, and prepare layout structure for CSS Grid refactor in Phase 16.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tailwind CSS v4 | 4.x | Utility-first CSS with native CSS variable support | Industry standard for design token systems, `@theme inline` directive eliminates JS config |
| Next.js App Router | 16.1.6 | React framework with file-based routing | Official Next.js pattern for layouts, supports nested layouts and server components |
| CSS Grid | Native CSS | Two-dimensional layout system | Modern standard for header/main/footer layouts, prevents layout shift via explicit row sizing |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Flexbox | Native CSS | One-dimensional layout | Current implementation for `min-h-screen` footer pattern, simpler than Grid for single-axis layouts |
| CSS Custom Properties | Native CSS | Runtime-accessible design tokens | Already used extensively via Tailwind v4 `@theme inline`, no additional dependencies |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| CSS Grid | Flexbox with padding | Flexbox simpler but requires padding calculations, Grid more explicit and prevents shift |
| `@theme inline` | Regular `:root` variables | `@theme inline` integrates with Tailwind utilities, `:root` requires manual class generation |
| 64px header | 80px or dynamic calc() | 64px is standard header height, dynamic sizing adds complexity without clear benefit |

**Installation:**
No new dependencies required. Uses existing Tailwind CSS v4 and native CSS features.

## Architecture Patterns

### Recommended File Structure
```
frontend/
├── app/
│   ├── layout.tsx              # Root layout - add Grid structure
│   ├── globals.css             # Add --header-height token here
│   ├── page.tsx                # Landing page - verify spacing
│   ├── results/[token]/page.tsx # Results page - verify spacing
│   ├── privacy/page.tsx        # Privacy page - verify spacing
│   └── terms/page.tsx          # Terms page - verify spacing
└── components/
    └── footer.tsx              # Footer component - no changes needed
```

### Pattern 1: CSS Custom Property for Layout Dimensions
**What:** Define `--header-height` as a Tailwind theme variable before the header component exists
**When to use:** Before adding any fixed or sticky positioned elements that affect layout flow
**Example:**
```css
/* frontend/app/globals.css */
/* Source: Tailwind CSS v4 docs + project design token pattern */

@theme inline {
  /* ... existing design tokens ... */

  /* Layout Dimensions */
  --header-height: 64px;  /* Reserve space for Phase 16 sticky header */
}
```

**Why this works:**
- Tailwind v4's `@theme inline` generates utility classes automatically
- CSS custom properties are runtime-accessible for JavaScript if needed
- Defining the variable first prevents layout shift when header is added later

### Pattern 2: CSS Grid Three-Row Layout (Header/Main/Footer)
**What:** Use CSS Grid with `grid-template-rows: auto 1fr auto` for predictable layout
**When to use:** When adding sticky headers to prevent Cumulative Layout Shift (CLS)
**Example:**
```tsx
// frontend/app/layout.tsx
// Source: Next.js App Router docs + CSS Grid sticky header pattern

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} font-sans antialiased`}>
        <div className="grid grid-rows-[auto_1fr_auto] min-h-screen">
          {/* Header slot - empty for Phase 15, populated in Phase 16 */}
          <div className="h-[--header-height]" aria-hidden="true">
            {/* Spacer reserves vertical space for future sticky header */}
          </div>

          {/* Main content - 1fr fills remaining space */}
          <main className="min-h-0">
            {children}
          </main>

          {/* Footer - auto height */}
          <Footer />
        </div>
      </body>
    </html>
  );
}
```

**Why this pattern:**
- `grid-template-rows: auto 1fr auto` allocates precise space: header takes natural height, footer takes natural height, main fills the rest
- `min-h-0` on main allows scrolling without breaking grid sizing
- Header spacer reserves 64px vertical space before sticky header exists, preventing layout shift in Phase 16

### Pattern 3: Flexbox Fallback (Current Implementation)
**What:** Keep existing `flex flex-col min-h-screen` pattern if Grid migration is deferred
**When to use:** If Phase 15 scope is limited to variable definition only
**Example:**
```tsx
// frontend/app/layout.tsx (current)
// Source: Project codebase

<body className={`${inter.variable} font-sans antialiased`}>
  <div className="flex flex-col min-h-screen">
    {/* Header slot - add padding-top when header becomes sticky */}
    <div className="flex-1">
      {children}
    </div>
    <Footer />
  </div>
</body>
```

**Tradeoff:**
- Simpler than Grid, requires less refactoring
- When sticky header is added, pages need `pt-[--header-height]` or content shifts behind header
- Less explicit about layout intentions compared to Grid

### Anti-Patterns to Avoid
- **Fixed padding before header exists:** Don't add `pt-16` to pages prematurely - this creates extra whitespace before header is added
- **Viewport height calculations without header offset:** Avoid `h-screen` on main content when sticky header consumes vertical space - use `min-h-0` with Grid instead
- **JavaScript-based height calculations:** Don't measure header height via `useEffect` - CSS variables are declarative and faster
- **Inline styles for layout dimensions:** Don't hardcode `64px` in multiple places - use the CSS custom property for single source of truth

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Layout shift detection | Custom viewport resize listeners + manual height tracking | CSS `grid-template-rows: auto 1fr auto` | Grid allocates space declaratively, no JavaScript required, prevents CLS automatically |
| Header height synchronization | JavaScript to measure and update padding | CSS custom property `--header-height` | Single source of truth, runtime-accessible, works with Tailwind utilities |
| Responsive header sizing | Multiple media query breakpoints with hardcoded heights | Single variable with Tailwind responsive prefixes | `md:h-[80px]` override if needed, but 64px standard works for all viewports |
| Sticky header positioning | Absolute positioning + scroll listeners | CSS `position: sticky; top: 0;` | Native browser optimization, better performance, no scroll jank |

**Key insight:** Modern CSS (Grid, custom properties, sticky positioning) eliminates the need for JavaScript-based layout management. CSS-only solutions are faster, more maintainable, and prevent layout shift by design rather than through runtime calculations.

## Common Pitfalls

### Pitfall 1: Adding Header Variable Without Testing Existing Spacing
**What goes wrong:** Defining `--header-height` doesn't break anything, but changing layout structure (Flexbox to Grid, adding padding) can shift existing content unexpectedly
**Why it happens:** Pages are designed assuming no vertical offset - adding header space changes baseline alignment
**How to avoid:**
1. Define `--header-height` variable first (safe, no visual impact)
2. Visually verify all routes BEFORE changing layout structure
3. Take screenshots of: `/`, `/results/[token]`, `/privacy`, `/terms` pages
4. Compare before/after when applying layout changes
**Warning signs:** Footer position changes, hero section shifts down, content no longer centered

### Pitfall 2: Using `h-screen` on Main Content with Sticky Header
**What goes wrong:** `h-screen` means 100vh, but sticky header consumes 64px, causing vertical scroll when none should exist
**Why it happens:** `100vh` doesn't account for sticky elements - viewport height includes the header overlap area
**How to avoid:**
- Use CSS Grid with `1fr` for main content instead of `h-screen`
- If Flexbox is kept, use `flex-1` instead of `h-screen`
- Reserve `h-screen` for full-bleed components (modals, hero sections with background images)
**Warning signs:** Unexpected vertical scrollbar on pages that should fit in viewport, double scrollbars

### Pitfall 3: Scoping Issues with CSS Custom Properties
**What goes wrong:** `--header-height` defined in component scope instead of `:root` or `@theme inline`, not accessible to other components
**Why it happens:** CSS variables are scoped to the element they're defined on and descendants
**How to avoid:**
- Define layout dimensions in `globals.css` using `@theme inline` (Tailwind v4 pattern)
- This makes variables available globally AND generates Tailwind utilities
- For Tailwind v4 specifically: use `@theme inline` when referencing other variables
**Warning signs:** Utility class `h-[--header-height]` doesn't work, variable shows as undefined in DevTools

### Pitfall 4: Forgetting aria-hidden on Header Spacer
**What goes wrong:** Screen readers announce empty div as separate content region, confusing navigation
**Why it happens:** Spacer div is purely presentational for layout, not semantic content
**How to avoid:** Add `aria-hidden="true"` to any spacer elements used for layout reservation
**Warning signs:** Screen reader testing reveals extra unnamed regions, keyboard navigation behaves oddly

### Pitfall 5: Layout Shift from Footer Position Change
**What goes wrong:** Current footer uses `mt-auto` in Flexbox context - switching to Grid changes how footer positioning works
**Why it happens:** Grid rows behave differently than Flex items with `margin: auto`
**How to avoid:**
- Test footer position on short pages (Privacy, Terms) vs long pages (Landing, Results)
- Grid's `auto` row size handles this automatically, but verify sticky behavior
- Ensure footer stays at bottom on short pages, flows naturally on long pages
**Warning signs:** Footer floats mid-page on short content, or overlaps main content

## Code Examples

Verified patterns from official sources:

### Example 1: Define Header Height Token
```css
/* frontend/app/globals.css */
/* Source: Tailwind CSS v4 @theme inline documentation */

@theme inline {
  /* ... existing color tokens ... */

  /* ===================================================================
     Layout Dimensions
     =================================================================== */

  /* Header height - reserved for Phase 16 sticky header
   * Used for: sticky header height, main content padding-top offset
   * DO NOT change without verifying all routes maintain spacing */
  --header-height: 64px;
}
```

**Usage in components:**
```tsx
// Tailwind utility class (auto-generated by @theme inline)
<div className="h-[--header-height]">Spacer</div>

// Arbitrary value with calc()
<main className="pt-[calc(var(--header-height)+1rem)]">
  {/* Extra 1rem breathing room */}
</main>
```

### Example 2: CSS Grid Layout (Recommended for Phase 16)
```tsx
// frontend/app/layout.tsx
// Source: Next.js App Router layout docs + CSS Grid sticky header pattern

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} font-sans antialiased`}>
        {/* Grid with auto/1fr/auto rows prevents layout shift */}
        <div className="grid grid-rows-[auto_1fr_auto] min-h-screen">
          {/* Phase 15: Empty spacer reserves space
              Phase 16: Replace with <Header /> sticky component */}
          <div className="h-[--header-height]" aria-hidden="true" />

          {/* Main content fills available space, min-h-0 allows scrolling */}
          <main className="min-h-0">
            {children}
          </main>

          {/* Footer takes natural height */}
          <Footer />
        </div>
      </body>
    </html>
  );
}
```

### Example 3: Flexbox Alternative (If Grid Deferred)
```tsx
// frontend/app/layout.tsx (alternative approach)
// Source: Current project codebase pattern

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} font-sans antialiased`}>
        <div className="flex flex-col min-h-screen">
          {/* Phase 15: Define variable only, no layout changes
              Phase 16: Add <Header /> and apply pt-[--header-height] to flex-1 div */}

          <div className="flex-1">
            {children}
          </div>

          <Footer />
        </div>
      </body>
    </html>
  );
}
```

**When sticky header is added in Phase 16:**
```tsx
<div className="flex flex-col min-h-screen">
  <Header className="sticky top-0 z-50" />
  <div className="flex-1 pt-[--header-height]">
    {children}
  </div>
  <Footer />
</div>
```

### Example 4: Visual Regression Testing Script
```bash
# Verify no layout shift before/after variable addition
# Source: Layout shift prevention best practices

# Take before screenshots
npm run dev
# Visit: /, /results/test-token, /privacy, /terms
# Screenshot each page's hero section and footer

# Add --header-height variable to globals.css
# Refresh pages without restarting dev server (CSS hot-reload)

# Compare screenshots - should be pixel-identical
# If any differences: investigate before proceeding
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| JavaScript height calculation + setState | CSS custom properties + `@theme inline` | Tailwind v4 (2024) | Eliminates runtime calculation, integrates with utility classes |
| Flexbox with `margin-top` offset | CSS Grid with `grid-template-rows` | 2023+ (Grid maturity) | Declarative space allocation, prevents layout shift by design |
| Fixed header with `padding-top` | Sticky header with Grid row | 2022+ (Safari sticky support) | Sticky keeps header in flow, Grid reserves space, no manual offset needed |
| Hardcoded pixel values in components | Design tokens via CSS variables | Tailwind v4 (2024) | Single source of truth, theme-aware, runtime accessible |

**Deprecated/outdated:**
- **`position: fixed` headers without layout offset:** Causes content to shift behind header, poor CLS score for Core Web Vitals
- **JavaScript `useEffect` to measure header height:** CSS variables are synchronous and declarative, no flash of incorrect sizing
- **Separate mobile/desktop header heights:** Modern responsive design uses same height, only content changes (icon vs wordmark)
- **Viewport height minus header calc in JavaScript:** CSS Grid `1fr` handles this automatically

## Open Questions

1. **Should Phase 15 include Grid migration or just variable definition?**
   - What we know: Grid is superior for layout shift prevention, but adds scope
   - What's unclear: Whether Grid refactor should be in Phase 15 or Phase 16
   - Recommendation: Define variable in Phase 15 (low risk), defer Grid migration to Phase 16 when header component is added (test Grid + header together)

2. **Do any pages use viewport-height calculations that would break?**
   - What we know: Current codebase uses `min-h-screen` on root flex container, individual pages don't have `h-screen` on main content
   - What's unclear: Whether any components use `calc(100vh - X)` patterns
   - Recommendation: Grep codebase for `vh` and `calc` in CSS/className strings, verify no conflicts

3. **Should header height be responsive (different sizes on mobile/desktop)?**
   - What we know: Standard practice is consistent height, only content changes (per HDR-02: wordmark vs icon)
   - What's unclear: Whether mobile needs smaller header for screen real estate
   - Recommendation: Use 64px consistently, proven to work across all viewport sizes without cramping mobile

## Sources

### Primary (HIGH confidence)
- [Tailwind CSS v4: Theme Variables](https://tailwindcss.com/docs/theme) - `@theme inline` directive, CSS variable generation
- [Next.js App Router: Pages and Layouts](https://nextjs.org/docs/app/building-your-application/routing/pages-and-layouts) - Layout structure, nested layouts, root layout requirements
- [MDN: CSS Grid Sticky Footers](https://developer.mozilla.org/en-US/docs/Web/CSS/How_to/Layout_cookbook/Sticky_footers) - `grid-template-rows: auto 1fr auto` pattern
- Project codebase: `frontend/app/layout.tsx`, `frontend/app/globals.css` - Current implementation

### Secondary (MEDIUM confidence)
- [Preventing Layout Shift with Modern CSS](https://blog.openreplay.com/preventing-layout-shift-modern-css/) - CLS prevention strategies, Grid vs Flexbox tradeoffs
- [CSS Grid for Sticky Headers and Footers](https://css-tricks.com/how-to-use-css-grid-for-sticky-headers-and-footers/) - Grid row sizing patterns
- [Smashing Magazine: Sticky Headers And Full-Height Elements](https://www.smashingmagazine.com/2024/09/sticky-headers-full-height-elements-tricky-combination/) - Mobile viewport height issues, Grid solutions
- [Tailwind CSS v4.0 Announcement](https://tailwindcss.com/blog/tailwindcss-v4) - CSS-first configuration, `@theme` directive rationale

### Tertiary (LOW confidence - requires validation)
- WebSearch: "CSS custom properties layout header height prevent layout shift 2026" - General best practices, not framework-specific
- WebSearch: "Next.js App Router header layout refactor best practices 2026" - Community patterns, not official docs
- WebSearch: "fixed header layout shift prevention sticky positioning best practices 2026" - `position: sticky` vs `position: fixed` tradeoffs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Tailwind v4 and Next.js patterns verified from official docs
- Architecture: HIGH - Grid pattern confirmed in MDN, Tailwind v4 `@theme inline` confirmed in docs, Next.js layout structure official
- Pitfalls: MEDIUM-HIGH - Layout shift and scoping issues verified, but visual regression testing is project-specific

**Research date:** 2026-02-10
**Valid until:** 2026-03-12 (30 days - stable CSS patterns, no breaking changes expected in Tailwind v4.x or Next.js 16.x minor versions)
