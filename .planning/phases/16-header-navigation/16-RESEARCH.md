# Phase 16: Header & Navigation - Research

**Researched:** 2026-02-11
**Domain:** Next.js App Router sticky header with responsive logo and CTA
**Confidence:** HIGH

## Summary

This phase implements a sticky header component across all application pages with responsive logo display (wordmark on desktop, icon on mobile) and a "Scan Now" call-to-action button. The implementation leverages Next.js 16 App Router patterns, Tailwind CSS v4 positioning utilities, and the existing design token system established in Phase 13-15.

The header will be a **Server Component by default** (no interactivity required for basic display), inserted into the root layout above the `{children}` outlet. The sticky positioning creates accessibility challenges for keyboard navigation that must be addressed via `scroll-padding-top` CSS to ensure focused elements don't hide behind the header (WCAG 2.2 Success Criterion 2.4.11 Focus Not Obscured).

The existing `--header-height: 64px` CSS variable in `globals.css` provides the correct offset value. The Logo component uses Next.js Image optimization with static imports, requiring a responsive display strategy using Tailwind breakpoint classes (`hidden`, `block`, `sm:hidden`, `sm:block`) rather than art direction via `<picture>` element since we have a single PNG with both wordmark and icon.

**Primary recommendation:** Create a Server Component at `components/header.tsx` using Tailwind's `sticky top-0` positioning, semantic design tokens for colors, and `scroll-padding-top: var(--header-height)` on `:root` to prevent keyboard focus obscuration.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js | 16.1.6 | App Router framework | Project standard, provides Link/Image components and layout system |
| Tailwind CSS | 4.x | Utility-first styling | Project standard, v4 uses CSS-based config via @theme inline |
| next/link | 16.1.6 | Client-side navigation | Official Next.js routing component with automatic prefetching |
| next/image | 16.1.6 | Optimized image component | Official Next.js image optimization with automatic srcset generation |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| usePathname | 16.1.6 | Active link detection | Only if implementing active nav link styling (requires "use client") |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Server Component header | Client Component with useState | Server Component is sufficient—no interactivity needed for logo/nav/CTA display |
| Tailwind breakpoints | CSS media queries | Tailwind provides mobile-first breakpoints matching design system standards |
| Single PNG logo | Art direction with picture element | Single PNG simpler—just toggle visibility at breakpoints |

**Installation:**
No new packages required. All dependencies already in project.

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── components/
│   ├── header.tsx          # NEW: Sticky header Server Component
│   ├── logo.tsx            # EXISTS: Logo with size variants
│   └── footer.tsx          # EXISTS: Reference pattern for Link usage
├── app/
│   ├── layout.tsx          # MODIFY: Insert <Header /> above {children}
│   └── globals.css         # MODIFY: Add scroll-padding-top for a11y
```

### Pattern 1: Sticky Header in Root Layout
**What:** Place header component in `app/layout.tsx` to share across all routes
**When to use:** Header appears on all pages (requirement HDR-01)
**Example:**
```typescript
// Source: Next.js 16 official docs + project layout.tsx
// app/layout.tsx
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <div className="flex flex-col min-h-screen">
          <Header /> {/* NEW: Phase 16 */}
          <div className="flex-1">
            {children}
          </div>
          <Footer />
        </div>
      </body>
    </html>
  )
}
```

### Pattern 2: Server Component Header (Default)
**What:** Header as Server Component unless interactivity needed
**When to use:** Static nav links, logo, CTA button
**Example:**
```typescript
// Source: Next.js docs - Server and Client Components
// components/header.tsx - Server Component by default
import Link from 'next/link'
import { Logo } from './logo'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
        <Link href="/" className="flex items-center">
          {/* Logo implementation here */}
        </Link>
        <Link
          href="/#scan-form"
          className="px-6 py-2 bg-brand-primary hover:bg-brand-primary-hover text-white font-semibold rounded-lg transition"
        >
          Scan Now
        </Link>
      </div>
    </header>
  )
}
```

### Pattern 3: Responsive Logo Display with Tailwind Breakpoints
**What:** Show different logo content at different screen sizes using visibility utilities
**When to use:** Single image with both wordmark and icon, switching at 640px (sm:)
**Example:**
```typescript
// Source: Tailwind CSS responsive design docs + Next.js Image docs
// components/header.tsx
import Image from 'next/image'

