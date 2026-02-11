# Phase 18: Favicon & OG Image - Research

**Researched:** 2026-02-11
**Domain:** Next.js metadata API, favicon generation, Open Graph images
**Confidence:** HIGH

## Summary

Phase 18 implements branded favicon assets and updates the Open Graph image to include the logo. The project already has a professionally designed PNG logo (1536x1024, 2.1MB) at `/frontend/public/logo.png` and a basic favicon.ico in `/frontend/app/`. Next.js 16.1.6 provides a file-based Metadata API that automatically generates appropriate `<head>` tags for favicons and OG images.

**Modern 2026 best practice** recommends a minimal favicon set (5-6 files) rather than dozens of variants, leveraging SVG for scalability and browser downscaling. For favicons, the essential files are: `favicon.ico` (32x32 for legacy browsers), `icon.svg` (with dark mode support), `apple-icon.png` (180x180), and optionally PWA icons (192x192, 512x512). For Open Graph, Next.js ImageResponse API enables programmatic generation of OG images with custom branding, compositing the PNG logo onto branded backgrounds.

**Primary recommendation:** Use Next.js file-based metadata API with minimal favicon set (favicon.ico, icon.svg with dark mode, apple-icon.png). Generate favicons from source PNG using build-time Node.js scripts. Create dynamic OG image using ImageResponse API compositing the logo.png onto branded background with design tokens.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Next.js Metadata API | 16.1.6 | File-based favicon/OG config | Built-in to Next.js App Router, zero dependencies |
| next/og ImageResponse | 16.1.6 | Programmatic OG image generation | Official Next.js API, uses Satori rendering |
| sharp | latest | PNG processing for favicons | Industry standard Node.js image processing, zero native dependencies |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| to-ico | 2.x | PNG to ICO conversion | Build-time script to generate favicon.ico |
| @vercel/satori | latest | HTML/CSS to PNG rendering | Used internally by ImageResponse |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| File-based metadata | Manual `<link>` tags in layout.tsx | File-based is simpler, auto-generates correct attributes |
| Sharp | ImageMagick CLI | Sharp is pure JS, no system dependencies, better for CI/CD |
| to-ico npm | png-to-ico npm | Both similar; to-ico has simpler API and better TypeScript support |
| Dynamic OG | Static opengraph-image.png | Dynamic allows brand consistency via design tokens |
| SVG vectorization | Use PNG favicons only | SVG supports dark mode, smaller file size, but requires manual creation |

**Installation:**
```bash
npm install --save-dev sharp to-ico
```

## Architecture Patterns

### Recommended Project Structure
```
frontend/
├── app/
│   ├── favicon.ico              # 32x32 multi-resolution ICO (auto-detected by Next.js)
│   ├── icon.svg                 # SVG with dark mode support (auto-detected)
│   ├── apple-icon.png           # 180x180 Apple touch icon (auto-detected)
│   ├── opengraph-image.tsx      # Dynamic OG image generator (auto-detected)
│   └── layout.tsx               # Metadata exports (title, description, etc.)
├── public/
│   └── logo.png                 # Source logo (existing)
└── scripts/
    └── generate-favicons.ts     # Build-time favicon generation from logo.png
```

### Pattern 1: File-Based Favicon Configuration

**What:** Next.js automatically detects special filenames (`favicon.ico`, `icon.svg`, `apple-icon.png`) in `/app` directory and generates appropriate `<link>` tags.

**When to use:** Always for Next.js App Router projects (v13.3+)

**Example:**
```typescript
// No code needed in layout.tsx - Next.js auto-detects files
// Just place these files in /app directory:
//   - favicon.ico (legacy browser support)
//   - icon.svg (modern browsers with dark mode)
//   - apple-icon.png (iOS home screen)
```

**Generated output:**
```html
<!-- Next.js automatically generates: -->
<link rel="icon" href="/favicon.ico" sizes="any" />
<link rel="icon" href="/icon.svg" type="image/svg+xml" />
<link rel="apple-touch-icon" href="/apple-icon.png" />
```

**Source:** [Next.js Metadata API - App Icons](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons)

### Pattern 2: SVG Favicon with Dark Mode Support

**What:** SVG favicons can adapt to light/dark mode using CSS `@media (prefers-color-scheme: dark)` inside the SVG file.

