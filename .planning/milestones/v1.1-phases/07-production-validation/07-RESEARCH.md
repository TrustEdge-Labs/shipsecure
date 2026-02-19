# Phase 07: Production Validation - Research

**Researched:** 2026-02-08
**Domain:** Production validation and smoke testing
**Confidence:** MEDIUM-HIGH

## Summary

Phase 07 validates the deployed ShipSecure application through comprehensive end-to-end smoke testing in the production environment. This is a "fix-it-here" phase where all failures must be resolved before completion, not just documented. The validation approach focuses on critical path testing: verify each of the 5 scanners independently, test the full scan pipeline end-to-end, confirm email delivery to actual inboxes (not just API success), validate Stripe test-mode checkout and webhook processing, test PDF report generation with Liberation Sans fonts, and verify systemd service recovery after simulated crashes.

**Key architectural insight:** Production validation differs from integration testing because it validates infrastructure configuration (DNS, SSL, Nginx routing, systemd supervision, font installation, email domain verification) rather than just application logic. Many issues only surface in production due to environmental differences.

**Primary recommendation:** Execute validation in ordered stages (infrastructure → individual scanners → full pipeline → payment flow → resilience) and fix blocking issues immediately before proceeding. Use testphp.vulnweb.com or a similar deliberately vulnerable public test application as the scan target. Configure Stripe test-mode keys on production to validate payment flow without real transactions. Verify email delivery by checking actual inbox, not just Resend API 200 response.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Validation scope:**
- Test each of the 5 scanners individually, confirming each returns findings
- Then test the full scan pipeline end-to-end (submit URL → results)
- Email delivery verified by checking actual inbox (not just API success)
- Service recovery tested by actually killing containers and verifying systemd restarts them
- Stripe checkout validated using test-mode keys

**Failure handling:**
- All failures found during validation must be fixed in this phase — not logged and deferred
- Scanner failures: debug and fix until all 5 scanners pass
- Email delivery: blocker — phase isn't done until emails actually arrive in inbox
- PDF report generation: fix if it fails (install fonts or whatever is needed)
- Stripe webhook: fix here if webhook doesn't fire correctly

**Pre-launch blockers:**
- Stripe account exists; configure test-mode keys (STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET) on production
- Liberation Sans fonts: not a pre-blocker, but install if PDF generation fails during validation
- Legal review of TOS/consent flow: deferred to next milestone (not needed for v1.1)

**Test targets:**
- Use a public intentionally-vulnerable test app (e.g., testphp.vulnweb.com or similar)
- Single URL target that triggers multiple scanner findings
- Scanners must return at least some findings to prove they work — zero findings is inconclusive
- Choose a target known to have security issues detectable by header, TLS, secrets, files, and vibe-code scanners

### Claude's Discretion

- Specific public test target selection (must produce findings)
- Order of validation steps
- How to verify email content correctness
- Exact systemd recovery test procedure

### Deferred Ideas (OUT OF SCOPE)

- Legal review of TOS/consent flow — deferred to next milestone (v1.2 or later)

</user_constraints>

## Standard Stack

### Core Validation Tools

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| **curl** | Latest | HTTP endpoint testing | Universal CLI tool for API validation |
| **systemctl** | systemd default | Service management validation | Built into systemd, standard service control |
| **journalctl** | systemd default | Log inspection | Built into systemd, standard log viewer |
| **Stripe CLI** | Latest | Webhook testing, event triggering | Official Stripe tool for local webhook testing |
| **Browser DevTools** | Latest | Frontend validation, network inspection | Standard web debugging tool |

### Supporting Components

| Component | Purpose | When to Use |
|-----------|---------|-------------|
| **testphp.vulnweb.com** | Intentionally vulnerable test target | Scanner validation (security headers, TLS, SQLi) |
| **Resend Dashboard** | Email delivery logs | Verify email sent, check delivery status |
| **Stripe Dashboard** | Payment event logs | Verify checkout sessions, webhook events |
| **ssh** | Remote server access | Execute validation commands on production |

### Installation

**Stripe CLI (local development machine for webhook testing):**
```bash
# macOS
brew install stripe/stripe-cli/stripe

# Linux
wget https://github.com/stripe/stripe-cli/releases/latest/download/stripe_linux_amd64.tar.gz
tar -xvf stripe_linux_amd64.tar.gz
sudo mv stripe /usr/local/bin/
```

**Liberation Sans fonts (on production server if PDF generation fails):**
```bash
# Ubuntu/Debian
sudo apt-get install fonts-liberation

# Or manual installation to project fonts/ directory
wget https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz
tar -xzf liberation-fonts-ttf-2.1.5.tar.gz
mkdir -p fonts
cp liberation-fonts-ttf-2.1.5/*.ttf fonts/
```

## Architecture Patterns

### Pattern 1: Staged Validation Pipeline

**What:** Execute validation in ordered stages, fixing blockers before proceeding

**When to use:** Always for production validation (prevents cascading failures)

**Validation sequence:**
```
Stage 1: Infrastructure (SSL, DNS, service health)
   ↓
Stage 2: Individual Scanners (test each scanner in isolation)
   ↓
Stage 3: Full Scan Pipeline (end-to-end free scan workflow)
   ↓
Stage 4: Payment Flow (Stripe checkout + webhook + PDF email)
   ↓
Stage 5: Resilience (systemd restart, container recovery)
```