// Wordmark: visible on desktop (>=640px), hidden on mobile
<div className="hidden sm:block">
  <Image src="/logo.png" alt="ShipSecure" width={192} height={64} priority />
</div>

// Icon mark: visible on mobile (<640px), hidden on desktop
<div className="sm:hidden">
  <Image src="/logo.png" alt="ShipSecure" width={48} height={48} priority />
</div>
```

### Pattern 4: Hash Link Navigation for Same-Page CTAs
**What:** Use hash links to scroll to anchor IDs on the same page
**When to use:** "Scan Now" CTA should scroll to scan form on homepage
**Example:**
```typescript
// Source: Next.js Link docs - "Scrolling to an id"
// Header CTA links to scan form
<Link href="/#scan-form" className="...">
  Scan Now
</Link>

// Landing page scan form with id anchor
<div id="scan-form" className="...">
  <ScanForm />
</div>
```

### Anti-Patterns to Avoid
- **Using position: fixed instead of sticky:** Sticky keeps header in document flow, fixed removes it (causes layout jump)
- **Forgetting z-index on sticky header:** Other elements may render on top without explicit stacking context
- **Hard-coding header height in multiple places:** Use CSS variable `--header-height` for single source of truth
- **Making header a Client Component without reason:** Server Component is default, only use "use client" if adding interactivity (mobile menu, active link styling)
- **Not setting priority on above-fold logo image:** Logo is LCP candidate, use `priority` prop to preload

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Image optimization | Custom responsive image logic | Next.js Image component | Automatic srcset generation, format conversion (WebP/AVIF), lazy loading, built-in optimization API |
| Client-side routing | Manual window.location or <a> tags | next/link | Prefetching, client-side transitions, maintains scroll position, proper SPA navigation |
| Responsive breakpoints | Custom media query hooks | Tailwind breakpoint classes | Mobile-first system (sm: md: lg:), matches design system, no JS needed |
| Keyboard focus visibility | Custom scroll event listeners | CSS scroll-padding-top | Native browser behavior, works with Tab/Shift+Tab, no JavaScript overhead |
| Logo size variants | Multiple logo files or custom sizing | Single Image with className/style | Aspect ratio preserved automatically, easier asset management |

**Key insight:** Next.js provides battle-tested primitives for navigation (Link), images (Image), and layouts (RootLayout) that handle edge cases like prefetching race conditions, image format negotiation, and scroll restoration. Custom solutions reintroduce solved problems.

## Common Pitfalls

### Pitfall 1: Sticky Header Obscuring Keyboard Focus
**What goes wrong:** When keyboard users Tab backwards, focused elements disappear behind sticky header (WCAG 2.2 SC 2.4.11 failure)
**Why it happens:** Default browser scroll-into-view doesn't account for sticky/fixed headers
**How to avoid:** Add `scroll-padding-top: var(--header-height)` to `:root` in globals.css
**Warning signs:** Manual testing with Tab/Shift+Tab shows focus indicator hidden by header
**Source:** [TPG article on preventing focus obscuration](https://www.tpgi.com/prevent-focused-elements-from-being-obscured-by-sticky-headers/)

### Pitfall 2: Z-Index Stacking Context Issues
**What goes wrong:** Modals, dropdowns, or other overlays appear behind sticky header
**Why it happens:** Sticky elements create stacking contexts; child z-index values don't affect parent context
**How to avoid:** Set explicit `z-50` on sticky header, use consistent z-index scale (10/20/30/40/50)
**Warning signs:** Overlays or interactive elements visually beneath header despite high z-index

### Pitfall 3: Logo Image Not Prioritized
**What goes wrong:** Logo loads late, causing Largest Contentful Paint (LCP) delay
**Why it happens:** Default `loading="lazy"` defers image load; logo is above fold
**How to avoid:** Add `priority` prop to Image component for above-fold logo
**Warning signs:** Core Web Vitals show poor LCP, logo flashes in after page render
**Source:** [Next.js Image priority docs](https://nextjs.org/docs/app/api-reference/components/image#priority)

### Pitfall 4: Responsive Logo Using Art Direction Pattern
**What goes wrong:** Using `<picture>` or getImageProps for simple visibility toggle
**Why it happens:** Confusing "different images" with "same image, different size"
**How to avoid:** Use Tailwind `hidden`/`block` with breakpoints for single image source
**Warning signs:** Unnecessary complexity, both logo versions downloaded on mobile

### Pitfall 5: Hard-Coding Header Height
**What goes wrong:** Changing header height requires updates in multiple files (header, scroll-padding, page padding)
**Why it happens:** Not using CSS variable for dimension token
**How to avoid:** Reference `--header-height` everywhere, update once in globals.css
**Warning signs:** Misaligned spacing, overlapping content when header height changes

### Pitfall 6: Server/Client Component Confusion
**What goes wrong:** Adding "use client" to header without understanding when it's needed
**Why it happens:** Assuming all components need client-side JavaScript
**How to avoid:** Default to Server Component; only add "use client" for interactivity (mobile menu, usePathname for active links)
**Warning signs:** Larger client bundle, unnecessary hydration, components that could be static are interactive

## Code Examples

Verified patterns from official sources and project conventions:

### Sticky Header with Design Tokens
```typescript
// Source: Project globals.css semantic tokens + Tailwind docs
// components/header.tsx
import Link from 'next/link'
import { Logo } from './logo'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
        {/* Logo and nav here */}
      </div>
    </header>
  )
}
```

### Responsive Logo with Next.js Image
```typescript
// Source: Next.js Image docs + project logo.png
// components/header.tsx
import Link from 'next/link'
import Image from 'next/image'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
        <Link href="/" className="flex items-center gap-2">
          {/* Desktop: Full wordmark */}
          <div className="hidden sm:block">
            <Image
              src="/logo.png"
              alt="ShipSecure"
              width={192}
              height={64}
              priority
            />
          </div>
          {/* Mobile: Icon only */}
          <div className="sm:hidden">
            <Image
              src="/logo.png"
              alt="ShipSecure"
              width={48}
              height={48}
              priority
            />
          </div>
        </Link>
        {/* CTA here */}
      </div>
    </header>
  )
}
```

### CTA Button Matching Scan Form Style
```typescript
// Source: Project scan-form.tsx button pattern
// components/header.tsx
import Link from 'next/link'

