# Requirements: ShipSecure

**Defined:** 2026-02-08
**Core Value:** Catch security flaws in vibe-coded apps before they become breaches, with remediation guidance anyone can follow — no security expertise required.

## v1.2 Requirements

Requirements for launch readiness. Each maps to roadmap phases.

### UX Polish

- [ ] **UX-01**: All pages render correctly on mobile viewports (375px-428px) without horizontal scroll or overlapping elements
- [ ] **UX-02**: All pages render correctly on tablet viewports (768px-1024px) with appropriate layout adjustments
- [ ] **UX-03**: Scan submission shows visual progress with stage-specific feedback ("Checking security headers...", "Running Nuclei templates...")
- [ ] **UX-04**: All API errors display constructive inline messages with suggested actions (never silent failures)
- [ ] **UX-05**: Visual design is consistent across all pages (spacing, colors, button styles, typography)
- [ ] **UX-06**: Lighthouse performance score >90 on landing page and results page

### Legal

- [ ] **LEGAL-01**: Privacy Policy page covers email collection, Stripe payment data, analytics, GDPR/CCPA rights, and data deletion requests
- [ ] **LEGAL-02**: Terms of Service page covers acceptable use, scanning consent/authorization, liability limits, and refund policy
- [ ] **LEGAL-03**: Legal pages are linked from site footer on all pages

### Analytics

- [ ] **ANLYT-01**: Plausible analytics script loads on all pages and tracks pageviews
- [ ] **ANLYT-02**: Custom events track key conversions (scan submitted, paid audit purchased)

### SEO

- [ ] **SEO-01**: All pages have unique, descriptive title tags and meta descriptions
- [ ] **SEO-02**: Landing page has Open Graph tags (title, description, image, URL) for social sharing
- [ ] **SEO-03**: Scan results pages have noindex/nofollow meta tags (private content)

### Landing Page

- [ ] **LAND-01**: Landing page copy is developer-focused, technically honest, and free of marketing jargon
- [ ] **LAND-02**: Landing page includes clear "how it works" section explaining scan methodology
- [ ] **LAND-03**: Open-source tool attribution (Nuclei, testssl.sh) visible in footer

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Trust Signals

- **TRUST-01**: About page with founder photo, bio, credentials, and "why I built this" story
- **TRUST-02**: Example scan results publicly accessible without signup (demo scan URL)
- **TRUST-03**: Transparent "How it works" dedicated page with full scanner methodology

### Enhanced UX

- **EUX-01**: Real-time scan insights showing current scan stage and preliminary finding counts
- **EUX-02**: Email support SLA documentation (24hr response time)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| User accounts / scan history | Massive scope (auth, sessions, password reset); free tier explicitly avoids signup |
| Live chat support | Founder burnout risk; one-person team; email sufficient for launch |
| WebSocket real-time updates | Polling works fine; adds deployment complexity for marginal UX gain |
| SOC 2 / ISO certifications | Costs $20K-50K, takes 3-6 months; overkill for bootstrapped MVP |
| AI-powered explanations | Adds API cost, latency, hallucination risk; curated templates safer |
| Cookie consent banner | Not needed with cookieless Plausible analytics |
| Social login (OAuth) | Only useful with user accounts; email-only flow sufficient |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| UX-01 | Phase 11 | Pending |
| UX-02 | Phase 11 | Pending |
| UX-03 | Phase 11 | Pending |
| UX-04 | Phase 11 | Pending |
| UX-05 | Phase 11 | Pending |
| UX-06 | Phase 11 | Pending |
| LEGAL-01 | Phase 10 | Pending |
| LEGAL-02 | Phase 10 | Pending |
| LEGAL-03 | Phase 10 | Pending |
| ANLYT-01 | Phase 8 | Pending |
| ANLYT-02 | Phase 8 | Pending |
| SEO-01 | Phase 9 | Pending |
| SEO-02 | Phase 9 | Pending |
| SEO-03 | Phase 9 | Pending |
| LAND-01 | Phase 12 | Pending |
| LAND-02 | Phase 12 | Pending |
| LAND-03 | Phase 12 | Pending |

**Coverage:**
- v1.2 requirements: 17 total
- Mapped to phases: 17
- Unmapped: 0
- Coverage: 100%

---
*Requirements defined: 2026-02-08*
*Last updated: 2026-02-08 after roadmap creation*
