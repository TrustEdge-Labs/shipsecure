# Architecture Research: Brand Identity Integration

**Domain:** Brand Identity for Next.js 16 + Tailwind CSS v4 Application
**Researched:** 2026-02-09
**Confidence:** HIGH (official docs verified)

## System Architecture

```
frontend/
  app/
    layout.tsx           <- MODIFY: Add <Header />, update viewport metadata
    globals.css          <- MODIFY: Add @theme design tokens (additive)
    icon.tsx             <- NEW: Dynamic favicon via ImageResponse
    apple-icon.png       <- NEW: Static 180x180 Apple icon
    favicon.ico          <- NEW: 32x32 legacy favicon
    page.tsx             <- MODIFY: Replace emoji icons, use design tokens
  components/
    brand/
      logo.tsx           <- NEW: SVG logo with variants (full/icon/wordmark)
      header.tsx         <- NEW: Sticky navbar with Logo, nav, CTA
    icons/
      index.ts           <- NEW: Barrel export for icon components
  public/
    logo.svg             <- NEW: Optimized source SVG
```

## Component Architecture

### 1. Design Token System (Tailwind v4 @theme)

```css
@theme {
  /* Brand colors */
  --color-brand-primary: oklch(0.5 0.2 250);
  --color-brand-primary-hover: oklch(0.45 0.2 250);

  /* Semantic tokens */
  --color-surface-primary: var(--background);
  --color-surface-secondary: #f9fafb;
  --color-text-primary: var(--foreground);
  --color-text-secondary: #6b7280;
  --color-border-subtle: #e5e7eb;
  --color-border-default: #d1d5db;

  /* Severity (preserve existing visual language) */
  --color-severity-critical: #991b1b;
  --color-severity-critical-bg: #fef2f2;
  --color-success: #16a34a;
  --color-danger: #dc2626;
  --color-warning: #ca8a04;
}

@media (prefers-color-scheme: dark) {
  @theme {
    --color-brand-primary: oklch(0.6 0.2 250);
    --color-surface-secondary: #1f2937;
    --color-text-secondary: #9ca3af;
    --color-border-subtle: #374151;
    --color-border-default: #4b5563;
    --color-severity-critical: #fca5a5;
    --color-severity-critical-bg: #7f1d1d;
    --color-success: #4ade80;
    --color-danger: #f87171;
  }
}
```

**Naming convention:** `--color-{category}-{variant}-{state}`
Categories: brand, surface, text, border, success, warning, danger, info

### 2. Logo Component

```tsx
// components/brand/logo.tsx
interface LogoProps {
  variant?: 'full' | 'icon' | 'wordmark'
  size?: 'sm' | 'md' | 'lg' | 'xl'
  className?: string
}

export function Logo({ variant = 'full', size = 'md', className }: LogoProps) {
  return (
    <svg className={className} fill="currentColor" aria-label="ShipSecure" role="img">
      {/* SVG paths - uses currentColor for theme support */}
    </svg>
  )
}
```

**Key decisions:**
- Use `currentColor` for fills (inherits text color, themeable)
- Provide variant/size props for reuse across contexts
- Always include `aria-label` and `role="img"`
- Never use `<img>` tag (not themeable, extra HTTP request)

### 3. Header/Navbar

```tsx
// components/brand/header.tsx
<header className="sticky top-0 z-[1020] border-b border-border-subtle bg-surface-primary">
  <div className="container mx-auto px-4 h-16 flex items-center justify-between">
    <Link href="/"><Logo variant="full" size="md" /></Link>
    <nav className="hidden md:flex gap-6">...</nav>
    <Button className="bg-brand-primary">Scan Now</Button>
  </div>
</header>
```

**Key decisions:**
- Fixed height (h-16 = 64px)
- Sticky positioning with z-index from defined scale
- Logo on left, nav center, CTA right
- Hide nav on mobile (`hidden md:flex`)
- Define `--header-height: 64px` CSS variable for spacing calculations

### 4. Icon System

```tsx
// Pattern: Use Lucide icons with consistent sizing
import { Shield, Lock, FileText, Search } from 'lucide-react'

// Decorative (next to text)
<Shield className="w-5 h-5" aria-hidden="true" />

// Standalone (needs label)
<Shield className="w-5 h-5" aria-label="Security shield" />
```