**Example validation script structure:**
```bash
#!/bin/bash
set -e  # Exit on any failure

echo "=== Stage 1: Infrastructure Validation ==="
./validate_infrastructure.sh || exit 1

echo "=== Stage 2: Scanner Validation ==="
./validate_scanners.sh || exit 1

echo "=== Stage 3: Full Scan Pipeline ==="
./validate_scan_pipeline.sh || exit 1

echo "=== Stage 4: Payment Flow ==="
./validate_payment_flow.sh || exit 1

echo "=== Stage 5: Resilience ==="
./validate_resilience.sh || exit 1

echo "✅ All validation stages passed"
```

**Why staged:** Early stages validate prerequisites for later stages (e.g., can't validate scan pipeline if scanners don't work individually).

### Pattern 2: Critical Path Smoke Testing

**What:** Focus validation on highest-value user flows that represent real usage

**When to use:** Production deployments where comprehensive testing is impractical

**Critical paths for ShipSecure:**
1. **Free scan flow:** Submit URL → email arrives → click link → view results
2. **Paid audit flow:** Submit URL → checkout → payment → webhook fires → PDF email arrives
3. **Service recovery:** Container killed → systemd restarts → service healthy again

**Verification criteria:**
- Path must complete successfully end-to-end
- All external integrations must work (email, Stripe, database)
- User-visible outputs must be correct (email content, PDF formatting, results display)

**Example critical path validation:**
```bash
# Critical Path: Free Scan
echo "Step 1: Submit scan"
RESPONSE=$(curl -X POST https://shipsecure.ai/api/scans \
  -H "Content-Type: application/json" \
  -d '{"url": "http://testphp.vulnweb.com", "email": "test@example.com"}')

echo "Step 2: Wait for scan completion (max 5 minutes)"
# Poll until status=completed or timeout

echo "Step 3: Check inbox for email"
# Manual verification or use email API

echo "Step 4: Access results URL"
# Verify results page loads with findings
```

### Pattern 3: Email Deliverability Verification

**What:** Verify emails actually arrive in inbox, not just API success

**When to use:** Always when validating production email delivery

**Multi-layer verification:**
```
Layer 1: API Response (Resend returns 200 OK)
   ↓
Layer 2: Resend Dashboard (email shows as "Delivered")
   ↓
Layer 3: Actual Inbox (email visible in recipient inbox)
   ↓
Layer 4: Content Validation (subject, body, links all correct)
```

**Example verification procedure:**
```bash
# 1. Configure test email address
TEST_EMAIL="your-real-email@gmail.com"

# 2. Trigger scan that sends email
curl -X POST https://shipsecure.ai/api/scans \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"http://testphp.vulnweb.com\", \"email\": \"$TEST_EMAIL\"}"

# 3. Check Resend dashboard (https://resend.com/emails)
# Verify: Status = "Delivered", no bounce/complaint

# 4. Check actual inbox
# Manual step: Open email client, verify email received

# 5. Validate email content
# - Subject: "Scan Complete: [Grade] for http://testphp.vulnweb.com"
# - Body: Contains results link, expiration date, grade
# - Links: Click results link, verify page loads
```

**Common failure modes:**
- **API success but no email:** DNS records (SPF, DKIM, DMARC) not configured
- **Email in spam:** Domain reputation issue, missing authentication
- **Email delivered but links broken:** TRUSTEDGE_BASE_URL environment variable incorrect

### Pattern 4: Stripe Test Mode Validation

**What:** Validate payment flow using Stripe test keys without real transactions

**When to use:** Production environment validation before going live

**Configuration:**
```bash
# Production .env file with TEST keys
STRIPE_SECRET_KEY=sk_test_51ABC...  # Test mode secret key
STRIPE_WEBHOOK_SECRET=whsec_...     # Test mode webhook secret

# Note: Test keys start with sk_test_, live keys with sk_live_
```

**Test card numbers (Stripe test mode):**
```
Success: 4242 4242 4242 4242 (any future expiry, any CVC)
Decline: 4000 0000 0000 0002
Requires authentication: 4000 0025 0000 3155
```

**Validation procedure:**
```bash
# 1. Trigger checkout session
CHECKOUT_RESPONSE=$(curl -X POST https://shipsecure.ai/api/checkout \
  -H "Content-Type: application/json" \
  -d '{"url": "http://testphp.vulnweb.com", "email": "test@example.com"}')

CHECKOUT_URL=$(echo $CHECKOUT_RESPONSE | jq -r '.url')

# 2. Complete checkout in browser
# - Open $CHECKOUT_URL
# - Use test card: 4242 4242 4242 4242
# - Enter any future expiry, any CVC
# - Complete payment

# 3. Verify webhook fired (check backend logs)
ssh -p 2222 deploy@shipsecure.ai \
  'docker logs trustedge-backend-1 | grep "Webhook received"'

# 4. Check for PDF email in inbox
# Manual: Verify email arrived with PDF attachment

# 5. Validate PDF content
# - Download attachment
# - Open PDF, verify formatting, findings, branding
```

**Webhook testing with Stripe CLI:**
```bash
# Forward production webhooks to local endpoint for debugging
stripe listen --forward-to https://shipsecure.ai/api/webhooks/stripe

# Trigger test event
stripe trigger checkout.session.completed
```

