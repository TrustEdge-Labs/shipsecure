# Requirements: v1.9 Customer Acquisition

## Funnel
- [ ] **FUNNEL-01**: Anonymous user can scan any URL (not locked to demo target)
- [ ] **FUNNEL-02**: Anonymous scans rate-limited to 3 per IP per day
- [ ] **FUNNEL-03**: Per-target rate limit of 5 scans per domain per hour (return cached results when exceeded)
- [ ] **FUNNEL-04**: Authenticated user can scan any URL without domain verification
- [ ] **FUNNEL-05**: User can copy scan results URL via share button on results page
- [ ] **FUNNEL-06**: Expired results page shows "scan again" CTA with pre-filled URL
- [ ] **FUNNEL-07**: Results page has OG meta tags with grade and finding count for social sharing

## Content
- [ ] **CONTENT-01**: /blog route renders MDX blog posts with proper typography
- [ ] **CONTENT-02**: /blog index shows "coming soon" page with scan CTA when no posts exist
- [ ] **CONTENT-03**: /check/{platform} landing pages for Lovable, Bolt, v0 with platform-specific accent colors and CVE context
- [ ] **CONTENT-04**: /check/{platform} pre-fills scan form with platform-appropriate placeholder URL

## Analytics
- [ ] **ANALYTICS-01**: Plausible fires "Scan Started" event when anonymous scan submitted
- [ ] **ANALYTICS-02**: Plausible fires "Signup Completed" event after Clerk signup
- [ ] **ANALYTICS-03**: Plausible fires "Share Clicked" event when share button used

## Future Requirements
- Retention mechanism (scheduled re-scans, email digests) -- after first 10 users
- Competitive differentiation statement on landing page -- after user feedback
- Pro tier pricing -- after demand signals
- Dynamic OG images with grade badge (Satori) -- after share volume > 10/week
- Email capture on demo results -- after funnel proves conversion

## Out of Scope
- Skill Scan v1 -- competitive market already served by Snyk, ClawSecure, AgentGuard
- Scan grade badges -- stale badge liability, point-in-time grade goes stale immediately
- Domain verification (removed, not deferred) -- reduces friction, add back if abuse occurs
- B2B platform partnerships -- solo dev constraint, not ready for enterprise sales
- Light mode -- target audience lives in dark IDEs, single theme reduces maintenance

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FUNNEL-01 | Phase 42 | Pending |
| FUNNEL-02 | Phase 42 | Pending |
| FUNNEL-03 | Phase 42 | Pending |
| FUNNEL-04 | Phase 42 | Pending |
| FUNNEL-05 | Phase 43 | Pending |
| FUNNEL-06 | Phase 43 | Pending |
| FUNNEL-07 | Phase 43 | Pending |
| CONTENT-01 | Phase 44 | Pending |
| CONTENT-02 | Phase 44 | Pending |
| CONTENT-03 | Phase 44 | Pending |
| CONTENT-04 | Phase 44 | Pending |
| ANALYTICS-01 | Phase 45 | Pending |
| ANALYTICS-02 | Phase 45 | Pending |
| ANALYTICS-03 | Phase 45 | Pending |
