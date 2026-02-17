// Stripe test card: 4242 4242 4242 4242 (not used in E2E — all Stripe responses are mocked)
// Stripe test mode is verified via cs_test_ prefix in checkout URL assertion (E2E-05)

import { test, expect } from 'next/experimental/testmode/playwright';
import { resultsFixtures } from './fixtures/results';
import { checkoutFixtures } from './fixtures/checkout';
import { mockResultsPage } from './helpers/fetch-mocks';

test.describe('Paid Audit Flow', () => {
  test('UpgradeCTA triggers Stripe Checkout redirect', async ({ page, next }) => {
    // Setup server-side mock for results page
    mockResultsPage(next, resultsFixtures.freeGradeB);

    // Mock client-side /api/v1/checkout POST with 200ms delay
    await page.route('**/api/v1/checkout', async (route) => {
      if (route.request().method() !== 'POST') {
        await route.continue();
        return;
      }
      await new Promise((r) => setTimeout(r, 200));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(checkoutFixtures.success),
      });
    });

    // Mock Stripe redirect: intercept the Stripe Checkout URL and assert cs_test_ pattern (E2E-05)
    await page.route('https://checkout.stripe.com/**', async (route) => {
      // Assert that the checkout URL targets Stripe test mode
      expect(route.request().url()).toContain('cs_test_');

      // Redirect to payment success page instead of actually navigating to Stripe
      await route.fulfill({
        status: 302,
        headers: {
          Location: 'http://localhost:3000/payment/success?session_id=mock_123',
        },
        body: '',
      });
    });

    // 1. Navigate to results page
    await page.goto('/results/tok_e2e_abc123');

    // 2. Verify UpgradeCTA is visible
    await expect(page.locator('text=Upgrade for $49')).toBeVisible();

    // 3. Click the upgrade button
    await page.click('text=Upgrade for $49');

    // 4. Wait for button to show loading state
    await expect(page.locator('text=Redirecting to checkout...')).toBeVisible();

    // 5. Checkout mock returns Stripe URL → window.location.href = data.checkout_url
    //    Stripe route mock intercepts, asserts cs_test_, redirects to success page

    // 6. Verify the payment success page
    await expect(page.locator('text=Payment Successful!')).toBeVisible({ timeout: 10000 });
  });

  test('payment success page renders correctly', async ({ page }) => {
    // 1. Navigate directly to payment success page
    await page.goto('/payment/success?session_id=mock_123');

    // 2. Verify success heading
    await expect(page.locator('text=Payment Successful!')).toBeVisible();

    // 3. Verify processing message
    await expect(page.locator('text=deep security audit is now processing')).toBeVisible();

    // 4. Verify return link
    await expect(page.locator('text=Return to Home')).toBeVisible();
  });

  test('paid tier results show enhanced content', async ({ page, next }) => {
    // Setup server-side mock for paid tier results
    mockResultsPage(next, resultsFixtures.paidGradeA);

    // 1. Navigate to results page
    await page.goto('/results/tok_e2e_abc123');

    // 2. Verify grade A is shown
    await expect(page.locator('text=A')).toBeVisible();

    // 3. Verify UpgradeCTA is NOT visible (tier is 'paid')
    await expect(page.locator('text=Upgrade to Deep Audit')).not.toBeVisible();

    // 4. Verify the Download Markdown Report link exists (available for all tiers)
    await expect(page.locator('text=Download Markdown Report')).toBeVisible();
  });

  test('payment cancel return is handled gracefully', async ({ page, next }) => {
    // Stripe cancel_url returns user to the results page they came from
    // Setup server-side mock for results page
    mockResultsPage(next, resultsFixtures.freeGradeB);

    // 1. Navigate to results page (simulating Stripe cancel redirect back)
    await page.goto('/results/tok_e2e_abc123');

    // 2. Verify results page renders normally — not an error page
    await expect(page.locator('h1')).toContainText('Security Scan Results');

    // 3. Verify UpgradeCTA is still available (user can try again)
    await expect(page.locator('text=Upgrade for $49')).toBeVisible();

    // 4. Verify the user can navigate — "Scan again" link is visible
    await expect(page.locator('text=Scan again')).toBeVisible();
  });
});