### 5. Favicon Configuration

```tsx
// app/icon.tsx - Dynamic generation from logo SVG
import { ImageResponse } from 'next/og'

export const size = { width: 32, height: 32 }
export const contentType = 'image/png'

export default function Icon() {
  return new ImageResponse(/* Logo SVG at small size */)
}
```

Dark mode favicon support via SVG:
```svg
<svg xmlns="http://www.w3.org/2000/svg">
  <style>
    path { fill: #2563eb; }
    @media (prefers-color-scheme: dark) {
      path { fill: #60a5fa; }
    }
  </style>
  <path d="..." />
</svg>
```

## Color Migration Strategy

**Incremental approach:** Add tokens WITHOUT removing existing classes first.

| Current Class | New Token | Usage |
|--------------|-----------|-------|
| `blue-600` | `brand-primary` | CTA buttons, primary links |
| `blue-700` | `brand-primary-hover` | Hover states |
| `gray-900` | `text-primary` | Headings |
| `gray-500` | `text-secondary` | Body text, muted |
| `gray-50` | `surface-secondary` | Light backgrounds |
| `gray-200` | `border-subtle` | Borders |
| `red-600` | `danger` | Error states |
| `green-600` | `success` | Success states |

**Migration order:**
1. Add @theme tokens (non-breaking)
2. New components use tokens exclusively
3. Migrate existing components one-by-one
4. Remove legacy classes after full verification

## Integration Points

### NEW Components
| Component | Purpose | Dependencies |
|-----------|---------|-------------|
| Logo | SVG wordmark/icon | React only |
| Header | Navigation bar | Logo, Next.js Link, usePathname |
| Favicon | Dynamic generation | next/og ImageResponse |

### MODIFIED Components
| Component | Changes | Risk |
|-----------|---------|------|
| layout.tsx | Add `<Header />`, update metadata | LOW |
| globals.css | Add `@theme` tokens | LOW (additive) |
| page.tsx | Replace emoji icons | MEDIUM |
| scan-form.tsx | Color token migration | MEDIUM |
| footer.tsx | Color token migration | LOW |

### NO CHANGES
- Backend API, Database, Server actions, Nginx config

## Suggested Build Order

1. **Design tokens in globals.css** (30 min) — Non-breaking, foundation
2. **Logo component** (1 hr) — SVG with variants and sizes
3. **Icon components / install Lucide** (1 hr) — Icon system setup
4. **Header component** (1.5 hr) — Build with logo, test in isolation
5. **Favicon generation** (30 min) — icon.tsx + apple-icon + favicon.ico
6. **Integrate header into layout** (30 min) — Add to layout.tsx, adjust spacing
7. **Migrate landing page** (1 hr) — Replace emojis, use tokens
8. **Migrate scan-form** (45 min) — Color tokens
9. **Migrate footer** (30 min) — Color tokens
10. **Migrate grade-summary** (1 hr) — Severity color tokens
11. **Cross-browser testing** (1 hr) — Chrome, Firefox, Safari
12. **Accessibility audit** (1 hr) — Contrast, aria-labels, keyboard nav

**Total: ~10-12 hours**

## Anti-Patterns to Avoid

1. **Mixing design systems** — Don't use `bg-brand-primary border-gray-300` (inconsistent)
2. **Logo as `<img>` tag** — Not themeable, extra request
3. **Inline SVG in JSX** — Duplicates code, use components
4. **Hex colors in @theme** — Use OKLch for perceptual uniformity
5. **Over-specific tokens** — `--color-scan-form-button-bg` is too specific
6. **Skipping a11y attributes** — Always include aria-label on SVGs

## Z-Index Scale

```css
@theme {
  --z-base: 0;
  --z-dropdown: 1000;
  --z-sticky: 1020;
  --z-fixed: 1030;
  --z-modal-backdrop: 1040;
  --z-modal: 1050;
  --z-tooltip: 1070;
}
```

## Sources

- Next.js 16 Metadata API: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons
- Tailwind CSS v4 Theme: https://tailwindcss.com/docs/theme
- OKLch Color Space: https://oklch.com
- WCAG 2.2 Contrast: https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum
