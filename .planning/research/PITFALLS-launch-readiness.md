# Pitfalls Research

**Domain:** Launch readiness features for developer-focused security scanning SaaS
**Researched:** 2026-02-08
**Confidence:** MEDIUM

## Critical Pitfalls

### Pitfall 1: Hydration Mismatch from Viewport-Dependent Rendering

**What goes wrong:**
Server renders one component tree, client renders different tree based on viewport size, causing React hydration errors and broken interactivity on mobile devices. The server will guess wrong, and the client will render the correct layout, causing a mismatch on the first render.

**Why it happens:**
Developers use `matchMedia`, `useMediaQuery`, or direct viewport checks to render different components for mobile vs desktop. Common pattern: `{isMobile ? <MobileNav /> : <DesktopNav />}` causes server to render one option, client hydrates with the other.

**How to avoid:**
- Use CSS-driven responsiveness instead of branching markup based on viewport
- CSS changes layout without changing DOM tree — same markup, different styles
- Keep component tree identical on server and client, use `display: none` or `visibility: hidden` for mobile/desktop variations
- Never use `window.innerWidth` or `navigator.userAgent` during initial render

**Warning signs:**
- Console errors: "Text content does not match server-rendered HTML"
- Mobile users report components not loading or being unclickable
- DevTools show hydration warnings in production

**Phase to address:**
Phase 1 (UX Polish & Mobile Responsiveness) — establish CSS-only responsive patterns before implementing features

---

### Pitfall 2: Third-Party Script CSP Conflicts

**What goes wrong:**
Analytics scripts (Plausible/Umami) blocked by Content Security Policy, causing silent tracking failures. Strict CSP blocks legitimate scripts, overly permissive CSP defeats security purpose. Google Analytics requires 187 separate domain entries in CSP.

**Why it happens:**
- Default Next.js CSP doesn't include analytics domains
- Self-hosted analytics assumed safe, but inline scripts still blocked by `script-src 'self'`
- CSP configured in development, breaks in production with different domains
- Docker reverse proxy changes request origins, breaking `connect-src` rules

**How to avoid:**
- Self-hosted analytics (Plausible/Umami) requires adding tracking domain to `script-src` and `connect-src`
- Use nonce-based CSP: `script-src 'nonce-{random}'` for inline scripts
- Test CSP in production-like environment (Docker with Nginx proxy)
- Monitor CSP violation reports to catch blocked legitimate requests
- For Umami specifically: add `data-do-not-track` flag to respect user preferences

**Warning signs:**
- Analytics dashboard shows zero traffic despite site visitors
- Browser console: "Refused to load script because it violates CSP directive"
- CSP violation reports appearing in logs
- Analytics working in development but not production

**Phase to address:**
Phase 2 (Analytics Integration) — configure CSP before integrating analytics, verify in production environment

---

### Pitfall 3: Unauthorized Scanning Legal Liability

**What goes wrong:**
User initiates scan against website they don't own, ShipSecure becomes liable under Computer Fraud and Abuse Act (CFAA). Security scanner service faces legal action for enabling unauthorized access attempts.

**Why it happens:**
- Free tier has no authentication → no user identity for legal recourse
- TOS buried or not shown before scan → user never explicitly consents
- No technical validation of ownership (DNS TXT record, file upload, email confirmation)
- Assumption that "publicly accessible website" means "scannable without permission"

**How to avoid:**
- **Critical:** Explicit consent checkbox before scan: "I confirm I own this website or have written authorization to scan it"
- TOS must state: "Service may only be used to scan websites you own or have explicit written permission to scan"
- Liability disclaimer: "User is solely responsible for all scans and legal consequences of unauthorized scanning"
- Email capture on free tier creates legal trail (user identity)
- Consider ownership verification for paid audits (DNS TXT or file upload)
- Terms must cite CFAA: scanning third-party assets without authorization is prohibited and may violate 18 U.S.C. § 1030

**Warning signs:**
- Legal review flags missing consent mechanism
- TOS doesn't explicitly prohibit unauthorized scanning
- No email capture = no user accountability
- Hacker News comments raise CFAA concerns

**Phase to address:**
Phase 3 (Legal Pages) — must be in place before public launch, blocking legal risk

---

### Pitfall 4: Missing Viewport Meta Tag or Incorrect Placement

**What goes wrong:**
Mobile browsers render desktop layout at 980px width and scale down, making text unreadable. Users pinch-to-zoom constantly. Site appears broken on mobile despite responsive CSS.