**When to use:** When you have a simple logo that can be vectorized or manually created as SVG.

**Example:**
```xml
<!-- app/icon.svg -->
<svg width="32" height="32" viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
  <style>
    .shield { fill: #2563eb; } /* blue-600 for light mode */
    @media (prefers-color-scheme: dark) {
      .shield { fill: #3b82f6; } /* blue-500 for dark mode */
    }
  </style>
  <path class="shield" d="M16 2 L28 8 L28 18 Q28 28 16 30 Q4 28 4 18 L4 8 Z"/>
</svg>
```

**Browser support:** 95-97% of browsers support SVG favicons (Chrome 80+, Firefox 41+, Edge 80+, Safari 26+). Falls back to favicon.ico for older browsers.

**Source:** [SVG Favicon Dark Mode - CodyHouse](https://codyhouse.co/nuggets/svg-favicon-dark-mode), [Evil Martians Favicon Guide](https://evilmartians.com/chronicles/how-to-favicon-in-2021-six-files-that-fit-most-needs)

### Pattern 3: Dynamic OG Image with Logo Composite

**What:** Generate Open Graph images programmatically using `next/og` ImageResponse API, compositing the PNG logo onto a branded background.

**When to use:** When you want to maintain brand consistency using design tokens and include your logo in social shares.

**Example:**
```typescript
// app/opengraph-image.tsx
import { ImageResponse } from 'next/og'
import { readFile } from 'node:fs/promises'
import { join } from 'node:path'

export const alt = 'ShipSecure - Security Scanning for Vibe-Coded Apps'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
  // Load logo as base64 data URI
  const logoData = await readFile(
    join(process.cwd(), 'public/logo.png'),
    'base64'
  )
  const logoSrc = `data:image/png;base64,${logoData}`

  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: '#0f172a', // slate-900 from design tokens
        }}
      >
        <img src={logoSrc} height="400" alt="Logo" />
      </div>
    ),
    { ...size }
  )
}
```

**Source:** [Next.js opengraph-image](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/opengraph-image)

### Pattern 4: Build-Time Favicon Generation Script

**What:** Node.js script that generates multiple favicon sizes from source PNG logo, called during build process.

**When to use:** When you have a PNG source logo and need to generate ICO and resized PNG files.

**Example:**
```typescript
// scripts/generate-favicons.ts
import sharp from 'sharp'
import { writeFile } from 'node:fs/promises'
import toIco from 'to-ico'

async function generateFavicons() {
  const input = 'public/logo.png'

  // Generate 32x32 PNG for ICO
  const png32 = await sharp(input)
    .resize(32, 32, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer()

  // Generate 16x16 PNG for ICO
  const png16 = await sharp(input)
    .resize(16, 16, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer()

  // Create multi-resolution ICO
  const ico = await toIco([png32, png16])
  await writeFile('app/favicon.ico', ico)

  // Generate Apple touch icon (180x180)
  await sharp(input)
    .resize(180, 180, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile('app/apple-icon.png')
}

generateFavicons().catch(console.error)
```

**Source:** Multiple npm packages and community patterns

### Anti-Patterns to Avoid

- **Dozens of favicon sizes:** Modern browsers downscale effectively. Don't create 10+ sizes (16, 24, 32, 48, 64, 96, etc.). Use 3-5 files maximum.
- **Favicon in `public/`:** Next.js App Router expects favicons in `/app` directory, not `/public`.
- **Manual `<link>` tags:** Don't manually add favicon links in `layout.tsx` - use file-based convention for auto-generation.
- **OG images exceeding 8MB:** Facebook rejects OG images > 8MB (Twitter limit is 5MB). Keep under 150KB for best performance.
- **Missing alt text:** Always export `alt` text for OG images for accessibility.
- **Grid layouts in ImageResponse:** Only flexbox works. Don't use `display: grid`.
- **Complex SVG favicons at 16x16:** Fine details turn to "mud" at small sizes. Keep favicon design simple.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| PNG to ICO conversion | Custom binary ICO encoder | `to-ico` npm package | ICO format is complex with headers, BITMAPINFOHEADER structures, color palettes |
| Image resizing | Canvas API or custom scaling | `sharp` library | Handles color spaces, alpha channels, resampling algorithms correctly |
| OG image generation | Puppeteer screenshots | Next.js `ImageResponse` | Faster, smaller bundle, built-in to Next.js |
| SVG vectorization | Manual path tracing | AI vectorization tools (Vectorizer.AI, Vector Magic) | Automatic tracing is faster and more accurate than manual |
| Dark mode detection in OG | Custom URL params | CSS `prefers-color-scheme` in SVG | Standard, browser-native, no JS required |

**Key insight:** Favicon and OG image generation involves subtle edge cases (color spaces, alpha blending, browser quirks, file format specs) that are better handled by battle-tested libraries. The 500+ stars and years of production use for libraries like `sharp` and `to-ico` represent thousands of hours debugging edge cases you don't want to re-discover.

## Common Pitfalls

### Pitfall 1: SVG Favicon Doesn't Update After Theme Change
**What goes wrong:** User changes OS theme from light to dark, but favicon doesn't update until page refresh.
**Why it happens:** Browsers cache favicons aggressively and don't re-evaluate CSS media queries without refresh.
**How to avoid:** Document this behavior; it's browser limitation, not fixable. All modern browsers require page refresh for favicon theme changes.
**Warning signs:** User reports "favicon doesn't match my dark mode."

### Pitfall 2: ImageResponse Bundle Size Exceeds 500KB
**What goes wrong:** Build fails with "ImageResponse bundle size exceeded 500KB" error.
**Why it happens:** Including large fonts, base64-encoded images, or excessive inline assets.
**How to avoid:**
- Load fonts from file system, not inline
- Use external URLs for images when possible
- Compress fonts (prefer TTF/OTF over WOFF)
- Minimize custom fonts (1-2 fonts max)
**Warning signs:** Build warnings about bundle size approaching limit.

### Pitfall 3: Favicon.ico Appears Blurry on Retina Displays
**What goes wrong:** Favicon looks pixelated on high-DPI screens.
**Why it happens:** ICO contains only 16x16 and 32x32 sizes; browser upscales for Retina.
**How to avoid:** This is expected for ICO format. Modern browsers use `icon.svg` on Retina displays, which scales perfectly. ICO is legacy fallback only.
**Warning signs:** User reports "favicon looks blurry" on Mac.

### Pitfall 4: OG Image Shows Broken Image on Social Media
**What goes wrong:** Social media shows blank or broken image when sharing.
**Why it happens:**
- Image exceeds size limit (8MB for Facebook, 5MB for Twitter)
- Server timeout during ImageResponse generation
- Incorrect `contentType` export
- Image URL not publicly accessible (localhost, auth-protected)
**How to avoid:**
- Keep images under 150KB
- Use static generation (`export const dynamic = 'force-static'`)
- Test with Facebook Debugger and Twitter Card Validator
- Ensure `metadataBase` is set in `layout.tsx`
**Warning signs:** Social media preview shows blank square or default screenshot.

### Pitfall 5: Apple Touch Icon Shows White Background on Dark Mode
**What goes wrong:** iOS home screen icon has white background even in dark mode.
**Why it happens:** PNG apple-icon doesn't support dark mode (unlike SVG). iOS expects opaque icon.
**How to avoid:** Design apple-icon with transparent or neutral background that works in both modes, or accept light-mode design for iOS.
**Warning signs:** User reports "home screen icon looks wrong in dark mode."

### Pitfall 6: Favicon Not Appearing After Deployment
**What goes wrong:** Favicon works locally but not in production.
**Why it happens:**
- Files not included in build output
- Incorrect file naming (case sensitivity on Linux servers)
- CDN/cache not cleared after update
- Files placed in wrong directory (`public/` instead of `app/`)
**How to avoid:**
- Verify files exist in `.next/static/` or `.next/server/app/` after build
- Use lowercase filenames consistently
- Set cache headers appropriately
- Place favicon files in `app/` directory for App Router
**Warning signs:** Works in `npm run dev` but not in production build.

## Code Examples

Verified patterns from official sources:

### Minimal Favicon Set (File-Based)
```bash
# Required structure (Next.js auto-detects)
app/
├── favicon.ico          # 32x32 + 16x16 multi-resolution ICO
├── icon.svg             # Scalable with dark mode support
└── apple-icon.png       # 180x180 for iOS home screen
```

**Source:** [Next.js App Icons](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons)

### Generate Multi-Resolution favicon.ico
```typescript
// scripts/generate-favicons.ts
import sharp from 'sharp'
import toIco from 'to-ico'
import { writeFile } from 'node:fs/promises'

async function generateFavicon() {
  // Generate 16x16 and 32x32 from source logo
  const sizes = [16, 32]
  const buffers = await Promise.all(
    sizes.map(size =>
      sharp('public/logo.png')
        .resize(size, size, {
          fit: 'contain',
          background: { r: 0, g: 0, b: 0, alpha: 0 }
        })
        .png()
        .toBuffer()
    )
  )

  // Create multi-resolution ICO
  const ico = await toIco(buffers)
  await writeFile('app/favicon.ico', ico)
}
```

**Source:** [to-ico npm](https://www.npmjs.com/package/to-ico), [sharp docs](https://sharp.pixelplumbing.com/)

### Dark Mode SVG Favicon
```xml
<!-- app/icon.svg -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <style>
    .logo { fill: #2563eb; }
    @media (prefers-color-scheme: dark) {
      .logo { fill: #3b82f6; }
    }
  </style>
  <!-- Simple shield shape - complex details won't render at 16x16 -->
  <path class="logo" d="M16 2 L28 8 L28 18 Q28 28 16 30 Q4 28 4 18 L4 8 Z"/>
</svg>
```

**Source:** [CodyHouse SVG Favicon Dark Mode](https://codyhouse.co/nuggets/svg-favicon-dark-mode)

### OG Image with Logo Composite
```typescript
// app/opengraph-image.tsx
import { ImageResponse } from 'next/og'
import { readFile } from 'node:fs/promises'
import { join } from 'node:path'

export const alt = 'ShipSecure - Security Scanning for Vibe-Coded Apps'
export const size = { width: 1200, height: 630 }
export const contentType = 'image/png'

export default async function Image() {
  // Load logo as base64 (ImageResponse supports data URIs)
  const logoBuffer = await readFile(join(process.cwd(), 'public/logo.png'))
  const logoBase64 = logoBuffer.toString('base64')
  const logoSrc = `data:image/png;base64,${logoBase64}`

  return new ImageResponse(
    (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: '#0f172a', // Design token: slate-900
          padding: '80px',
        }}
      >
        {/* Logo (will be resized to fit) */}
        <img
          src={logoSrc}
          alt="Logo"
          width="500"
          height="333"
          style={{ marginBottom: '40px' }}
        />
        {/* Optional: Add text overlay */}
        <div
          style={{
            fontSize: 48,
            fontWeight: 'bold',
            color: '#ffffff',
            textAlign: 'center',
          }}
        >
          Security Scanning for Vibe-Coded Apps
        </div>
      </div>
    ),
    { ...size }
  )
}
```

**Source:** [Next.js opengraph-image](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/opengraph-image), [ImageResponse API](https://nextjs.org/docs/app/api-reference/functions/image-response)

### Layout Metadata Configuration
```typescript
// app/layout.tsx
import type { Metadata } from 'next'

export const metadata: Metadata = {
  metadataBase: new URL('https://shipsecure.ai'),
  title: 'ShipSecure - Security Scanning for Vibe-Coded Apps',
  description: 'Ship fast, stay safe. Free security scanning for AI-generated web applications.',
  // Favicon and OG images auto-detected from files, no explicit config needed
}
```

**Source:** [Next.js Metadata API](https://nextjs.org/docs/app/api-reference/functions/generate-metadata)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 20+ favicon sizes | 3-5 files (ICO + SVG + Apple) | 2021-2023 | Modern browsers downscale effectively, reduced complexity |
| Manual `<link>` tags | File-based metadata API | Next.js 13.3 (2023) | Auto-generation, type safety, simplified config |
| Static OG images | Dynamic ImageResponse | Next.js 13.0 (2022) | Programmatic generation, design token integration |
| Puppeteer screenshots | Satori HTML/CSS rendering | 2022 | Faster, smaller bundle, no browser headless |
| PNG favicons only | SVG with dark mode | 2020-2021 | Native theme adaptation without JS |
| Web App Manifest for all icons | Minimal manifest + file-based | 2021 | Browsers auto-detect from file conventions |

**Deprecated/outdated:**
- **browserconfig.xml:** Windows 8 tile config - no longer needed, Microsoft dropped support
- **Multiple manifest files:** Use single `site.webmanifest` for PWA only
- **Favicon in `public/`:** Next.js App Router expects `/app` directory
- **`<meta name="msapplication-*">`:** Windows Phone dead, IE11 end-of-life
- **Dozens of PNG sizes:** Use SVG + browser downscaling instead

## Open Questions

1. **Should we create an SVG version of the shield logo?**
   - What we know: Current logo is professionally designed PNG (1536x1024). SVG favicons support dark mode and are smaller.
   - What's unclear: Whether the multi-color shield design can be effectively vectorized, or if we should create a simplified SVG variant.
   - Recommendation: Assess complexity of vectorization in Phase 14 context. If logo is simple shield + text, manually create SVG. If complex gradients/effects, use PNG favicons only (simpler, still works).

2. **Should we extract shield mark separately from wordmark?**
   - What we know: Favicons at 16x16 don't render text well. Best practice is simple iconic mark.
   - What's unclear: Whether the current logo.png contains separable shield mark, or if it's combined design.
   - Recommendation: In planning, decide whether to crop logo.png to just shield portion for favicon, or use full logo scaled down.

3. **Should OG image be static or dynamic?**
   - What we know: ImageResponse enables dynamic generation with design tokens. Static files are simpler and faster.
   - What's unclear: Whether we need different OG images per route, or single site-wide image.
   - Recommendation: Start with dynamic site-wide OG image using ImageResponse for brand consistency. Can optimize to static if no dynamic needs emerge.

## Sources

### Primary (HIGH confidence)
- [Next.js Metadata API - App Icons](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/app-icons) - File-based favicon conventions
- [Next.js opengraph-image](https://nextjs.org/docs/app/api-reference/file-conventions/metadata/opengraph-image) - OG image generation
- [Next.js ImageResponse API](https://nextjs.org/docs/app/api-reference/functions/image-response) - Dynamic image generation
- [Sharp Documentation](https://sharp.pixelplumbing.com/) - Image processing library
- [to-ico npm package](https://www.npmjs.com/package/to-ico) - PNG to ICO conversion

### Secondary (MEDIUM confidence)
- [Evil Martians Favicon Guide](https://evilmartians.com/chronicles/how-to-favicon-in-2021-six-files-that-fit-most-needs) - Modern minimal favicon strategy
- [CodyHouse SVG Favicon Dark Mode](https://codyhouse.co/nuggets/svg-favicon-dark-mode) - Dark mode SVG implementation
- [TheLinuxCode Favicon Guide](https://thelinuxcode.com/what-is-a-favicon-and-what-sizes-should-you-use-in-html-2026-guide/) - 2026 size recommendations
- [Favicon.im Blog](https://favicon.im/blog/add-favicon-to-nextjs-project) - Next.js implementation guide
- [Oreate AI OG Image Guide](https://www.oreateai.com/blog/understanding-open-graph-image-dimensions-a-guide-for-social-media-success/773a7b918df428fd0b74bac43c0cd21e) - OG dimensions best practices

### Tertiary (LOW confidence)
- [Vectorizer.AI](https://vectorizer.ai/) - PNG to SVG conversion tool (for reference)
- [WebSearch: Various favicon generators](https://realfavicongenerator.net/) - Alternative generation tools (not recommended for Next.js)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Next.js APIs, well-documented npm packages
- Architecture: HIGH - Official Next.js patterns, verified with documentation
- Pitfalls: MEDIUM - Based on community reports and common issues, some inferred from experience
- SVG vectorization: LOW - No direct experience with this project's specific logo

**Research date:** 2026-02-11
**Valid until:** 2026-03-11 (30 days - stable domain)

**Key constraints discovered:**
- Existing logo is PNG (1536x1024), not SVG
- Existing favicon.ico already exists in `/app` directory (may need replacement)
- ImageResponse has 500KB bundle size limit
- OG images must be < 8MB (Facebook) / 5MB (Twitter)
- SVG favicons require page refresh to update theme
