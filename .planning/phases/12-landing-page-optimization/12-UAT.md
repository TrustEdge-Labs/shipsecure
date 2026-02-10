---
status: testing
phase: 12-landing-page-optimization
source: [12-01-SUMMARY.md, 12-02-SUMMARY.md]
started: 2026-02-09T12:00:00Z
updated: 2026-02-09T12:00:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

number: 1
name: Hero headline and subhead
expected: |
  Landing page headline reads "Security scanning for AI-generated web apps" (with gradient styling).
  Subhead below mentions specific vulnerability types: exposed .env, weak TLS, hardcoded API keys, framework misconfigurations.
  Below the form: "No signup required. Results in ~60 seconds."
awaiting: user response

## Tests

### 1. Hero headline and subhead
expected: Landing page headline reads "Security scanning for AI-generated web apps". Subhead mentions specific vulnerability types (exposed .env, weak TLS, hardcoded API keys, framework misconfigurations). Below the form: "No signup required. Results in ~60 seconds."
result: [pending]

### 2. Feature cards with technical specifics
expected: Four "What We Check" cards visible. Security Headers card lists specific headers (CSP, HSTS, etc.). TLS card mentions SSL Labs. Exposed Files card mentions 20+ paths. JavaScript Secrets card names specific patterns (AWS, Stripe, Firebase).
result: [pending]

### 3. How It Works section
expected: A "How it works" section with three numbered steps explaining the scan flow (enter URL, scan runs, get results). Steps have clear titles and brief descriptions.
result: [pending]

### 4. Expandable scan methodology
expected: Below the "How it works" steps, a clickable "Scan methodology" element. Clicking it expands to show detailed descriptions of each scanner type (security headers, TLS, exposed files, JS secrets, and paid Nuclei tier).
result: [pending]

### 5. Footer OSS attribution
expected: Site footer shows "Powered by open source:" line. Nuclei credited with link to GitHub repo, ProjectDiscovery author credit, and MIT license link. testssl.sh credited with link to website and GPLv2 license link. All links open in new tab.
result: [pending]

### 6. Copy tone check
expected: Reading through the full landing page, the copy feels like developer-focused technical communication — not marketing material. No superlatives (best, fastest, most powerful), no vague promises, no enterprise jargon. Specific and honest about what the tool does.
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0

## Gaps

[none yet]
