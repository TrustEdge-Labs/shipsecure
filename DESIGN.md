# Design System -- ShipSecure

## Product Context
- **What this is:** Security scanning SaaS for vibe-coded apps. Scans websites for headers, TLS, exposed files, JS secrets, and vibe-code vulnerabilities.
- **Who it's for:** Solo developers and small teams shipping with AI code generators (Cursor, Bolt, Lovable, v0). No security expertise required.
- **Space/industry:** Developer security tools. Peers: Snyk, Aikido, Wiz (enterprise). Linear, Vercel (dev tool aesthetic).
- **Project type:** Hybrid (marketing landing page + app UI for scan results and dashboard).

## Aesthetic Direction
- **Direction:** Industrial/Utilitarian. Linear meets a security terminal.
- **Decoration level:** Minimal. Typography and color do the work. No gradients, no blobs, no decorative elements.
- **Mood:** Professional but approachable. Feels like a tool a developer built for themselves. Not enterprise. Not scary. Fast, dark, trustworthy.
- **Reference sites:** linear.app (aesthetic baseline), vercel.com (typography), aikido.dev (security product balance)

## Typography
- **Display/Hero:** Geist 700 -- clean, geometric, native to the Next.js/Vercel ecosystem the audience lives in. 48px, letter-spacing -0.02em.
- **Body:** Geist 400 -- same family for consistency. 16px, line-height 1.6.
- **UI/Labels:** Geist 600 -- 12px uppercase with 0.5px letter-spacing for section labels.
- **Data/Tables:** Geist Mono 400 -- tabular-nums for scan timestamps, severity counts, and technical output. 14px.
- **Code:** Geist Mono 400 -- inline code in remediation guidance, API responses.
- **Loading:** Google Fonts CDN (`fonts.googleapis.com/css2?family=Geist:wght@400;500;600;700&family=Geist+Mono:wght@400;500`)
- **Scale:** 11px (mono labels) / 12px (labels) / 13px (small text, descriptions) / 14px (body small, buttons) / 15px (inputs) / 16px (body) / 18px (subhead) / 32px (h2) / 48px (h1 display)

## Color
- **Approach:** Restrained. One accent + semantic colors. Color is rare and meaningful.
- **Background Primary:** #0a0a0f -- near-black with subtle blue undertone
- **Background Surface:** #111118 -- cards, panels, elevated containers
- **Background Elevated:** #1a1a24 -- hover states, nested surfaces
- **Background Hover:** #222230 -- interactive hover feedback
- **Text Primary:** #fafafa -- high contrast on dark backgrounds
- **Text Secondary:** #a1a1aa -- descriptions, helper text (warm gray)
- **Text Tertiary:** #71717a -- timestamps, metadata, deemphasized
- **Text Muted:** #52525b -- placeholder text, disabled states
- **Accent (Pass/Security):** #22c55e -- scan pass, positive states, primary CTAs, brand accent
- **Accent (Pass Dim):** #166534 -- grade badge background for A/A+ grades
- **Error (Fail/Findings):** #ef4444 -- high severity findings, error states
- **Error Dim:** #7f1d1d -- grade badge background for D/F grades
- **Warning:** #f59e0b -- medium severity findings, caution states
- **Warning Dim:** #78350f -- grade badge background for C grades
- **Info:** #3b82f6 -- informational alerts, links
- **Border Default:** #27272a -- card borders, dividers
- **Border Subtle:** #1e1e24 -- inner dividers, finding separators
- **Focus Ring:** accent green (#22c55e) for keyboard focus indicators
- **Dark mode:** This IS the dark mode. No light mode planned. The product audience lives in dark IDEs.

## Spacing
- **Base unit:** 4px
- **Density:** Comfortable -- not cramped (Linear-level density), not wasteful
- **Scale:** 2xs(2) xs(4) sm(8) md(16) lg(24) xl(32) 2xl(48) 3xl(64)
- **Content max-width:** 1100px (via PageContainer component)

## Layout
- **Approach:** Hybrid. Landing page hero uses composition-first (full-width, centered). App UI (results, dashboard) uses data-dense single-column with cards.
- **Grid:** Single-column content within PageContainer. Two-column grid for component pairs (e.g., findings list + grade summary).
- **Max content width:** 1100px
- **Border radius:** sm(4px) md(8px) lg(12px) -- hierarchical. Cards get lg. Buttons and inputs get md. Badges get sm.

## Motion
- **Approach:** Minimal-functional. Motion aids comprehension, never decorates.
- **Easing:** enter(ease-out) exit(ease-in) move(ease-in-out)
- **Duration:** micro(50-100ms for hovers) short(150-250ms for state changes) medium(250-400ms for scan progress transitions)
- **Scan progress:** per-scanner stage animations (existing pattern, keep it)
- **No:** scroll animations, entrance effects, page transition animations, loading spinners longer than 100ms

## Component Vocabulary
- **PageContainer:** shared layout, max-width 1100px, horizontal padding 24px
- **ScanForm:** URL input + email input + CFAA checkbox + submit button
- **Grade Display:** 64x64 rounded-lg box, grade letter, color-coded (green/amber/red)
- **Severity Badge:** monospace uppercase, sm border-radius, dim background matching severity
- **Finding Item:** severity badge + title + description, separated by subtle borders
- **Alert:** colored left-accent, semantic background, single-line or multi-line
- **Share Button:** btn-secondary style, "Copy Link" label

## Anti-Patterns (never use)
- Purple/violet gradients or accents
- 3-column icon grids with colored circles
- Centered-everything layouts (hero is centered, content sections are not)
- Decorative blobs, wavy SVG dividers, floating circles
- Emoji as design elements in UI (Lucide icons only)
- Generic hero copy ("Welcome to...", "Unlock the power of...")
- Cards with colored left borders
- Stock-photo-style imagery
- Marketing fluff sections ("Trusted by 500+ companies") until real social proof exists

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Initial design system | Created by /design-consultation. Industrial/utilitarian, Geist, restrained palette, no purple. |
| 2026-03-30 | No light mode | Target audience lives in dark IDEs. One theme reduces maintenance. |
| 2026-03-30 | Geist over display fonts | "I'm a tool you use" not "I'm a brand you worship." Audience uses Next.js/Vercel. |
| 2026-03-30 | Green accent, no purple | Every security tool defaults to purple/violet. Green = security-pass, distinctive in the category. |