### Pattern 5: Individual Scanner Validation

**What:** Test each scanner in isolation to prove it returns findings

**When to use:** Before testing full pipeline (isolates scanner-specific failures)

**Scanner validation matrix:**

| Scanner | Test Target | Expected Finding | How to Verify |
|---------|-------------|------------------|---------------|
| security_headers | testphp.vulnweb.com | Missing CSP, X-Frame-Options | Check findings for "Missing security header" |
| tls | testphp.vulnweb.com | Weak TLS config (if any) | Check findings for TLS issues or confirm modern config |
| js_secrets | testphp.vulnweb.com | Hardcoded strings in JS | Check findings for "Potential secret" or similar |
| exposed_files | testphp.vulnweb.com | Common files (.git, .env) | Check findings for exposed files |
| vibecode | testphp.vulnweb.com | Framework detection (PHP) | Check metadata for detected framework |

**Example scanner test:**
```bash
# SSH into production
ssh -p 2222 deploy@shipsecure.ai

# Test individual scanner via backend API (internal testing endpoint)
# Note: If no testing endpoint exists, trigger full scan and inspect findings

# Trigger scan and extract scanner-specific findings
curl -X POST http://localhost:3000/api/scans \
  -H "Content-Type: application/json" \
  -d '{"url": "http://testphp.vulnweb.com", "email": "test@example.com"}' \
  | jq -r '.scan_id'

# Wait for completion, then check findings grouped by scanner
# Manual inspection: Verify each scanner contributed findings
```

**Zero findings = inconclusive:**
If a scanner returns zero findings, it's unclear if the scanner works or the target has no issues. Use deliberately vulnerable targets to ensure scanners fire.

### Pattern 6: Systemd Service Recovery Validation

**What:** Simulate failures and verify systemd restarts services automatically

**When to use:** Production deployments with systemd supervision

**Test scenarios:**

**Scenario 1: Graceful service restart**
```bash
ssh -p 2222 deploy@shipsecure.ai

# Check current status
sudo systemctl status trustedge.service

# Restart service
sudo systemctl restart trustedge.service

# Verify restart succeeded
sudo systemctl status trustedge.service
# Should show "active (running)"

# Check container health
docker ps | grep trustedge
# Both backend and frontend containers should be up
```

**Scenario 2: Simulated container crash**
```bash
# Kill backend container directly
docker kill trustedge-backend-1

# Wait 5 seconds, verify systemd restarted it
sleep 5
docker ps | grep trustedge-backend-1
# Should be running with recent "Created" timestamp

# Check logs for restart
journalctl -u trustedge.service -n 50
# Should show restart activity
```

**Scenario 3: Full service stop/start**
```bash
# Stop service completely
sudo systemctl stop trustedge.service

# Verify containers are down
docker ps | grep trustedge
# Should show no results

# Start service
sudo systemctl start trustedge.service

# Verify containers are up and healthy
docker ps | grep trustedge
# Should show both containers running

# Test application health
curl http://localhost:3000/health
# Should return 200 OK
```

**Recovery verification criteria:**
- Service restarts within 10 seconds
- Containers come up with no errors in logs
- Application responds to health checks
- Database connections re-establish successfully

### Pattern 7: Test Target Selection for Scanner Coverage

**What:** Choose public test targets known to trigger multiple scanner findings

**When to use:** Scanner validation requires reliable detection

**Recommended targets:**

| Target | URL | Known Vulnerabilities | Scanner Coverage |
|--------|-----|----------------------|------------------|
| **Acunetix Test Site** | http://testphp.vulnweb.com | SQLi, XSS, weak headers | headers, TLS, files, secrets |
| **OWASP Juice Shop (public demo)** | https://juice-shop.herokuapp.com | OWASP Top 10 | headers, secrets, vibecode |
| **BadSSL** | https://expired.badssl.com | SSL/TLS issues | TLS scanner only |

**Primary recommendation: testphp.vulnweb.com**
- Hosted by Acunetix specifically for security scanner testing
- Free, public, legal to scan
- Known to have missing security headers, exposed files, weak configurations
- PHP-based (vibecode scanner can detect framework)
- Has been stable and available for years

**Alternative if testphp.vulnweb.com is unavailable:**
- Deploy OWASP Juice Shop locally on a public VPS
- Use http://<vps-ip>:3000 as scan target
- Docker: `docker run -d -p 3000:3000 bkimminich/juice-shop`

**DO NOT scan without permission:**
- Never scan production sites without authorization
- Never scan sites not explicitly designed for testing
- Stick to deliberately vulnerable test applications

### Anti-Patterns to Avoid

