---
status: complete
phase: 02-free-tier-mvp
source: 02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-04-SUMMARY.md, 02-05-SUMMARY.md, 02-06-SUMMARY.md, 02-07-SUMMARY.md, 02-08-SUMMARY.md
started: 2026-02-05T15:00:00Z
updated: 2026-02-05T15:15:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Docker Compose full stack startup
expected: Running `docker compose up --build -d` starts 3 services (db, backend, frontend). All containers healthy. `curl http://localhost:3000/health` returns success.
result: pass

### 2. Landing page loads with value proposition
expected: Opening http://localhost:3001 in a browser shows a landing page with "Ship fast, stay safe" hero text, a scan form with URL and email fields, a "What We Check" section, and a scan counter showing completed scans.
result: pass

### 3. Form validation rejects bad input
expected: Submitting the form with an invalid URL (e.g., "notaurl") or empty email shows field-level error messages inline. The form does not submit to the backend.
result: pass

### 4. Scan submission and redirect
expected: Entering a valid URL (e.g., https://example.com) and email, then clicking submit shows a success confirmation with a green checkmark. After ~2.5 seconds, the browser auto-redirects to a /scan/{id} progress page.
result: pass

### 5. Progress page with stage checklist
expected: The /scan/{id} page shows a progress checklist with 4 stages: Security Headers, TLS/SSL, Exposed Files, JavaScript Secrets. Each stage shows a pending indicator (circle) that updates to a checkmark as the backend completes it. The page polls every 2 seconds.
result: pass

### 6. Auto-redirect to results
expected: When all scan stages complete, the progress page shows a brief "Scan complete" message and then auto-redirects to /results/{token} after about 1 second.
result: pass

### 7. Results dashboard with A-F grade
expected: The results page shows an A-F letter grade in a colored circle (48px), finding count badges by severity (Critical/High/Medium/Low), and the scanned URL and timestamp.
result: pass

### 8. Findings grouped by severity with accordions
expected: Findings are grouped by severity (Critical > High > Medium > Low) by default. Each finding is an expandable accordion showing severity badge, title, description, and remediation guidance when clicked.
result: pass

### 9. Toggle severity/category grouping
expected: A toggle button switches between "By Severity" and "By Category" grouping. Category mode groups findings by scanner type (Headers, TLS, Files, Secrets). Active button has solid background, inactive has outline.
result: pass

### 10. Markdown report download
expected: Clicking "Download Report" on the results page downloads a .md file containing the scan summary, grade, and findings grouped by severity with remediation steps.
result: pass

### 11. Results expiry warning
expected: The results page shows an expiry date (3 days from scan completion). If within 24 hours of expiry, the warning appears in red; otherwise in blue.
result: pass

### 12. Invalid token shows 404
expected: Visiting /results/nonexistent-token-here shows a 404/not found page rather than an error or blank page.
result: pass

### 13. Email notification received
expected: After scan completes, an email is sent to the address provided (requires RESEND_API_KEY configured). The email shows the grade with color coding, a findings summary, and a link to the full results page.
result: pass

### 14. Re-scan CTA on results page
expected: The results page includes a "Scan Again" or similar link/button that navigates back to the landing page with the URL pre-filled for easy re-scanning.
result: pass

## Summary

total: 14
passed: 14
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