**Why it happens:**
- Viewport meta tag added in `_document.js` causes Next.js deduplication errors
- Forgotten entirely when developer only tests in desktop browser
- Incorrect content value: `width=device-width, initial-scale=1.0` is required

**How to avoid:**
- **Next.js Pages Router:** Add to `pages/_app.js` using `next/head`
- **Next.js App Router:** Add to `app/layout.tsx` in `<head>` with `content="width=device-width, initial-scale=1.0"`
- Never add viewport meta in `_document.js`
- Test on real mobile devices, not just DevTools responsive mode

**Warning signs:**
- Site looks tiny on mobile, users must zoom in
- Media queries not triggering on mobile devices
- Next.js warning: "Viewport meta tags should not be used in _document.js"

**Phase to address:**
Phase 1 (UX Polish & Mobile Responsiveness) — foundation for all mobile work

---

### Pitfall 5: Show HN Post Flagged or Ignored

**What goes wrong:**
Hacker News submission flagged as spam, buried by vote-ring detection, or ignored due to poor presentation. Launch fails to gain traction despite good product.

**Why it happens:**
- Marketing language ("fastest", "best", "revolutionary") triggers instant rejection
- Signup wall prevents HN users from trying product (violates Show HN guidelines)
- Username matches company name → looks promotional
- Shared link on Slack/Discord for upvotes → vote-ring penalty
- No URL in submission, only text description
- Can't tell what product does from title/description

**How to avoid:**
- **Title format:** "Show HN: ShipSecure – Security scanner for AI-generated apps"
- **Submission:** Put URL in URL field, leave text field blank
- **Landing page:** Free tier must work without signup (HN guideline: "things people can run")
- **Language:** Technical and modest, not marketing ("scans for X, finds Y, shows Z")
- **Username:** Personal, not "shipsecure" or "trustedge"
- **No vote coordination:** Don't ask anyone to upvote
- **Clear description:** "I built X because Y" in first comment, explain what it does

**Warning signs:**
- Draft title uses superlatives or sales language
- Free tier requires email before showing any results
- Planning to share link for upvotes
- Can't explain product in one sentence

**Phase to address:**
Phase 5 (Landing Page Copy) — must align with Show HN guidelines, verify free tier works without friction

---

### Pitfall 6: SEO Meta Tags Missing or Hydration-Broken

**What goes wrong:**
Open Graph preview shows "No preview available" when shared on Twitter/Slack. Google doesn't index properly. Meta tags rendered client-side only, invisible to crawlers and social media bots.

**Why it happens:**
- Meta tags added in client component with `useEffect` → invisible to SSR
- Images referenced as relative paths, bots can't resolve
- Title/description not in `generateMetadata()` for App Router
- OG image wrong size (needs 1200x630px for Twitter/Facebook)
- Multilingual sites reuse English metadata for all languages