- **Skipping inbox verification:** Trusting Resend API 200 response without checking actual inbox. Emails can be sent but end up in spam or fail silently.
- **Using real Stripe keys in production validation:** Always use test-mode keys (sk_test_*) for validation. Switch to live keys only after full validation passes.
- **Assuming zero findings means scanner works:** Could indicate scanner failure, not secure target. Use deliberately vulnerable targets.
- **Testing scanners only via full pipeline:** Makes debugging hard. Test scanners individually first.
- **Validating only happy path:** Must also test failure modes (invalid URLs, Stripe decline cards, email bounces).
- **Not checking systemd logs after restart:** Container might restart but have errors. Always check `journalctl -u trustedge.service`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Email deliverability testing | Custom SMTP validation scripts | Resend Dashboard + manual inbox check | Dashboard shows delivery metrics, inbox check confirms user experience |
| Stripe webhook testing | Custom webhook simulation | Stripe CLI (`stripe listen`, `stripe trigger`) | Official tool handles signature generation, event format |
| Service health monitoring | Polling scripts in cron | systemd's built-in restart policies | systemd handles supervision, respects dependencies, manages resource limits |
| PDF validation | Parsing PDF bytes to check content | Manual visual inspection of generated PDF | PDF generation libraries are complex, visual inspection catches formatting issues |
| SSL certificate validation | Custom certificate parsing | Browser DevTools, `curl -v` | Built-in tools show full chain, expiry, cipher info |

**Key insight:** Production validation benefits from official tools (Stripe CLI, Resend Dashboard) and manual verification (inbox check, PDF inspection) over custom automation. The goal is to validate real user experience, not just API contracts.

## Common Pitfalls

### Pitfall 1: Email Delivery False Positives

**What goes wrong:** Resend API returns 200 OK, but email never arrives in inbox or lands in spam.

**Why it happens:** Email deliverability depends on domain reputation, DNS records (SPF, DKIM, DMARC), and recipient server spam filters. API success only means email was accepted for delivery.

**How to avoid:**
1. Verify DNS records before testing:
   - SPF: `dig TXT shipsecure.ai` should show SPF record
   - DKIM: Check Resend Dashboard for DKIM status
   - DMARC: `dig TXT _dmarc.shipsecure.ai`
2. Use your own email address for testing (real inbox you control)
3. Check spam folder if email doesn't arrive in inbox
4. Use Resend Dashboard to check delivery status (not just API response)

**Warning signs:**
- API returns 200 but Resend Dashboard shows "Bounced" or "Complaint"
- Email arrives but links don't work (TRUSTEDGE_BASE_URL misconfigured)
- Email in spam folder consistently

**Fix:**
- DNS records: Ensure SPF, DKIM, DMARC configured per Resend documentation
- Email content: Avoid spam trigger words, include unsubscribe link
- Domain reputation: Send from verified domain (shipsecure.ai), not generic domains

### Pitfall 2: Stripe Webhook Signature Verification Failures

**What goes wrong:** Webhook endpoint returns 400 or 500 errors, payment completes but backend doesn't process it.

**Why it happens:** STRIPE_WEBHOOK_SECRET doesn't match the webhook endpoint secret in Stripe Dashboard, or test/live mode mismatch.

**How to avoid:**
1. Use separate webhook endpoints for test and live mode
2. Get webhook secret from Stripe Dashboard > Webhooks > endpoint > "Signing secret"
3. Verify environment variable on production server:
   ```bash
   ssh -p 2222 deploy@shipsecure.ai
   grep STRIPE_WEBHOOK_SECRET /opt/trustedge/.env
   # Should match Stripe Dashboard signing secret
   ```
4. Check backend logs for signature verification errors:
   ```bash
   docker logs trustedge-backend-1 | grep -i "webhook\|stripe"
   ```

**Warning signs:**
- Stripe Dashboard shows webhook delivery attempts with 400/500 responses
- Payment succeeds but no PDF email sent
- Backend logs show "Invalid signature" or "Webhook verification failed"

**Fix:**
- Copy signing secret exactly from Stripe Dashboard (includes whsec_ prefix)
- Ensure production uses test-mode webhook secret during validation
- Use Stripe CLI to test locally: `stripe listen --forward-to https://shipsecure.ai/api/webhooks/stripe`

### Pitfall 3: Liberation Sans Font Missing on Production

**What goes wrong:** PDF generation fails with "Font error: Failed to load Liberation fonts" or produces PDFs with broken formatting.

**Why it happens:** genpdf library loads fonts from `fonts/` directory, but fonts weren't installed or directory doesn't exist in Docker container.

**How to avoid:**
1. Check if fonts directory exists and contains .ttf files:
   ```bash
   ssh -p 2222 deploy@shipsecure.ai
   docker exec trustedge-backend-1 ls -la fonts/
   # Should show LiberationSans-*.ttf files
   ```
2. If missing, install fonts on host then rebuild Docker image:
   ```bash
   # On production host
   mkdir -p /opt/trustedge/fonts
   wget https://github.com/liberationfonts/liberation-fonts/files/7261482/liberation-fonts-ttf-2.1.5.tar.gz
   tar -xzf liberation-fonts-ttf-2.1.5.tar.gz
   cp liberation-fonts-ttf-2.1.5/*.ttf /opt/trustedge/fonts/

   # Rebuild and restart
   cd /opt/trustedge
   docker compose -f docker-compose.yml -f docker-compose.prod.yml build backend
   docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```
3. Alternative: Modify Dockerfile to include fonts in image build

**Warning signs:**
- Paid audit completes but no email sent
- Backend logs show PDF generation errors
- Test PDF generation fails with font errors

**Fix (if Dockerfile includes COPY fonts/):**
- Ensure fonts/ directory exists in repo before building Docker image
- Commit fonts to git or download during Docker build