export function Header() {
  return (
    <header className="...">
      <div className="...">
        {/* Logo */}
        <Link
          href="/#scan-form"
          className="px-6 py-2 bg-brand-primary hover:bg-brand-primary-hover text-white font-semibold rounded-lg transition"
        >
          Scan Now
        </Link>
      </div>
    </header>
  )
}
```

### Accessibility: Scroll Padding for Keyboard Focus
```css
/* Source: WCAG 2.2 SC 2.4.11 guidance + TPG article
 * app/globals.css - Add to existing :root block */
:root {
  /* Prevent sticky header from obscuring keyboard focus */
  scroll-padding-top: var(--header-height);
}
```

### Layout Integration
```typescript
// Source: Project layout.tsx + Next.js App Router docs
// app/layout.tsx
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.variable} font-sans antialiased`}>
        <div className="flex flex-col min-h-screen">
          <Header />
          <div className="flex-1">
            {children}
          </div>
          <Footer />
        </div>
      </body>
    </html>
  )
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| position: fixed with manual offset | position: sticky | CSS3 (2012), widely supported 2020+ | Sticky keeps header in document flow, no manual spacing needed |
| Manual scroll listeners for focus | scroll-padding-top CSS | WCAG 2.2 (2023) | Native browser handles focus visibility without JavaScript |
| Multiple logo files for responsive | Single image + visibility classes | Tailwind v2+ (2020) | Simpler asset management, single source of truth |
| <picture> for all responsive images | Tailwind breakpoints for visibility | Common since 2020 | Reserve <picture> for true art direction (different crops/compositions) |
| Client Components by default | Server Components by default | Next.js 13+ App Router (2022) | Smaller client bundles, faster initial load |
| priority prop | preload prop (v16+) | Next.js 16.0.0 (Jan 2025) | `priority` deprecated but still works, `preload` more explicit |

**Deprecated/outdated:**
- **Image `priority` prop:** Deprecated in Next.js 16 in favor of `preload` for clarity, but still functional in 16.1.6
- **position: fixed for headers:** Sticky is standard practice, fixed causes layout issues
- **Manual z-index management:** Use design system scale (Tailwind defaults: 0/10/20/30/40/50)

## Open Questions

1. **Should header background be translucent (backdrop-blur) or opaque?**
   - What we know: Current footer uses opaque `bg-surface-primary`
   - What's unclear: Design preference for glassmorphism vs solid
   - Recommendation: Start with opaque solid background matching footer pattern; glassmorphism can be added in future phase if desired

2. **Should "Scan Now" CTA on non-homepage pages link to / or /#scan-form?**
   - What we know: Homepage has scan form, results/scan pages don't
   - What's unclear: User expectation when clicking "Scan Now" from results page
   - Recommendation: Always link to `/#scan-form` (scrolls on homepage, navigates then scrolls on other pages)