**How to avoid:**
- **App Router:** Use `generateMetadata()` in `layout.tsx` or `page.tsx` (server-rendered)
- **Pages Router:** Use `next/head` in `_app.js` or page component
- OG image must be absolute URL: `https://shipsecure.ai/og-image.png`
- Verify size: 1200x630px for high-res Twitter/Facebook cards
- Test with [Twitter Card Validator](https://cards-dev.twitter.com/validator) and [Facebook Sharing Debugger](https://developers.facebook.com/tools/debug/)
- Include: title, description, og:image, og:url, twitter:card

**Warning signs:**
- Slack/Discord preview shows blank card
- View source shows no meta tags (only added client-side)
- Google Search Console shows missing metadata warnings
- Social shares have no image preview

**Phase to address:**
Phase 4 (SEO/OG Meta Tags) — must be server-rendered, verified before public launch

---

### Pitfall 7: Analytics Database Misconfiguration in Docker

**What goes wrong:**
Umami/Plausible fails to connect to database, analytics container crashes on startup. Database URL environment variable doesn't match actual database credentials. Migrations fail silently.

**Why it happens:**
- `DATABASE_URL` format differs between Postgres and MySQL
- Environment variables in `docker-compose.yml` don't match database container settings
- Database container uses different port internally vs externally
- Analytics container starts before database is ready (race condition)
- Umami migration documentation incomplete for Docker deployments

**How to avoid:**
- Match DATABASE_URL to actual db container: `postgresql://user:pass@db:5432/dbname`
- Use `depends_on` with health checks in docker-compose
- Expose analytics only on localhost:3000, not publicly (security)
- Test database connection before running migrations
- For ShipSecure: reuse existing Postgres instance, don't create separate database
- DigitalOcean Managed Postgres: use `doadmin` user for schema CREATE privileges

**Warning signs:**
- Analytics container logs show connection refused errors
- `docker-compose up` shows analytics exiting immediately
- Database migrations not applying
- Port conflicts with existing services

**Phase to address:**
Phase 2 (Analytics Integration) — must configure correctly in production Docker environment

---

### Pitfall 8: Terms of Service Template Liability Gaps

**What goes wrong:**
Generic TOS template doesn't address security scanning liability. Service gets sued when user scans competitor's site. No disclaimer about scan accuracy. GDPR/privacy obligations missed.

**Why it happens:**
- ChatGPT-generated TOS not legally binding, non-compliant with privacy laws
- Generic SaaS template doesn't cover security testing specifics
- Data Processing Agreement (DPA) treated as "available upon request" instead of required
- Vague language: "We may share data" instead of explicit third-party list
- No security patching timeframe commitments
- Missing clause about user responsibility for authorization

**How to avoid:**
- **Required clauses for security scanner:**
  - User represents they own or have written authorization to scan target
  - User solely responsible for legal consequences of unauthorized scans
  - Service not liable for scan accuracy, false positives, or missed vulnerabilities
  - Scan results "as-is" with no warranty of completeness
  - Cite CFAA compliance requirement (18 U.S.C. § 1030)
- Include DPA terms in main contract or as required exhibit, not "upon request"
- Specify security patching: "Critical vulnerabilities patched within 72 hours"
- List all third-party data sharing explicitly (Stripe, email provider, analytics)
- Consider legal review before launch (40+ years cybersecurity experience = target for lawsuits)

**Warning signs:**
- TOS generated by AI without legal review
- No mention of CFAA or authorization requirements
- Vague data sharing language
- No DPA or security commitments
- Generic template not customized for security scanning

**Phase to address:**
Phase 3 (Legal Pages) — must be legally sound before accepting payments or launching publicly

---

### Pitfall 9: Mobile Performance Testing Only in DevTools

**What goes wrong:**
Site feels fast in Chrome DevTools responsive mode, slow and janky on real mobile devices. Core Web Vitals fail on actual phones. Images load at desktop resolution on mobile. Touch gestures don't work.

**Why it happens:**
- DevTools throttling doesn't match real mobile CPU/network constraints
- Desktop images served to mobile (no `sizes` prop on `next/image`)
- JavaScript bundle too large, blocking mobile render
- LCP (Largest Contentful Paint) > 2.5s on mobile, < 2.5s on desktop
- CLS (Cumulative Layout Shift) from images without width/height
- Third-party scripts (analytics) mutate DOM before hydration

**How to avoid:**
- Test on real devices: iPhone, Android phone, not just DevTools
- Use `next/image` with `sizes` prop: responsive srcset for mobile
- Measure Core Web Vitals with PageSpeed Insights or WebPageTest (mobile profile)
- Target: LCP < 2.5s, CLS < 0.1, INP < 200ms
- Always specify width/height on images to prevent CLS
- Lazy load non-critical components with `next/dynamic`
- Monitor: 53% of mobile users abandon sites > 3 seconds load time

**Warning signs:**
- PageSpeed Insights mobile score < 90 but desktop score > 90
- Real device testing shows janky scrolling
- Mobile users report slow loading
- Images downloading at full desktop resolution on mobile

**Phase to address:**
Phase 1 (UX Polish & Mobile Responsiveness) — establish performance budget and testing protocol

---

### Pitfall 10: Landing Page Copy Too Technical or Too Vague

**What goes wrong:**
Developer audience bounces because value proposition unclear or implementation complexity seems high. Technical jargon alienates decision-makers. Vague benefits don't differentiate from competitors.

**Why it happens:**
- Focus on features ("uses Nuclei engine") instead of outcomes ("finds 20+ vulnerability types")
- Vague value prop: "Better security for your apps" vs. "Scans AI-generated code for SQL injection, XSS, SSRF in 60 seconds"
- Unclear CTA: "Get Started" vs. "Scan Your Site Free (No Signup)"
- No implementation visibility for technical audience (developers want to see the code/process)
- Missing trust signals for sensitive use case (security scanning requires trust)

**How to avoid:**
- **Headline:** Clear technical outcome: "Security scanner for AI-generated apps"
- **Subheadline:** Reduce perceived risk: "Free scan in 60 seconds. No signup required."
- **Benefits over features:** "Finds SQL injection, XSS, SSRF" not "Powered by Nuclei"
- **Trust signals:** "Built by 40+ year cybersecurity veteran", show sample report
- **Developer experience:** Code preview or scan demo above the fold
- **Clear CTA:** Action-oriented: "Scan My Site Free" not "Learn More"
- **Avoid:** Superlatives ("fastest", "best"), jargon without context, vague promises

**Warning signs:**
- Bounce rate > 70% on landing page
- Hacker News comments: "What does this actually do?"
- A/B test shows vague headline underperforms specific headline
- No mention of free tier or signup friction in hero section

**Phase to address:**
Phase 5 (Landing Page Copy) — message must resonate with technical audience and HN community

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip ownership verification | Faster signup flow | CFAA legal liability, potential lawsuits | **Never** — legal risk too high |
| Generic TOS template | Launch faster | Non-compliant, liability gaps for security scanning | Only if legal review planned pre-revenue |
| Client-side meta tags | Simpler implementation | SEO broken, social shares broken, Googlebot can't index | **Never** — defeats entire purpose |
| Skip CSP configuration | Faster analytics integration | Analytics silently broken in production | Only in development, must fix before production |
| Desktop-only testing | Faster QA | 53% of mobile users bounce (> 3s load time) | Only in early prototype, must fix before launch |
| Vote coordination for HN | Higher initial visibility | Vote-ring penalty, post buried/flagged | **Never** — HN detects and penalizes |

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Umami/Plausible Analytics | DATABASE_URL doesn't match docker-compose credentials | Match user, pass, db name, use internal container network (`db:5432` not `localhost:5432`) |
| Next.js Image Optimization | No `sizes` prop → desktop images on mobile | Add `sizes="(max-width: 768px) 100vw, 1200px"` for responsive images |
| CSP with Self-Hosted Analytics | `script-src 'self'` blocks inline tracking code | Add analytics domain to `script-src` and `connect-src`, or use nonces |
| Stripe Payment Forms | Stripe.js blocked by CSP | Add `https://js.stripe.com` to `script-src`, `https://*.stripe.com` to `connect-src` and `frame-src` |
| DigitalOcean Managed Postgres | Regular user can't CREATE schema for analytics | Use `doadmin` user or pre-create schema with admin privileges |
| Nginx Reverse Proxy + Docker | Backend API 404s, health endpoint exposed publicly | Map `/api/*` to backend, `/` to frontend; `/health` internal only |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Unoptimized images on mobile | Slow LCP, high bandwidth usage | Use `next/image` with `sizes` prop, serve WebP/AVIF | Immediately on mobile (53% bounce rate) |
| Large JS bundle to mobile | Slow initial load, poor INP | Use `next/dynamic` for code splitting, reduce bundle size | 100+ concurrent mobile users |
| Analytics container on public port | Security risk, potential abuse | Bind to localhost:3000, access via SSH tunnel only | First security scan of infrastructure |
| No Core Web Vitals monitoring | Silent performance degradation | Set up PageSpeed Insights monitoring, track LCP/CLS/INP | When organic traffic drops due to poor SEO |
| Client-side rendering for meta tags | Bots see blank metadata | Use `generateMetadata()` (App Router) or `next/head` (Pages Router) | Immediately — Googlebot can't index |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| No consent mechanism for scans | CFAA legal liability, enabling unauthorized scanning | Explicit checkbox: "I own this site or have written authorization" before scan |
| Generic SaaS TOS | Legal liability when user scans without authorization | Include CFAA compliance clause, user responsibility for authorization |
| Analytics exposed on public port | Unauthorized access to analytics dashboard | Docker bind localhost only, nginx doesn't proxy analytics port |
| Missing CSP for analytics | XSS via analytics script injection (if compromised) | Configure CSP with specific domains, use nonces for inline scripts |
| No rate limiting on free scans | Abuse for reconnaissance of third-party sites | Rate limit by IP (already planned in backend) |
| Vague data sharing in Privacy Policy | GDPR violations, loss of user trust | Explicitly list all third parties: Stripe, Resend, analytics provider |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Signup required for free scan | HN users bounce immediately, violates Show HN guidelines | Email-only free tier (as planned), show results before capture |
| No mobile testing | 53% of users on phones see broken layout | Test on real devices, not just DevTools responsive mode |
| TOS not shown before scan | User scans site without knowing they're liable | Inline consent checkbox with link to TOS, required before scan |
| Vague scan results | User doesn't understand severity or how to fix | Clear severity labels, fix recommendations, code examples |
| No loading indicators | User thinks scan failed, refreshes page | Progress indicator, estimated time, status updates |
| Desktop-optimized layout on mobile | Unreadable text, broken navigation | CSS-only responsive design, same DOM tree, different styles |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Mobile responsiveness:** Often missing viewport meta tag — verify `content="width=device-width, initial-scale=1.0"` in correct location (not `_document.js`)
- [ ] **Analytics integration:** Often missing CSP configuration — verify browser console shows no CSP violations in production
- [ ] **SEO meta tags:** Often rendered client-side only — verify view-source shows meta tags, test with Twitter Card Validator
- [ ] **Legal pages:** Often use AI-generated template — verify CFAA clause, authorization requirements, DPA terms for security scanning
- [ ] **Hacker News launch:** Often has signup friction — verify free tier works completely without account creation
- [ ] **Image optimization:** Often missing `sizes` prop — verify mobile devices download mobile-sized images, not desktop
- [ ] **Core Web Vitals:** Often only tested in DevTools — verify real mobile device testing, PageSpeed Insights mobile score > 90
- [ ] **Docker analytics:** Often has database connection issues — verify DATABASE_URL matches container credentials, migrations applied
- [ ] **Landing page copy:** Often too vague or too technical — verify headline states clear outcome, CTA shows free tier prominently
- [ ] **Terms of Service:** Often missing scan authorization clause — verify user consent checkbox before scan, TOS prohibits unauthorized scanning

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Hydration mismatch shipped to production | MEDIUM | Refactor viewport checks to CSS-only, deploy fix, monitor error rates, test on real devices |
| Show HN post flagged/buried | HIGH | Cannot resubmit same product for months, build community presence first, fix friction points, soft launch to build organic interest |
| TOS missing CFAA clause | HIGH | Immediate legal review, add authorization consent checkpoint, email existing users with updated terms, consider pausing new signups until fixed |
| CSP blocks analytics | LOW | Add domains to CSP header, deploy via nginx config update, verify in production, check historical data retention |
| Mobile performance failing | MEDIUM | Add `next/image` with `sizes`, code-split with `next/dynamic`, defer non-critical scripts, re-test Core Web Vitals |
| Unauthorized scan incident | VERY HIGH | Legal consultation immediately, document user consent trail, cooperate with affected party, strengthen consent mechanism |
| SEO meta tags client-only | LOW | Migrate to `generateMetadata()`, deploy, request Google re-crawl via Search Console, wait 1-2 weeks for re-index |
| Analytics database misconfigured | LOW | Fix DATABASE_URL in docker-compose, restart container, verify connection, check historical data (may be lost) |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Hydration mismatch from viewport-dependent rendering | Phase 1: UX Polish & Mobile Responsiveness | Test on real mobile devices, check browser console for hydration errors |
| Third-party script CSP conflicts | Phase 2: Analytics Integration | Verify analytics tracking in production with CSP enabled, check browser console |
| Unauthorized scanning legal liability | Phase 3: Legal Pages | Legal review confirms CFAA clause, consent checkbox implemented and required |
| Missing viewport meta tag | Phase 1: UX Polish & Mobile Responsiveness | Mobile browser displays correct width, text readable without zoom |
| Show HN post flagged or ignored | Phase 5: Landing Page Copy | Free tier works without signup, language is modest/technical, URL submission ready |
| SEO meta tags missing or hydration-broken | Phase 4: SEO/OG Meta Tags | View-source shows tags, Twitter Card Validator shows preview, PageSpeed Insights validates |
| Analytics database misconfiguration in Docker | Phase 2: Analytics Integration | Analytics dashboard shows live traffic, docker logs show no connection errors |
| Terms of Service template liability gaps | Phase 3: Legal Pages | Legal review approved, DPA included, security scanning clauses present |
| Mobile performance testing only in DevTools | Phase 1: UX Polish & Mobile Responsiveness | PageSpeed Insights mobile score > 90, real device testing shows fast load |
| Landing page copy too technical or too vague | Phase 5: Landing Page Copy | A/B testing shows clear headline outperforms vague, HN feedback is positive |

## Sources

**Next.js Mobile & Hydration:**
- [Next.js Hydration Errors in 2026: The Real Causes, Fixes, and Prevention Checklist](https://medium.com/@blogs-world/next-js-hydration-errors-in-2026-the-real-causes-fixes-and-prevention-checklist-4a8304d53702)
- [Why Your Next.js App Feels Slow on Mobile (And How to Fix It)](https://medium.com/@sureshdotariya/why-your-next-js-app-feels-slow-on-mobile-and-how-to-fix-it-eeb686935cb8)
- [How do you go about responsiveness? · vercel/next.js · Discussion #13356](https://github.com/vercel/next.js/discussions/13356)
- [Viewport meta tags should not be used in _document.js | Next.js](https://nextjs.org/docs/messages/no-document-viewport-meta)

**Analytics & CSP:**
- [CSP Policy for Google Analytics GA4](https://content-security-policy.com/examples/google-analytics/)
- [Content Security Policy: Is your web security blocking your data?](https://www.adviso.ca/en/blog/tech-en/content-security-policy-csp-does-your-web-security-block-your-data)
- [Umami vs Plausible vs Matomo for Self-Hosted Analytics](https://aaronjbecker.com/posts/umami-vs-plausible-vs-matomo-self-hosted-analytics/)
- [Self-Hosted Site Analytics with Umami, Docker, and Traefik](https://aaronjbecker.com/posts/self-hosted-analytics-umami-docker-compose-traefik/)

**Legal & CFAA:**
- [Justice Manual | 9-48.000 - Computer Fraud and Abuse Act | United States Department of Justice](https://www.justice.gov/jm/jm-9-48000-computer-fraud)
- [Future-Proof Your SaaS: U.S. Federal Privacy Trends to Watch in 2026](https://thedataprivacylawyer.com/2026/01/06/future-proof-your-saas-u-s-federal-privacy-trends-to-watch-in-2026/)
- [The Data Privacy & Security Clauses in SaaS Agreements Attorneys Can't Overlook](https://callidusai.com/blog/data-privacy-security-clauses-in-saas-agreements/)
- [Terms of Service - Website Security Scanner](https://webscansec.com/legal/terms)
- [Legal Issues | Nmap Network Scanning](https://nmap.org/book/legal-issues.html)

**SEO & Meta Tags:**
- [Next.js SEO Optimization Guide (2026 Edition)](https://www.djamware.com/post/697a19b07c935b6bb054313e/next-js-seo-optimization-guide--2026-edition)
- [Getting Started: Metadata and OG images | Next.js](https://nextjs.org/docs/app/getting-started/metadata-and-og-images)
- [How to Configure Meta Tags for SEO in Next.js (2024 Guide)](https://hanabitech.com/blogs/25)

**Hacker News Launch:**
- [Show HN Guidelines](https://news.ycombinator.com/showhn.html)
- [How to Post on Hacker News Without Getting Flagged or Ignored](https://dev.to/developuls/how-to-post-on-hacker-news-without-getting-flagged-or-ignored-2eaf)
- [How to launch a dev tool on Hacker News](https://www.markepear.dev/blog/dev-tool-hacker-news-launch)
- [My Show HN reached Hacker News front page. Here is how you can do it.](https://www.indiehackers.com/post/my-show-hn-reached-hacker-news-front-page-here-is-how-you-can-do-it-44c73fbdc6)

**Landing Page Copy:**
- [Top Landing Page Copywriting Mistakes to Avoid in 2026](https://www.landingpageflow.com/post/top-landing-page-copywriting-mistakes-to-avoid)
- [10 Landing Page Copywriting Mistakes That Hurt Conversion](https://zenithcopy.com/landing-page-copywriting-mistakes/)
- [13 common landing page mistakes in 2026 and how to fix them](https://www.zoho.com/landingpage/landing-page-mistakes.html)

**Core Web Vitals:**
- [How to Optimize Core Web Vitals in NextJS App Router for 2025](https://makersden.io/blog/optimize-web-vitals-in-nextjs-2025)
- [Core Web Vitals optimization guide 2025](https://www.ateamsoftsolutions.com/core-web-vitals-optimization-guide-2025-showing-lcp-inp-cls-metrics-and-performance-improvement-strategies-for-web-applications/)
- [Web Vitals | Articles | web.dev](https://web.dev/articles/vitals)

---
*Pitfalls research for: Launch readiness features (UX polish, analytics, legal, SEO, landing page copy) for ShipSecure security scanning SaaS*
*Researched: 2026-02-08*
