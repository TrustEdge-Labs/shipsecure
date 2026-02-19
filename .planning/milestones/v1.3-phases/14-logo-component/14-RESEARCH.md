# Phase 14: Logo Component - Research

**Researched:** 2026-02-10
**Domain:** SVG logo component design, React/TypeScript patterns, theme-aware color systems, responsive scaling
**Confidence:** HIGH

## Summary

Creating a theme-aware SVG logo component requires understanding three core technical domains: (1) SVG responsive scaling from favicon (16x16) to desktop, (2) CSS `currentColor` and `prefers-color-scheme` for dark mode adaptation, and (3) React/TypeScript component patterns for size variants. The existing design token system from Phase 13 provides validated OKLch blue primitives that meet WCAG AA contrast ratios in both light and dark modes.

For SVG negative space cutouts (checkmark within shield), use `fill-rule="evenodd"` with compound paths where inner paths subtract from outer paths. Convert wordmark text to paths for font portability. Define three component size variants (small/medium/large) using TypeScript discriminated unions. Place component in `frontend/components/logo.tsx` following existing Next.js patterns.

**Primary recommendation:** Build inline SVG component with `currentColor` for wordmark, CSS custom properties for shield blue, `fill-rule="evenodd"` for checkmark cutout, and size prop controlling which variant renders.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Logo mark concept:**
- Geometric/angular shield shape with 2-3 facets — technical, digital fortress feel
- Checkmark inside as negative space (white cutout from shield body)
- Shield geometry stays consistent across all size variants (2-3 facets hold at any size)

**Wordmark treatment:**
- Brand name: "ShipSecure" in PascalCase
- Typography: Inter (system font already in use), medium/semi-bold weight
- No custom web font — consistent with existing app typography

**Color & mode behavior:**
- Shield mark: brand-primary blue (flat solid color, no gradient)
- Checkmark: white negative space cut from shield
- Wordmark: monochrome via currentColor (dark in light mode, light in dark mode)
- Dark mode: shield blue lightens slightly for better contrast/vibrancy against dark surfaces
- Both modes must meet WCAG AA contrast ratios (validated in Phase 13)

**Size adaptation — three variants:**
- **Small (favicon/16px):** Shield-shaped "S" lettermark — the letter S itself has angular, shield-like geometry
- **Medium (mobile header/32-48px):** Geometric shield mark only (with checkmark cutout), no wordmark
- **Large (desktop/full):** Shield mark + "ShipSecure" wordmark side by side

### Claude's Discretion

- Exact SVG path geometry for the shield facets
- Precise angular styling of the shield-S lettermark
- Spacing between shield mark and wordmark at large size
- Exact blue shade for dark mode lightened variant (must pass WCAG AA)

### Specific Ideas from User

- Shield should feel like a "digital fortress" — sharp angles, not rounded/friendly
- The small "S" lettermark should be recognizable as related to the full shield mark — same angular DNA
- Checkmark cutout should be simple and bold enough to read at medium sizes

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope. Favicon generation and OG image updates are Phase 18.
</user_constraints>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.2.3 | UI component framework | Already in use (frontend/package.json) |
| TypeScript | ^5 | Type safety | Already configured (frontend/tsconfig.json) |
| Next.js | 16.1.6 | App framework | Existing setup with App Router |
| Tailwind CSS v4 | ^4 | Styling via design tokens | Phase 13 established @theme inline tokens |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| N/A | N/A | No additional libraries needed | Inline SVG in React component |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Inline SVG | SVGR loader | SVGR auto-converts .svg files to components, but adds build complexity and we only have one logo |
| Inline SVG | react-inlinesvg | Adds dependency for async SVG loading; unnecessary for single static logo |
| Size variants | CSS-only scaling | Can't conditionally render different SVG paths (lettermark vs shield vs shield+wordmark) |

**Installation:**
```bash
# No new dependencies required
# All tools already in frontend/package.json
```

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── components/
│   ├── logo.tsx              # Logo component (NEW)
│   ├── scan-form.tsx         # Existing components
│   └── ...
├── app/
│   └── globals.css           # Design tokens defined here (Phase 13)
```

### Pattern 1: Inline SVG Component with Size Variants

**What:** Define React component that renders different SVG markup based on size prop

**When to use:** Multi-variant logos where each size needs different visual elements (lettermark vs mark vs mark+wordmark)

**Example:**
```typescript
// Simplified pattern based on research findings
type LogoSize = 'small' | 'medium' | 'large'

interface LogoProps {
  size?: LogoSize
  className?: string
}

export function Logo({ size = 'large', className = '' }: LogoProps) {
  if (size === 'small') {
    return <svg viewBox="0 0 32 32" role="img" aria-label="ShipSecure">{/* S lettermark */}</svg>
  }

  if (size === 'medium') {
    return <svg viewBox="0 0 48 48" role="img" aria-label="ShipSecure">{/* Shield mark */}</svg>
  }

  // Large: shield + wordmark
  return <svg viewBox="0 0 200 48" role="img" aria-label="ShipSecure">{/* Shield + text */}</svg>
}
```

### Pattern 2: currentColor for Theme-Aware Wordmark

**What:** Use `fill="currentColor"` on SVG paths that should inherit text color from parent element

**When to use:** SVG elements that should adapt to surrounding text color and dark mode

**Example:**
```typescript
// Source: Web research (LogRocket, iconvectors.io, hidde.blog)
<svg>
  {/* Wordmark text paths use currentColor to inherit from parent */}
  <path d="M..." fill="currentColor" />
</svg>

// Usage in component tree:
<div className="text-text-primary"> {/* semantic token: gray-900 light, gray-100 dark */}
  <Logo size="large" />
</div>
```

### Pattern 3: CSS Custom Properties for Shield Color

**What:** Reference design token via `var(--color-brand-primary)` in SVG fill attribute

**When to use:** SVG elements that should use specific semantic color tokens, not inherit from currentColor

**Example:**
```typescript
// Shield mark uses brand-primary blue (validated WCAG AA in Phase 13)
<svg>
  <path d="M..." fill="var(--color-brand-primary)" />
</svg>
```

### Pattern 4: fill-rule="evenodd" for Negative Space Cutouts

**What:** Compound SVG path with outer shield and inner checkmark subpath; evenodd rule creates cutout effect

**When to use:** Creating negative space (knockout) effects without separate mask elements

**Example:**
```typescript
// Source: MDN (developer.mozilla.org/docs/Web/SVG/Attribute/fill-rule)
// Outer shield path winds clockwise, inner checkmark path winds counter-clockwise
<path
  fillRule="evenodd"
  d="M... [outer shield] Z M... [inner checkmark] Z"
  fill="var(--color-brand-primary)"
/>
```

### Pattern 5: ViewBox Without Fixed Width/Height

**What:** Define viewBox but omit width/height attributes to allow CSS control of sizing

**When to use:** Responsive SVGs that scale to container size

**Example:**
```typescript
// Source: Web research (svgontheweb.com, creativebloq.com)
<svg
  viewBox="0 0 200 48"
  className={className}
  // No width/height attributes = fully responsive
>
  {/* paths */}
</svg>
```

### Pattern 6: Dark Mode Adaptation via prefers-color-scheme

**What:** Tailwind v4 @theme inline tokens already handle dark mode via CSS custom properties; component references tokens

**When to use:** Already implemented in Phase 13; logo automatically adapts when referencing `--color-brand-primary`

**Example:**
```css
/* globals.css (already exists from Phase 13) */
@theme inline {
  --color-brand-primary: var(--primitive-blue-600); /* Light mode */
}

@media (prefers-color-scheme: dark) {
  @theme inline {
    /* Dark mode override - Phase 13 adjusted blue-600: oklch(0.585 0.220 262) */
    /* Lightened from light mode blue-600: oklch(0.546 0.245 262) */
    --color-brand-primary: var(--primitive-blue-600);
  }
}
```

### Anti-Patterns to Avoid

- **Fixed width/height on SVG element:** Breaks responsive scaling. Use viewBox only, let CSS control dimensions.
- **Hard-coded hex colors in paths:** Prevents theme adaptation. Use currentColor or CSS custom properties.
- **Separate mask/clipPath for simple cutouts:** Adds complexity. Use fill-rule="evenodd" compound paths instead.
- **SVG text element with font reference:** Breaks if font not installed. Convert text to paths for portability.
- **Multiple component files for variants:** Increases maintenance. Single component with size prop conditional rendering.
- **External .svg file imports:** Prevents dynamic color injection. Use inline SVG as JSX.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SVG optimization (decimal precision, redundant attributes) | Manual path cleanup | SVGO (svgo npm package or svgomg.net web UI) | Handles precision rounding, removes editor metadata, optimizes path commands |
| Contrast ratio validation | Manual calculation of relative luminance | Phase 13's contrast-checker.js (already exists) | Implements WCAG formula correctly, validates OKLch color space |
| Multiple favicon sizes (16x16, 32x32, etc.) | Hand-export each size | Defer to Phase 18 (favicon generation from SVG source) | Automated tooling handles pixel grid alignment at small sizes |

**Key insight:** SVG path optimization is deceptively complex (precision rounding, arc-to-curve conversion, removing redundant commands). SVGO has handled edge cases for years. For logo design, optimize final SVG with SVGO before converting to JSX.

## Common Pitfalls

### Pitfall 1: White Checkmark Disappears in Dark Mode

**What goes wrong:** Using literal white (#ffffff) for checkmark cutout causes it to vanish on dark backgrounds

**Why it happens:** Checkmark is negative space (cutout) showing background through shield, not a colored element

**How to avoid:** Checkmark is created via fill-rule="evenodd" compound path technique (inner path subtracts from outer), not a separate white path. Background color shows through naturally.

**Warning signs:** If you find yourself adding fill="white" to checkmark path, you're doing it wrong

### Pitfall 2: Losing Crispness at 16x16 Favicon Size

**What goes wrong:** Complex shield geometry with 2-3 facets becomes muddy at 16px

**Why it happens:** At 16x16, each pixel matters; multi-facet shield doesn't have enough pixels to render clearly

**How to avoid:** Use separate lettermark variant at small size (shield-styled "S" shape, not full shield). User already decided this in CONTEXT.md.

**Warning signs:** Testing shield mark at 16x16 in browser shows blurry/aliased edges

### Pitfall 3: SVG Doesn't Scale Responsively

**What goes wrong:** Logo renders at fixed size regardless of container width

**Why it happens:** SVG has width/height attributes without viewBox, or CSS doesn't control dimensions

**How to avoid:** Omit width/height attributes on SVG element. Define viewBox. Use className prop to apply Tailwind sizing utilities (e.g., `h-8`, `w-auto`).

**Warning signs:** Logo overflows container or has fixed pixel size in DevTools

### Pitfall 4: Text Not Accessible to Screen Readers

**What goes wrong:** Screen reader announces "graphic" without identifying logo

**Why it happens:** Missing aria-label or role="img" on SVG element

**How to avoid:** Always add `role="img"` and `aria-label="ShipSecure"` (or `aria-labelledby` referencing internal `<title>` element) to SVG elements used as images.

**Warning signs:** Running Axe DevTools or NVDA shows SVG without accessible name

### Pitfall 5: Inconsistent Blue Between Light/Dark Modes

**What goes wrong:** Dark mode blue fails WCAG AA contrast on gray-950 background

**Why it happens:** Using same blue-600 primitive in both modes without adjusting for dark surface contrast

**How to avoid:** Phase 13 already solved this. `--primitive-blue-600` has dark mode override (oklch lightness 0.546 → 0.585). Logo component uses `var(--color-brand-primary)` which references blue-600, automatically getting correct variant.

**Warning signs:** contrast-checker.js (Phase 13 tool) reports ratio < 4.5:1 for brand-primary on surface-primary in dark mode

### Pitfall 6: SVGO Removes fill-rule="evenodd"

**What goes wrong:** Running SVGO on compound path removes fill-rule attribute, breaking checkmark cutout

**Why it happens:** SVGO's cleanupAttrs plugin may remove fill-rule if it thinks nonzero is default

**How to avoid:** Configure SVGO to preserve fill-rule, or manually re-add fill-rule="evenodd" after optimization

**Warning signs:** Checkmark disappears after running SVGO; inspecting SVG shows fill-rule attribute missing

## Code Examples

Verified patterns from official sources and established conventions:

### Accessible SVG Logo Component

```typescript
// Source: WAI-ARIA best practices, MDN ARIA img role docs
// Pattern: role="img" + aria-label for screen reader support
interface LogoProps {
  size?: 'small' | 'medium' | 'large'
  className?: string
}

export function Logo({ size = 'large', className = '' }: LogoProps) {
  // Conditionally render variants based on size
  return (
    <svg
      viewBox="0 0 200 48"
      role="img"
      aria-label="ShipSecure"
      className={className}
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* SVG paths here */}
    </svg>
  )
}
```

### Compound Path with fill-rule="evenodd" for Cutout

```typescript
// Source: MDN SVG fill-rule docs, SitePoint SVG fill-rule tutorial
// Pattern: Outer path winds clockwise, inner path winds counter-clockwise
<path
  fillRule="evenodd"
  d="
    M 10 10 L 40 10 L 40 40 L 10 40 Z
    M 20 20 L 30 20 L 30 30 L 20 30 Z
  "
  fill="var(--color-brand-primary)"