**Fix (if fonts/ mounted as volume):**
- Ensure fonts directory on production host has .ttf files
- Update docker-compose.prod.yml to mount fonts: `- ./fonts:/app/fonts:ro`

### Pitfall 4: Test Target Returns Zero Findings

**What goes wrong:** Scan completes successfully but all scanners return zero findings, unclear if scanners work.

**Why it happens:** Test target may be too secure, scanner templates out of date, or scanners silently failing.

**How to avoid:**
1. Use deliberately vulnerable targets (testphp.vulnweb.com, OWASP Juice Shop)
2. Validate scanners individually, not just full pipeline
3. Check backend logs for scanner errors:
   ```bash
   docker logs trustedge-backend-1 | grep -i "scanner\|nuclei\|testssl"
   ```
4. Verify scanner binaries exist and are executable:
   ```bash
   docker exec trustedge-backend-1 which nuclei
   docker exec trustedge-backend-1 nuclei -version
   ```

**Warning signs:**
- All scanners return zero findings for known-vulnerable target
- Logs show "Scanner skipped" or "Binary not found"
- Scan completes in unusually short time (<30 seconds for full scan)

**Fix:**
- Install missing scanner binaries (Nuclei, testssl.sh)
- Update Nuclei templates: `nuclei -update-templates`
- Test scanners individually with verbose logging
- Use different test target known to have findings

### Pitfall 5: Systemd Restart Loops

**What goes wrong:** Service crashes, systemd restarts it, it crashes again, infinite loop.

**Why it happens:** Application has a startup failure (bad config, database unreachable), systemd keeps trying to restart.

**How to avoid:**
1. Configure systemd restart limits in service unit:
   ```ini
   [Service]
   Restart=on-failure
   RestartSec=10
   StartLimitIntervalSec=300
   StartLimitBurst=5
   ```
2. Check logs before testing restart:
   ```bash
   journalctl -u trustedge.service -n 100
   # Should show healthy startup, no errors
   ```
3. Test application health before triggering restart test:
   ```bash
   curl http://localhost:3000/health
   # Should return 200 OK
   ```

**Warning signs:**
- `systemctl status trustedge.service` shows "activating (auto-restart)"
- Logs show repeated startup errors
- Containers keep restarting every few seconds

**Fix:**
- Identify root cause: check container logs for startup errors
- Fix configuration issue (DATABASE_URL, environment variables)
- If restart limit hit, reset with: `systemctl reset-failed trustedge.service`
- After fixing, restart manually: `systemctl restart trustedge.service`

### Pitfall 6: CORS Issues on Production Frontend

**What goes wrong:** Frontend can't reach backend API, browser console shows CORS errors.

**Why it happens:** NEXT_PUBLIC_BACKEND_URL points to localhost instead of production domain, or Nginx doesn't proxy /api/ correctly.

**How to avoid:**
1. Verify environment variables on production:
   ```bash
   docker exec trustedge-frontend-1 env | grep BACKEND
   # NEXT_PUBLIC_BACKEND_URL should be https://shipsecure.ai
   ```
2. Test API endpoint directly:
   ```bash
   curl https://shipsecure.ai/api/health
   # Should return 200 OK
   ```
3. Check Nginx configuration:
   ```bash
   sudo nginx -t
   sudo cat /etc/nginx/sites-enabled/shipsecure.ai
   # Verify location /api/ proxies to http://127.0.0.1:3000
   ```

**Warning signs:**
- Frontend loads but scan submission fails
- Browser DevTools shows "CORS policy" errors
- API requests go to localhost instead of domain

**Fix:**
- Update production .env: `NEXT_PUBLIC_BACKEND_URL=https://shipsecure.ai`
- Rebuild frontend: `docker compose -f docker-compose.yml -f docker-compose.prod.yml build frontend`
- Restart: `docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d`

## Code Examples

### Complete Validation Checklist Script

