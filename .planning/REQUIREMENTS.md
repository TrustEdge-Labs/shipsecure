# Requirements: ShipSecure v1.3 Brand Identity

**Defined:** 2026-02-09
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow.

## v1.3 Requirements

Requirements for brand identity milestone. Each maps to roadmap phases.

### Logo

- [ ] **LOGO-01**: SVG logo mark renders correctly in light and dark mode
- [ ] **LOGO-02**: SVG wordmark renders correctly in light and dark mode
- [ ] **LOGO-03**: Logo scales cleanly from 16x16px (favicon) to full size
- [ ] **LOGO-04**: Logo uses `currentColor` for theme-aware rendering

### Color System

- [ ] **COLOR-01**: Design tokens defined via Tailwind v4 `@theme` with semantic naming
- [ ] **COLOR-02**: Dark mode overrides defined for all tokens via `prefers-color-scheme`
- [ ] **COLOR-03**: Existing components migrated from raw Tailwind colors to design tokens
- [ ] **COLOR-04**: All color combinations meet WCAG AA contrast (4.5:1)

### Favicon

- [ ] **FAV-01**: Branded favicon renders in browser tabs (ICO + SVG formats)
- [ ] **FAV-02**: Apple touch icon renders on iOS home screen (180x180 PNG)
- [ ] **FAV-03**: Favicon adapts to dark mode via SVG `prefers-color-scheme`

### Header

- [ ] **HDR-01**: Sticky header displays logo and "Scan Now" CTA on all pages
- [ ] **HDR-02**: Header shows wordmark on desktop, icon mark on mobile (<640px)
- [ ] **HDR-03**: Header does not cause layout shift on any existing route

### Icons

- [ ] **ICON-01**: Landing page feature grid uses SVG icons instead of emoji
- [ ] **ICON-02**: All SVG icons use consistent sizing and `currentColor` inheritance
- [ ] **ICON-03**: Decorative icons have `aria-hidden`, standalone icons have `aria-label`

### OG Images

- [ ] **OG-01**: Open Graph image includes branded logo and color system

## Future Requirements

Deferred to future release.

### Brand Polish

- **BRAND-01**: Animated logo micro-interaction on hover
- **BRAND-02**: Custom iconography replacing Lucide with brand-specific icons
- **BRAND-03**: Gradient accent system for CTAs and hero elements
- **BRAND-04**: SVG scroll animations on landing page

## Out of Scope

| Feature | Reason |
|---------|--------|
| Mascot/character logo | Too playful for security product, reduces credibility |
| Multiple brand colors | Monochromatic + single accent is cleaner for dev tools |
| Custom icon font | Performance overhead, accessibility issues; use inline SVG |
| Animated logo (autoplay) | Distracting, accessibility concern; defer to future |
| Full design system package | Premature for current scale; document patterns inline |
| Serif typography | Reduces technical feel; stick with Inter sans-serif |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| LOGO-01 | Phase 14 | Pending |
| LOGO-02 | Phase 14 | Pending |
| LOGO-03 | Phase 14 | Pending |
| LOGO-04 | Phase 14 | Pending |
| COLOR-01 | Phase 13 | Pending |
| COLOR-02 | Phase 13 | Pending |
| COLOR-03 | Phase 13 | Pending |
| COLOR-04 | Phase 13 | Pending |
| FAV-01 | Phase 18 | Pending |
| FAV-02 | Phase 18 | Pending |
| FAV-03 | Phase 18 | Pending |
| HDR-01 | Phase 16 | Pending |
| HDR-02 | Phase 16 | Pending |
| HDR-03 | Phase 15 | Pending |
| ICON-01 | Phase 17 | Pending |
| ICON-02 | Phase 17 | Pending |
| ICON-03 | Phase 17 | Pending |
| OG-01 | Phase 18 | Pending |

**Coverage:**
- v1.3 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0

**Coverage validation:** ✓ 100% coverage (18/18 requirements mapped)

---
*Requirements defined: 2026-02-09*
*Last updated: 2026-02-09 after roadmap creation*
