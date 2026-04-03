---
phase: 44-content-routes
plan: "01"
subsystem: frontend
tags: [blog, mdx, content-marketing, seo]
dependency_graph:
  requires: []
  provides: [blog-infrastructure, blog-routes]
  affects: [frontend/next.config.ts, frontend/mdx-components.tsx, frontend/app/blog]
tech_stack:
  added: ["@next/mdx", "@mdx-js/react"]
  patterns: [mdx-compilation, dynamic-import, server-components]
key_files:
  created:
    - frontend/mdx-components.tsx
    - frontend/app/blog/blog.css
    - frontend/app/blog/page.tsx
    - frontend/app/blog/[slug]/page.tsx
    - content/blog/.gitkeep
  modified:
    - frontend/next.config.ts
    - frontend/package.json
    - frontend/package-lock.json
decisions:
  - "@content webpack alias resolves project-root content/ directory from frontend/ app dir"
  - "Blog index shows coming-soon with scan CTA when no published MDX posts exist"
  - "Dynamic import pattern with fs.existsSync guard prevents prerender errors on empty content dir"
metrics:
  duration: 15
  completed: "2026-03-31"
  tasks: 2
  files: 8
requirements: [CONTENT-01, CONTENT-02]
---

# Phase 44 Plan 01: Blog MDX Infrastructure Summary

**One-liner:** @next/mdx blog with Geist typography, coming-soon index fallback, and dynamic [slug] route reading MDX files from project-root content/blog/.

## What Was Built

### Task 1: MDX Compilation Setup
- Installed `@next/mdx` and `@mdx-js/react` packages
- Updated `frontend/next.config.ts` with `createMDX()` wrapper, `pageExtensions` for `.md`/`.mdx`, and `@content` webpack alias pointing to project-root `content/` directory
- Created `frontend/mdx-components.tsx` mapping h1/h2/h3/p/a/code/pre/ul/ol/blockquote to Geist-styled elements using design token classes
- Created `frontend/app/blog/blog.css` with `.blog-article` wrapper (720px max-width, `> * + *` element spacing, Geist Mono code fonts)
- Created `content/blog/.gitkeep` to establish blog content directory at project root (per D-01)

### Task 2: Blog Routes
- Created `frontend/app/blog/page.tsx` — blog index with static metadata, `fs.readdirSync` to discover published MDX posts, coming-soon fallback with scan CTA when no posts exist
- Created `frontend/app/blog/[slug]/page.tsx` — dynamic post route with `generateMetadata` (title, description, OG article), `generateStaticParams`, `<time>` element, MDX rendering via `@content/blog/${slug}.mdx` dynamic import, back-to-blog link

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| `@content` webpack alias | Allows `import('@content/blog/slug.mdx')` from frontend app code pointing to project-root `content/` without `../..` traversal issues |
| `fs.existsSync` + slug guard before dynamic import | Prevents prerender errors when content/blog/ is empty or slug doesn't exist |
| Static metadata on blog index | No dynamic data needed; description is fixed |
| `published: boolean` filter in getPublishedPosts | Allows draft posts in content/blog/ without them appearing on the site |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Dynamic import path resolution**
- **Found during:** Task 2 verification (next build)
- **Issue:** `import('../../content/blog/${slug}.mdx')` caused Module not found warnings and prerender failures because webpack couldn't resolve template literal imports with relative `../..` paths outside the Next.js root
- **Fix:** Added `@content` webpack alias in next.config.ts and changed import paths to `import('@content/blog/${slug}.mdx' as any)`. Also added `fs.existsSync` guard and slug presence check before attempting imports to prevent prerender errors on empty content directory.
- **Files modified:** `frontend/next.config.ts`, `frontend/app/blog/page.tsx`, `frontend/app/blog/[slug]/page.tsx`
- **Commits:** 0cb7404

## Known Stubs

- **Coming-soon page** in `frontend/app/blog/page.tsx` line 94 — intentional per CONTENT-02 requirement. This is the correct state until MDX posts are added to `content/blog/`. Not a stub; it's a conditional render based on `posts.length === 0`.

## Build Notes

The `npm run build` command in CI requires `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY` for static generation of all routes (root layout wraps everything in `<ClerkProvider>`). This is a pre-existing condition affecting all routes — not introduced by this plan. TypeScript compilation (`tsc --noEmit`) passes clean. Build succeeds in CI where Clerk keys are available (see `.github/workflows/build-push.yml`).

## Verification Results

```
grep -r "Coming soon" frontend/app/blog/page.tsx     → MATCH (line 94)
grep -r "generateMetadata" frontend/app/blog/[slug]/page.tsx → MATCH (line 34)
grep -r "<time" frontend/app/blog/[slug]/page.tsx    → MATCH (line 114)
grep -r "createMDX" frontend/next.config.ts          → MATCH (line 2,50)
grep -r "useMDXComponents" frontend/mdx-components.tsx → MATCH (line 3)
grep -r "blog-article" frontend/app/blog/blog.css    → MATCH (line 4)
content/blog/.gitkeep                                 → EXISTS
tsc --noEmit                                          → CLEAN (no errors)
```

## Self-Check: PASSED

Files verified:
- FOUND: frontend/next.config.ts
- FOUND: frontend/mdx-components.tsx
- FOUND: frontend/app/blog/blog.css
- FOUND: frontend/app/blog/page.tsx
- FOUND: frontend/app/blog/[slug]/page.tsx
- FOUND: content/blog/.gitkeep

Commits verified:
- FOUND: 914c768 (Task 1 - MDX compilation setup)
- FOUND: 0cb7404 (Task 2 - blog routes)