```bash
#!/bin/bash
# production-validation.sh
# Validates ShipSecure production deployment

set -e
TARGET_URL="http://testphp.vulnweb.com"
TEST_EMAIL="your-email@example.com"  # CHANGE THIS
PROD_DOMAIN="shipsecure.ai"

echo "=========================================="
echo "ShipSecure Production Validation"
echo "=========================================="

# Stage 1: Infrastructure
echo ""
echo "=== Stage 1: Infrastructure ==="

echo "1.1 Testing HTTPS and SSL certificate..."
curl -sS -o /dev/null -w "%{http_code}" https://$PROD_DOMAIN | grep -q "200" \
  && echo "✅ HTTPS accessible" \
  || (echo "❌ HTTPS failed" && exit 1)

openssl s_client -connect $PROD_DOMAIN:443 -servername $PROD_DOMAIN < /dev/null 2>/dev/null \
  | openssl x509 -noout -dates \
  && echo "✅ SSL certificate valid" \
  || (echo "❌ SSL certificate invalid" && exit 1)

echo "1.2 Testing backend health endpoint..."
ssh -p 2222 deploy@$PROD_DOMAIN 'curl -s http://localhost:3000/health' | grep -q "ok" \
  && echo "✅ Backend health check passed" \
  || (echo "❌ Backend health check failed" && exit 1)

echo "1.3 Testing frontend accessibility..."
curl -sS -o /dev/null -w "%{http_code}" https://$PROD_DOMAIN | grep -q "200" \
  && echo "✅ Frontend accessible" \
  || (echo "❌ Frontend failed" && exit 1)

# Stage 2: Scanner Validation
echo ""
echo "=== Stage 2: Scanner Validation ==="

echo "2.1 Verifying Nuclei installation..."
ssh -p 2222 deploy@$PROD_DOMAIN 'docker exec trustedge-backend-1 nuclei -version' \
  && echo "✅ Nuclei installed" \
  || (echo "❌ Nuclei not found" && exit 1)

echo "2.2 Submitting test scan to validate all scanners..."
echo "   Target: $TARGET_URL"
echo "   Email: $TEST_EMAIL"

SCAN_RESPONSE=$(curl -sS -X POST https://$PROD_DOMAIN/api/scans \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"$TARGET_URL\", \"email\": \"$TEST_EMAIL\"}")

SCAN_ID=$(echo $SCAN_RESPONSE | jq -r '.scan_id')
echo "   Scan ID: $SCAN_ID"

echo "2.3 Waiting for scan completion (max 5 minutes)..."
for i in {1..60}; do
  sleep 5
  STATUS=$(curl -sS "https://$PROD_DOMAIN/api/scans/$SCAN_ID" | jq -r '.status')
  if [ "$STATUS" = "completed" ]; then
    echo "✅ Scan completed successfully"
    break
  elif [ "$STATUS" = "failed" ]; then
    echo "❌ Scan failed"
    exit 1
  fi
  echo "   Status: $STATUS (${i}0s elapsed)"
done

# Stage 3: Email Delivery
echo ""
echo "=== Stage 3: Email Delivery Validation ==="

echo "3.1 Checking Resend API configuration..."
ssh -p 2222 deploy@$PROD_DOMAIN 'grep -q RESEND_API_KEY /opt/trustedge/.env' \
  && echo "✅ RESEND_API_KEY configured" \
  || (echo "❌ RESEND_API_KEY missing" && exit 1)

echo "3.2 MANUAL STEP: Check your inbox ($TEST_EMAIL)"
echo "   Expected subject: Scan Complete: [GRADE] for $TARGET_URL"
echo "   Expected content: Results link, grade, findings summary"
echo ""
read -p "   Did email arrive in inbox? (y/n) " EMAIL_RECEIVED
if [ "$EMAIL_RECEIVED" != "y" ]; then
  echo "❌ Email delivery failed"
  echo "   Check Resend Dashboard: https://resend.com/emails"
  echo "   Check spam folder"
  exit 1
fi
echo "✅ Email delivery confirmed"

echo "3.3 MANUAL STEP: Click results link in email"
read -p "   Did results page load correctly? (y/n) " RESULTS_PAGE
if [ "$RESULTS_PAGE" != "y" ]; then
  echo "❌ Results page failed"
  exit 1
fi
echo "✅ Results page validated"

# Stage 4: Payment Flow (Stripe Test Mode)
echo ""
echo "=== Stage 4: Payment Flow Validation ==="

echo "4.1 Verifying Stripe test keys configured..."
ssh -p 2222 deploy@$PROD_DOMAIN 'grep "STRIPE_SECRET_KEY=sk_test" /opt/trustedge/.env' \
  && echo "✅ Stripe test key configured" \
  || (echo "⚠️  WARNING: Not using Stripe test key!" && exit 1)

echo "4.2 Creating checkout session..."
CHECKOUT_RESPONSE=$(curl -sS -X POST https://$PROD_DOMAIN/api/checkout \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"$TARGET_URL\", \"email\": \"$TEST_EMAIL\"}")

CHECKOUT_URL=$(echo $CHECKOUT_RESPONSE | jq -r '.url')
echo "   Checkout URL: $CHECKOUT_URL"

echo "4.3 MANUAL STEP: Complete Stripe checkout"
echo "   1. Open: $CHECKOUT_URL"
echo "   2. Use test card: 4242 4242 4242 4242"
echo "   3. Expiry: Any future date (e.g., 12/30)"
echo "   4. CVC: Any 3 digits (e.g., 123)"
echo "   5. Complete payment"
echo ""
read -p "   Did payment complete successfully? (y/n) " PAYMENT_SUCCESS
if [ "$PAYMENT_SUCCESS" != "y" ]; then
  echo "❌ Payment failed"
  exit 1
fi
echo "✅ Payment completed"

echo "4.4 Verifying webhook processing..."
sleep 5
ssh -p 2222 deploy@$PROD_DOMAIN \
  'docker logs trustedge-backend-1 --tail 100 | grep -i "webhook"' \
  && echo "✅ Webhook processed" \
  || (echo "❌ Webhook not processed" && exit 1)

echo "4.5 MANUAL STEP: Check for PDF email"
read -p "   Did PDF email arrive? (y/n) " PDF_EMAIL
if [ "$PDF_EMAIL" != "y" ]; then
  echo "❌ PDF email failed"
  exit 1
fi
echo "✅ PDF email received"

echo "4.6 MANUAL STEP: Validate PDF content"
echo "   - Download PDF attachment"
echo "   - Open PDF, check for:"
echo "     * ShipSecure branding"
echo "     * Target URL: $TARGET_URL"
echo "     * Security findings listed"
echo "     * Professional formatting"
read -p "   Is PDF formatted correctly? (y/n) " PDF_VALID
if [ "$PDF_VALID" != "y" ]; then
  echo "❌ PDF validation failed"
  exit 1
fi
echo "✅ PDF validated"

# Stage 5: Resilience
echo ""
echo "=== Stage 5: Resilience Validation ==="

echo "5.1 Testing systemd service restart..."
ssh -p 2222 deploy@$PROD_DOMAIN 'sudo systemctl restart trustedge.service'
sleep 10

ssh -p 2222 deploy@$PROD_DOMAIN \
  'systemctl is-active trustedge.service' | grep -q "active" \
  && echo "✅ Service restarted successfully" \
  || (echo "❌ Service restart failed" && exit 1)

echo "5.2 Testing container recovery after crash..."
ssh -p 2222 deploy@$PROD_DOMAIN 'docker kill trustedge-backend-1'
sleep 10

ssh -p 2222 deploy@$PROD_DOMAIN \
  'docker ps | grep trustedge-backend-1' \
  && echo "✅ Backend container recovered" \
  || (echo "❌ Backend container did not restart" && exit 1)

echo "5.3 Verifying application health after recovery..."
ssh -p 2222 deploy@$PROD_DOMAIN 'curl -s http://localhost:3000/health' | grep -q "ok" \
  && echo "✅ Application healthy after recovery" \
  || (echo "❌ Application unhealthy" && exit 1)

# Summary
echo ""
echo "=========================================="
echo "✅ ALL VALIDATION STAGES PASSED"
echo "=========================================="
echo ""
echo "Production deployment validated successfully!"
echo ""
echo "Next steps:"
echo "1. Switch Stripe to live keys (sk_live_*) when ready for real customers"
echo "2. Monitor logs for first 24 hours: docker logs -f trustedge-backend-1"
echo "3. Set up alerts for errors (future phase)"
```

### Individual Scanner Test Script

```bash
#!/bin/bash
# test-individual-scanners.sh
# Tests each scanner in isolation

TARGET="http://testphp.vulnweb.com"
BACKEND_HOST="localhost:3000"

echo "Testing individual scanners against $TARGET"
echo ""

# Note: This assumes backend has a testing endpoint or you inspect full scan results
# In production, trigger full scan and inspect findings by scanner type

echo "Triggering full scan..."
RESPONSE=$(curl -sS -X POST http://$BACKEND_HOST/api/scans \
  -H "Content-Type: application/json" \
  -d "{\"url\": \"$TARGET\", \"email\": \"test@example.com\"}")

SCAN_ID=$(echo $RESPONSE | jq -r '.scan_id')
echo "Scan ID: $SCAN_ID"

echo "Waiting for completion..."
while true; do
  sleep 5
  STATUS=$(curl -sS "http://$BACKEND_HOST/api/scans/$SCAN_ID" | jq -r '.status')
  if [ "$STATUS" = "completed" ]; then
    break
  elif [ "$STATUS" = "failed" ]; then
    echo "Scan failed!"
    exit 1
  fi
done

echo ""
echo "Analyzing findings by scanner..."

# Get findings and group by scanner type
FINDINGS=$(curl -sS "http://$BACKEND_HOST/api/scans/$SCAN_ID" | jq -r '.findings')

# Check each scanner
for SCANNER in security_headers tls js_secrets exposed_files vibecode; do
  COUNT=$(echo $FINDINGS | jq "[.[] | select(.scanner_type == \"$SCANNER\")] | length")
  if [ "$COUNT" -gt 0 ]; then
    echo "✅ $SCANNER: $COUNT findings"
  else
    echo "❌ $SCANNER: 0 findings (may indicate scanner issue)"
  fi
done
```

### Email Content Validation

