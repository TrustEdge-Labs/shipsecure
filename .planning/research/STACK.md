# Technology Stack — Brand Identity Features

**Project:** ShipSecure
**Domain:** Brand identity (logo, favicon, color system, icons)
**Researched:** 2026-02-09
**Confidence:** HIGH

## Context

This research covers **brand identity stack additions only** for an existing Next.js 16 + Tailwind CSS v4 application. The application already has validated Next.js 16, Tailwind CSS v4, dark mode, and component architecture. Focus is on what's needed to add:

- SVG logo/wordmark rendering
- Favicon generation (all sizes/formats)
- Design token system for colors
- SVG icon component library

**Key constraint:** Zero-dependency preference. Use native Next.js and Tailwind features where possible.

---

## Recommended Stack

### SVG Logo/Wordmark

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Native SVG in React** | N/A | Logo rendering | Zero dependencies, full TypeScript support, optimal tree-shaking, React Server Components compatible |
| **SVGO** | CLI tool | SVG optimization | Industry standard optimizer, removes unnecessary metadata, reduces file size 40-60% |

**Why not libraries:**
- `react-svg-loader`: Unnecessary webpack config complexity in Next.js 16 App Router
- `@svgr/webpack`: Built-in Next.js handles SVG imports natively
- `next-svg`: Deprecated, native support is better

### Favicon Generation

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Next.js Metadata API** | Built-in (16.x) | Favicon management | Native support for all formats, automatic generation from single source, built into App Router |
| **Sharp** | Latest (auto-installed) | Image processing | **Already a Next.js dependency**, handles PNG generation for Apple Touch icons |

**File-based convention:**
```
app/
  favicon.ico           -> Browser favicon (32x32)
  icon.png             -> Auto-generates multiple sizes
  apple-icon.png       -> iOS home screen icon
```

### Design Token System

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Tailwind CSS v4 `@theme`** | Built-in (4.x) | CSS custom properties | Native design token system, CSS Variables under the hood, full IntelliSense support |
| **`globals.css`** | Existing file | Token definition source | Already in use, no new files needed |

**Pattern:**
```css
@import "tailwindcss";

@theme {
  --color-brand-primary: #3b82f6;
  --color-brand-secondary: #1e40af;
}
```

**Why not alternatives:**
- Style Dictionary: Overkill for single platform, adds build complexity
- Vanilla Extract: Requires CSS-in-JS setup, conflicts with Tailwind
- Panda CSS: Alternative to Tailwind, not additive

### SVG Icon Components

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **Lucide React** | ^0.468.0 | Icon component library | 1400+ icons, tree-shakeable, TypeScript-native, consistent 24px design system, actively maintained |

**Alternative comparison:**

| Library | Bundle Size | Tree-Shake | Tailwind Native | Server Components | Verdict |
|---------|-------------|------------|-----------------|-------------------|---------|
| **Lucide** | 1-2KB/icon | Yes | Yes | Yes | Recommended |
| Heroicons | 1-2KB/icon | Yes | Yes | Yes | Also excellent |
| React Icons | 10-20KB/icon | No | No | Partial | Too heavy |
| Font Awesome | 50KB+ base | No | No | No | Legacy approach |

---

## What NOT to Add

| Library | Why Avoid |
|---------|-----------|
| `@svgr/webpack` | Next.js 16 handles SVG imports natively |
| `react-inlinesvg` | Unnecessary abstraction, use native `<svg>` |
| `tailwindcss-themer` | Tailwind v4 `@theme` replaces this |
| `styled-components` / `emotion` | Already using Tailwind |
| `react-icons` | Bundle bloat, poor tree-shaking |
| Font Awesome | Legacy CSS-based approach, heavy bundle |

---

## Installation

```bash
cd frontend

# Icon library (ONLY new dependency)
npm install lucide-react@^0.468.0
```

**That's it.** Everything else is already available:
- SVG handling: Native Next.js
- Favicon generation: Built-in Metadata API
- Design tokens: Tailwind CSS v4 `@theme`
- Image optimization: Sharp (already a Next.js dep)

---

## Integration Patterns

### 1. SVG Logo Component

```tsx
// components/brand/logo.tsx
export function Logo({ className = '' }: { className?: string }) {
  return (
    <svg viewBox="0 0 200 48" className={className} aria-label="ShipSecure">
      {/* Inline SVG path data */}
    </svg>
  )
}
```

**Why inline:** Eliminates HTTP request, enables dynamic `className`, tree-shakes unused variants, works with Server Components.

### 2. Favicon Setup

```typescript
// app/icon.tsx for dynamic generation
import { ImageResponse } from 'next/og'

export const size = { width: 512, height: 512 }
export const contentType = 'image/png'

export default function Icon() {
  return new ImageResponse(
    <div style={{ /* SVG-based icon */ }}>...</div>,
    { ...size }
  )
}
```

### 3. Design Token System

```css
@import "tailwindcss";

@theme {
  --color-brand-blue-50: #eff6ff;
  --color-brand-blue-500: #3b82f6;
  --color-brand-blue-600: #2563eb;
  --color-brand-blue-900: #1e3a8a;
  --color-primary: var(--color-brand-blue-600);
  --color-primary-hover: var(--color-brand-blue-700);
}
```

### 4. Icon Component Pattern

```tsx
import { Shield, CheckCircle, AlertTriangle } from 'lucide-react'

<Shield className="w-5 h-5 text-brand-blue-600" />
```

---

## Performance Considerations

| Addition | Bundle Impact | Notes |
|----------|---------------|-------|
| SVG logo (inline) | ~1-2KB | One-time cost |
| Lucide icons | ~1-2KB per icon | Tree-shakeable |
| Design tokens | ~0.5KB | CSS variables |
| Favicon files | 0KB (static) | Not in JS bundle |

**Total estimated impact:** ~5-10KB for typical usage (5-8 icons).

---

## Sources

- Next.js Metadata API: https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons
- Tailwind CSS v4 Theme: https://tailwindcss.com/docs/theme
- Lucide Icons: https://lucide.dev
- SVGO: https://github.com/svg/svgo