/>
// Result: Inner 20x20 square cut out from outer 40x40 square
```

### currentColor for Theme-Aware Wordmark

```typescript
// Source: CSS-Tricks, hidde.blog SVG dark mode articles
// Pattern: Wordmark inherits text color from parent div
<div className="text-text-primary">
  <svg viewBox="0 0 200 48" role="img" aria-label="ShipSecure">
    {/* Shield mark - uses brand blue token */}
    <path d="M..." fill="var(--color-brand-primary)" />

    {/* Wordmark - uses currentColor to inherit text-primary token */}
    <path d="M..." fill="currentColor" />
  </svg>
</div>
```

### Responsive SVG Without Fixed Dimensions

```typescript
// Source: SVG on the Web guide, Creative Bloq responsive SVG article
// Pattern: viewBox without width/height allows CSS control
<svg
  viewBox="0 0 200 48"
  className="h-8 w-auto" // Tailwind controls size
  role="img"
  aria-label="ShipSecure"
>
  {/* paths */}
</svg>
```

### Size Variant Conditional Rendering

```typescript
// Source: TypeScript React patterns, discriminated unions for props
// Pattern: Different viewBox and paths per size
export function Logo({ size = 'large', className = '' }: LogoProps) {
  const baseClasses = className

  if (size === 'small') {
    // 16x16 lettermark for favicons
    return (
      <svg viewBox="0 0 32 32" className={baseClasses} role="img" aria-label="ShipSecure">
        {/* Shield-styled "S" lettermark paths */}
      </svg>
    )
  }

  if (size === 'medium') {
    // 32-48px shield mark only
    return (
      <svg viewBox="0 0 48 48" className={baseClasses} role="img" aria-label="ShipSecure">
        {/* Shield mark with checkmark cutout */}
      </svg>
    )
  }

  // Large: shield + wordmark
  return (
    <svg viewBox="0 0 200 48" className={baseClasses} role="img" aria-label="ShipSecure">
      {/* Shield mark + wordmark side by side */}
    </svg>
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| PNG logo sprites for 2x/3x DPI | SVG with viewBox (vector) | ~2015-2018 | Single SVG file scales to any DPI without quality loss |
| Separate light/dark logo files | CSS currentColor + prefers-color-scheme | ~2019-2021 | Single SVG adapts to theme via CSS custom properties |
| Font icons (icon fonts) | Inline SVG components | ~2020-2022 | Better accessibility, no FOIT/FOUT, selectively color parts |
| SVG mask/clipPath for cutouts | fill-rule="evenodd" compound paths | Always supported, now preferred | Simpler markup, fewer elements, better optimization |
| React.SVGProps generic typing | SVGProps<SVGSVGElement> specific typing | React 18+ (2022) | Accurate prop typing for SVG elements in TypeScript |
| defaultProps for defaults | ES6 default parameters in function signature | React 18.3+ (2024) | TypeScript-friendly, collocated defaults, no deprecation warnings |

**Deprecated/outdated:**
- **Icon fonts (FontAwesome, IcoMoon):** SVG components provide better accessibility, granular control, and tree-shaking
- **Base64-encoded inline data URIs:** Larger file size than plain SVG text, harder to maintain, no CSS theming
- **<img src="logo.svg">:** Can't inject CSS custom properties for theming; use inline SVG for logos that need theme adaptation

## Open Questions

1. **Exact shield facet geometry - how many facets exactly?**
   - What we know: User specified "2-3 facets" for technical/angular feel
   - What's unclear: Whether 2 or 3 provides better balance at small sizes
   - Recommendation: Prototype both in Figma/SVG editor; test at 16px, 32px, 48px. Fewer facets = better clarity at small sizes. Likely 2 facets (left and right side) for maximum simplicity.

2. **Wordmark spacing - how much gap between shield mark and text?**
   - What we know: Large variant has shield + wordmark side by side
   - What's unclear: Optimal spacing in design units (viewBox coordinates)
   - Recommendation: Start with 12-16 viewBox units (~25% of shield width) for visual breathing room. Test with design token color combinations.

3. **Small lettermark - shield-S hybrid geometry**
   - What we know: "S" should have angular, shield-like geometry related to full shield
   - What's unclear: How to translate 2-3 facet shield into letterform
   - Recommendation: Use angular diagonal cuts through "S" curves at positions matching shield facet angles. Keep checkmark as accent inside "S" if space permits at 16px.

4. **Dark mode blue lightness - exact value for Claude's discretion?**
   - What we know: Must pass WCAG AA (4.5:1) on gray-950 background; Phase 13 adjusted blue-600 to L=0.585 in dark mode
   - What's unclear: Whether logo-specific blue needs different lightness than general brand-primary
   - Recommendation: Use existing `--color-brand-primary` token (blue-600 with dark mode override). Already validated in Phase 13. Only create logo-specific token if visual testing shows logo needs more contrast than other brand-primary uses.

## Sources

### Primary (HIGH confidence)
- Next.js 16.1.6 Documentation (nextjs.org/docs) - App Router, component patterns
- Phase 13 SUMMARY.md files (.planning/phases/13-*) - Design token system, OKLch primitives, WCAG validation
- MDN Web Docs (developer.mozilla.org) - SVG fill-rule, preserveAspectRatio, ARIA img role, prefers-color-scheme
- W3C SVG 1.1 Specification (w3.org/TR/SVG11) - Clipping, masking, compositing
- W3C WAI-ARIA (w3.org/WAI) - SVG accessibility, ARIA roles for graphics

### Secondary (MEDIUM confidence)
- [React TypeScript Component Patterns](https://fettblog.eu/typescript-react-component-patterns/) - TypeScript discriminated unions for props
- [SVG on the Web Guide](https://svgontheweb.com/) - Best practices for viewBox, responsive SVGs
- [Creative Bloq: 10 Golden Rules for Responsive SVGs](https://www.creativebloq.com/how-to/10-golden-rules-for-responsive-svgs) - Remove width/height attributes
- [SitePoint: Understanding SVG fill-rule Property](https://www.sitepoint.com/understanding-svg-fill-rule-property/) - evenodd vs nonzero winding
- [CSS-Tricks: SVG Text for Typographic Designs](https://css-tricks.com/svg-text-typographic-designs/) - When to use text vs paths
- [Sara Soueidan: Tips for Better SVGs](https://www.sarasoueidan.com/blog/svg-tips-for-designers/) - Text to outlines tradeoffs
- [Cassidy James: SVG Light/Dark Style Support](https://cassidyjames.com/blog/prefers-color-scheme-svg-light-dark/) - prefers-color-scheme in SVG
- [Hidde de Vries: Single Color SVG Icons in Dark Mode](https://hidde.blog/making-single-color-svg-icons-work-in-dark-mode/) - currentColor pattern
- [TPGi: Using ARIA to Enhance SVG Accessibility](https://www.tpgi.com/using-aria-enhance-svg-accessibility/) - role="img" and aria-labelledby
- [Evil Martians: How to Favicon in 2026](https://evilmartians.com/chronicles/how-to-favicon-in-2021-six-files-that-fit-most-needs) - SVG favicon best practices
- [Favicon.im: Complete Favicon Guide 2025](https://favicon.im/blog/complete-favicon-size-format-guide-2025) - Size and format standards
- [SVG AI: Logo Design Principles](https://www.svgai.org/blog/svg-logo-design-principles) - Geometric fundamentals, angular designs
- [LogoDesign.net: Geometric Logo Design](https://www.logodesign.net/blog/geometric-logo-design/) - Sharp angles for technical brands

### Tertiary (LOW confidence)
- [Medium: Best Way to Organize Icons in Next.js](https://medium.com/@franciscomoretti/the-best-way-to-organize-icons-in-a-next-js-site-7615022f3bf4) - Component organization patterns (low confidence: Medium blog post, not official)
- [DevCamp: Logo Component Custom Size Prop](https://devcamp.com/trails/comprehensive-react-development-tutorial/campsites/api-search-engine/guides/customizing-logo-component-take-custom-size-prop) - Size prop pattern (low confidence: tutorial site, not authoritative)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries verified in frontend/package.json; React 19.2.3, Next.js 16.1.6, TypeScript 5, Tailwind v4
- Architecture patterns: HIGH - Patterns verified against MDN docs, W3C specs, established React/TypeScript conventions
- Design token integration: HIGH - Phase 13 SUMMARY files document OKLch primitives, semantic tokens, WCAG validation
- SVG techniques: HIGH - fill-rule, currentColor, viewBox, preserveAspectRatio verified in MDN and W3C specs
- Accessibility: HIGH - ARIA img role, aria-label verified in W3C WAI-ARIA and TPGi guidelines
- Component organization: MEDIUM - Based on Next.js docs (official) but icon organization is project-specific convention

**Research date:** 2026-02-10
**Valid until:** 2026-03-10 (30 days) - SVG and CSS specs stable; React/Next.js patterns evolving slowly
