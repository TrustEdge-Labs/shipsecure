# Phase 14: Logo Component - Context

**Gathered:** 2026-02-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Create a theme-aware SVG logo component for ShipSecure that scales from 16x16 favicon to full desktop size. Three size variants (small lettermark, medium shield mark, large shield + wordmark). Uses `currentColor` for wordmark and brand blue for shield, with dark mode adaptation. Favicon generation and OG image updates are Phase 18.

</domain>

<decisions>
## Implementation Decisions

### Logo mark concept
- Geometric/angular shield shape with 2-3 facets — technical, digital fortress feel
- Checkmark inside as negative space (white cutout from shield body)
- Shield geometry stays consistent across all size variants (2-3 facets hold at any size)

### Wordmark treatment
- Brand name: "ShipSecure" in PascalCase
- Typography: Inter (system font already in use), medium/semi-bold weight
- No custom web font — consistent with existing app typography

### Color & mode behavior
- Shield mark: brand-primary blue (flat solid color, no gradient)
- Checkmark: white negative space cut from shield
- Wordmark: monochrome via currentColor (dark in light mode, light in dark mode)
- Dark mode: shield blue lightens slightly for better contrast/vibrancy against dark surfaces
- Both modes must meet WCAG AA contrast ratios (validated in Phase 13)

### Size adaptation — three variants
- **Small (favicon/16px):** Shield-shaped "S" lettermark — the letter S itself has angular, shield-like geometry
- **Medium (mobile header/32-48px):** Geometric shield mark only (with checkmark cutout), no wordmark
- **Large (desktop/full):** Shield mark + "ShipSecure" wordmark side by side

### Claude's Discretion
- Exact SVG path geometry for the shield facets
- Precise angular styling of the shield-S lettermark
- Spacing between shield mark and wordmark at large size
- Exact blue shade for dark mode lightened variant (must pass WCAG AA)

</decisions>

<specifics>
## Specific Ideas

- Shield should feel like a "digital fortress" — sharp angles, not rounded/friendly
- The small "S" lettermark should be recognizable as related to the full shield mark — same angular DNA
- Checkmark cutout should be simple and bold enough to read at medium sizes

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 14-logo-component*
*Context gathered: 2026-02-10*