3. **Should nav links include Privacy/Terms in header or only in footer?**
   - What we know: Footer has Privacy/Terms links already
   - What's unclear: HDR-01 requirement says "logo and CTA", doesn't mention nav links
   - Recommendation: Header shows only logo + CTA for simplicity; Privacy/Terms remain footer-only (matches common pattern)

## Sources

### Primary (HIGH confidence)
- [Next.js 16 Image Component Docs](https://nextjs.org/docs/app/api-reference/components/image) - Image props, priority, responsive patterns
- [Next.js 16 Link Component Docs](https://nextjs.org/docs/app/api-reference/components/link) - Link props, prefetch, hash navigation
- [Next.js Server and Client Components](https://nextjs.org/docs/app/getting-started/server-and-client-components) - When to use "use client"
- [Tailwind CSS Position Docs](https://tailwindcss.com/docs/position) - Sticky positioning utilities
- [Tailwind CSS Responsive Design](https://tailwindcss.com/docs/responsive-design) - Breakpoint system (sm: 640px, md: 768px)

### Secondary (MEDIUM confidence)
- [TPG: Prevent focused elements from being obscured by sticky headers](https://www.tpgi.com/prevent-focused-elements-from-being-obscured-by-sticky-headers/) - WCAG 2.2 SC 2.4.11 compliance
- [Medium: Building Sticky Header and Footer with Tailwind CSS 4](https://medium.com/@sureshdotariya/building-sticky-header-and-footer-components-with-tailwind-css-4-utilities-e6bc0527c358) - Tailwind v4 patterns
- [BOIA: How Sticky and Fixed Elements Impact Accessibility](https://www.boia.org/blog/how-sticky-and-fixed-elements-impact-accessibility) - Accessibility considerations
- [MDN: Understanding z-index](https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Positioned_layout/Understanding_z-index) - Stacking contexts
- [Josh Comeau: What The Heck, z-index??](https://www.joshwcomeau.com/css/stacking-contexts/) - Stacking context gotchas

### Tertiary (LOW confidence)
- Various blog posts on sticky navigation patterns - general guidance only, verified against official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in package.json, official Next.js docs verified
- Architecture: HIGH - Patterns verified in Next.js 16 docs + existing project layout.tsx
- Pitfalls: HIGH - WCAG 2.2 guidance official, z-index issues well-documented, Image priority in official docs

**Research date:** 2026-02-11
**Valid until:** 2026-03-11 (30 days - stable domain, Next.js 16 unlikely to change core patterns)
