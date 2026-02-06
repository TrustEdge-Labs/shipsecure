# Phase 4: User Setup Guide

This phase requires manual configuration of external services. Complete these steps before testing payment flows.

---

## Required Services

### Stripe (Payment Processing)

**Why:** Process paid audit payments ($49 one-time purchases)

#### 1. Create Stripe Account
1. Sign up at https://stripe.com
2. Complete account verification (if required for your region)
3. Switch to Test Mode for development (toggle in top-right of dashboard)

#### 2. Get API Keys

**Secret Key** (for backend API calls):
1. Go to: Stripe Dashboard → Developers → API keys
2. Copy the "Secret key" (starts with `sk_test_` in test mode)
3. Add to your `.env` file:
   ```bash
   STRIPE_SECRET_KEY=sk_test_xxxxxxxxxxxxxxxxxxxxx
   ```

#### 3. Configure Webhook

**Create Webhook Endpoint:**
1. Go to: Stripe Dashboard → Developers → Webhooks
2. Click "Add endpoint"
3. Enter your backend URL + webhook path:
   - **Development:** `http://localhost:3000/api/v1/webhooks/stripe`
   - **Production:** `https://your-backend-domain.com/api/v1/webhooks/stripe`
4. Click "Select events"
5. Search for and select: `checkout.session.completed`
6. Click "Add endpoint"

**Get Webhook Secret:**
1. Click on the newly created webhook endpoint
2. In the "Signing secret" section, click "Reveal"
3. Copy the signing secret (starts with `whsec_`)
4. Add to your `.env` file:
   ```bash
   STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxxxxxxxxxxx
   ```

#### 4. Set Frontend URL (Optional)

If your frontend runs on a port other than 3001, set:
```bash
FRONTEND_URL=http://localhost:YOUR_PORT
```

This is used for Stripe checkout redirect URLs (success and cancel).

---

## Verification

### 1. Check Environment Variables

Ensure your `.env` file has:
```bash
STRIPE_SECRET_KEY=sk_test_xxxxxxxxxxxxxxxxxxxxx
STRIPE_WEBHOOK_SECRET=whsec_xxxxxxxxxxxxxxxxxxxxx
FRONTEND_URL=http://localhost:3001  # Optional, defaults to this
```

### 2. Test Checkout Endpoint

Start your backend:
```bash
cargo run
```

In another terminal, test checkout creation (replace SCAN_ID with a real completed scan UUID):
```bash
curl -X POST http://localhost:3000/api/v1/checkout \
  -H "Content-Type: application/json" \
  -d '{"scan_id": "YOUR_SCAN_ID_HERE"}'
```

Expected response:
```json
{
  "checkout_url": "https://checkout.stripe.com/c/pay/cs_test_..."
}
```

### 3. Test Webhook (Using Stripe CLI)

Install Stripe CLI: https://stripe.com/docs/stripe-cli

Forward webhooks to local backend:
```bash
stripe listen --forward-to http://localhost:3000/api/v1/webhooks/stripe
```

In another terminal, trigger a test event:
```bash
stripe trigger checkout.session.completed
```

Check your backend logs for:
```
Processing checkout.session.completed for scan_id=...
Paid scan triggered for scan_id=...
```

### 4. Test Full Flow (Manual)

1. Create a scan via POST /api/v1/scans
2. Wait for scan to complete
3. Call POST /api/v1/checkout with scan_id
4. Open the returned checkout_url in a browser
5. Use Stripe test card: `4242 4242 4242 4242`, any future expiry, any CVC
6. Complete payment
7. Check backend logs for webhook processing
8. Verify in database:
   ```sql
   SELECT * FROM paid_audits WHERE scan_id = 'YOUR_SCAN_ID';
   -- Should show status = 'completed'

   SELECT * FROM scans WHERE id = 'YOUR_SCAN_ID';
   -- Should show tier = 'paid'
   ```

---

## Troubleshooting

### "Stripe not configured" error
- Ensure STRIPE_SECRET_KEY is set in .env
- Restart backend after adding environment variables

### "Invalid webhook signature" error
- Check STRIPE_WEBHOOK_SECRET matches the webhook endpoint in Stripe Dashboard
- Ensure webhook endpoint URL is correct
- Verify timestamp tolerance (5 minutes) - check server clock if failing

### Webhook not firing
- Confirm webhook endpoint is publicly accessible (use ngrok for local development)
- Verify `checkout.session.completed` event is selected in Stripe Dashboard
- Check Stripe Dashboard → Developers → Webhooks → your endpoint → "Events" tab for delivery attempts

### Duplicate webhook warnings
- This is normal! Stripe retries webhooks if it doesn't receive 200 OK
- The idempotency check (stripe_events table) prevents duplicate processing
- If you see "Duplicate webhook event XXX ignored" in logs, everything is working correctly

---

## Production Checklist

Before deploying to production:

- [ ] Switch to Stripe Live Mode keys (sk_live_xxx, whsec_xxx)
- [ ] Update webhook endpoint URL to production backend domain
- [ ] Verify webhook events are being delivered (check Stripe Dashboard)
- [ ] Test a real $49 payment (or use Stripe test mode until ready for real charges)
- [ ] Set up Stripe webhook endpoint monitoring/alerting
- [ ] Review Stripe Dashboard for failed webhook deliveries

---

**Next:** Once Stripe is configured, the payment backend is ready. Plan 04-03 will wire the paid scan triggering to the orchestrator.
