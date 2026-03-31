# Phase 44: Content Routes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-03-31
**Phase:** 44-content-routes
**Areas discussed:** Blog post structure, /check page CVE content, Platform accent implementation

---

## Blog Post Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Content directory | content/blog/*.mdx at project root. Separate content from code. | Yes |
| Alongside route | frontend/app/blog/posts/*.mdx. Everything in one place. | |
| You decide | Claude picks. | |

**User's choice:** Content directory
**Notes:** Cleaner git diffs, separates content from code.

---

## /check Page CVE Content

| Option | Description | Selected |
|--------|-------------|----------|
| Specific CVE callout | Lovable names CVE, Bolt/v0 use general vibe-code stats. | Yes |
| Generic security messaging | Same pitch per platform, different names. | |
| You decide | Claude writes appropriate copy. | |

**User's choice:** Specific CVE callout per platform

---

## Platform Accent Implementation

| Option | Description | Selected |
|--------|-------------|----------|
| CSS variables per page | Override --accent-* in style block. | |
| Data-driven from config | Platform config object, shared template component. | Yes |
| You decide | Claude picks. | |

**User's choice:** Data-driven config
**Notes:** One shared component, platform config objects with name/accent/placeholder/cveText. Cleanest for future platforms.

---

## Claude's Discretion

- MDX config approach
- Platform config file location
- Blog post typography styling

## Deferred Ideas

None