```bash
#!/bin/bash
# validate-email-content.sh
# Validates email content structure

EMAIL_HTML=$(cat << 'EOF'
<!-- Expected email structure -->
<html>
  <body>
    <h1>Scan Complete</h1>
    <p>Grade: <strong>{GRADE}</strong></p>
    <p>Target: {URL}</p>
    <p>Findings Summary:</p>
    <ul>
      <li>Critical: {COUNT}</li>
      <li>High: {COUNT}</li>
      <li>Medium: {COUNT}</li>
      <li>Low: {COUNT}</li>
    </ul>
    <a href="{RESULTS_URL}">View Full Report</a>
    <p>Results expire: {EXPIRY_DATE}</p>
  </body>
</html>
EOF
)

echo "Expected email content structure:"
echo "$EMAIL_HTML"
echo ""
echo "Manual validation checklist:"
echo "[ ] Subject contains grade (A/B/C/D/F)"
echo "[ ] Subject contains target URL"
echo "[ ] Body shows findings summary with counts"
echo "[ ] Results link is clickable and valid"
echo "[ ] Expiration date is 3 days from scan date"
echo "[ ] ShipSecure branding present"
echo "[ ] From address: ShipSecure <scans@shipsecure.ai>"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual testing via browser only | Automated smoke test scripts + manual validation | 2020+ | Faster validation, reproducible tests |
| Testing email via logs | Resend Dashboard + inbox verification | 2022+ | Real deliverability validation |
| Production testing with real Stripe keys | Test mode keys in production validation | Always | Prevents accidental charges |
| Comprehensive end-to-end tests | Critical path smoke tests | 2023+ | Faster feedback, focus on user-visible issues |
| Custom health check scripts | systemd built-in restart policies | systemd adoption | Native supervision, less custom code |

**Current trends (2026):**
- **AI-assisted smoke testing:** Tools suggest critical paths based on code changes (not yet widely adopted)
- **Observability-first validation:** Structured logging, distributed tracing even in smoke tests
- **Infrastructure as Code validation:** Validate infrastructure changes (Terraform, Ansible) before app validation

## Open Questions

1. **Alternative test targets if testphp.vulnweb.com becomes unavailable**
   - What we know: testphp.vulnweb.com has been stable for years, but is single point of failure
   - What's unclear: Long-term availability, whether Acunetix will maintain it
   - Recommendation: Document fallback options (deploy OWASP Juice Shop on VPS, use badssl.com for TLS testing)

2. **Optimal scanner timeout values for production**
   - What we know: Scanners currently have default timeouts
   - What's unclear: Whether production latency requires timeout adjustments
   - Recommendation: Monitor first 10 production scans, adjust timeouts if needed

3. **Email deliverability monitoring long-term**
   - What we know: Manual inbox check works for validation
   - What's unclear: How to monitor deliverability after launch (bounce rates, spam reports)
   - Recommendation: Set up Resend webhook for bounce/complaint events (future phase)

4. **Stripe webhook retry behavior**
   - What we know: Stripe retries failed webhooks automatically
   - What's unclear: Exact retry schedule, how to handle persistent failures
   - Recommendation: Test webhook failure scenario (backend down during payment) to verify retry works

5. **PDF generation performance at scale**
   - What we know: PDF generation works for individual reports
   - What's unclear: Performance impact if multiple PDF generations happen concurrently
   - Recommendation: Load test if paid audits exceed 10/hour

## Sources

### Primary (HIGH confidence)

- [Stripe Webhooks Testing Documentation](https://stripe.com/docs/webhooks/test) - Official Stripe webhook testing guide
- [Resend Send Test Emails](https://resend.com/docs/dashboard/emails/send-test-emails) - Official Resend testing documentation
- [genpdf Fonts Documentation](https://docs.rs/genpdf/latest/genpdf/fonts/index.html) - Rust genpdf library font loading
- [systemd Service Management](https://www.freedesktop.org/software/systemd/man/systemctl.html) - Official systemd documentation
- [Liberation Fonts GitHub](https://github.com/liberationfonts/liberation-fonts) - Official Liberation fonts repository

### Secondary (MEDIUM confidence)

- [Smoke Testing in 2026: Essential QA Guide](https://blog.qasource.com/a-complete-guide-to-smoke-testing-in-software-qa) - Smoke testing best practices
- [BrowserStack Smoke Testing Guide](https://www.browserstack.com/guide/smoke-testing) - Critical path testing patterns
- [Best Practices for Testing Stripe Webhooks](https://launchdarkly.com/blog/best-practices-for-testing-stripe-webhook-event-processing/) - Webhook validation strategies
- [Mailtrap Email Deliverability Testing](https://mailtrap.io/blog/test-email-deliverability/) - Email verification techniques
- [OWASP Juice Shop](https://owasp.org/www-project-juice-shop/) - Deliberately vulnerable test application
- [Acunetix Test Site](http://testphp.vulnweb.com/) - Intentionally vulnerable PHP application
- [Implementing Service Recovery with systemd](https://dohost.us/index.php/2025/10/27/implementing-service-recovery-and-restart-policies-in-systemd/) - systemd restart policies
- [Docker Start Containers Automatically](https://docs.docker.com/engine/containers/start-containers-automatically/) - Docker restart policies vs systemd
- [How to Test Stripe Webhooks (Complete Guide)](https://www.webhookdebugger.com/blog/how-to-test-stripe-webhooks-complete-guide) - Webhook debugging techniques

### Tertiary (LOW confidence)

- [Nuclei with OWASP Juice Shop](https://medium.com/cyberscribers-exploring-cybersecurity/nuclei-a-vulnerability-scanner-and-owasp-juice-shop-e324867a276e) - Nuclei testing with vulnerable apps (Medium article, not official)
- [Liberation Sans Font Installation](https://github.com/shantigilbert/liberation-fonts-ttf) - Community font installation guide

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Well-established tools (curl, systemctl, Stripe CLI)
- Architecture patterns: MEDIUM-HIGH - Smoke testing patterns are proven, but ShipSecure-specific validation is custom
- Pitfalls: MEDIUM - Based on common production issues and official documentation warnings
- Test targets: MEDIUM - testphp.vulnweb.com is established but not officially guaranteed available

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - production validation patterns stable, but check Stripe/Resend API changes)

**Key verifications performed:**
- ✅ Verified Stripe test mode webhook testing approach via official docs
- ✅ Confirmed Resend email deliverability verification methods
- ✅ Verified genpdf font loading requirements via Rust docs
- ✅ Confirmed testphp.vulnweb.com availability and purpose
- ✅ Verified systemd service restart testing patterns
- ⚠️ testphp.vulnweb.com long-term availability not officially guaranteed
- ⚠️ Scanner-specific validation approach is custom (no official testing framework)
