/**
 * Reusable page.route() interceptors for E2E tests.
 * These intercept client-side fetch calls made directly from the browser.
 */

import type { Page } from '@playwright/test';
import type { ScanFixtures } from '../fixtures/scan';

/**
 * Sets up a stateful polling interceptor on scan status endpoints.
 * Returns inProgress for the first few calls, then completed.
 * Adds 200ms delay per mocking decision to simulate real timing.
 */
export async function mockScanPolling(
  page: Page,
  fixtures: ScanFixtures
): Promise<void> {
  let callCount = 0;
  const pendingThreshold = 2;

  await page.route('**/api/v1/scans/**', async (route) => {
    await new Promise((r) => setTimeout(r, 200));
    callCount++;

    let responseBody;
    if (callCount <= pendingThreshold) {
      responseBody = fixtures.inProgress;
    } else {
      responseBody = fixtures.completed;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(responseBody),
    });
  });
}

/**
 * Intercepts checkout POST requests and returns a checkout URL.
 * Also intercepts Stripe checkout redirect with a 302 to a success page.
 * Adds 150ms delay.
 */
export async function mockCheckout(
  page: Page,
  checkoutUrl: string
): Promise<void> {
  await page.route('**/api/v1/checkout', async (route) => {
    if (route.request().method() !== 'POST') {
      await route.continue();
      return;
    }
    await new Promise((r) => setTimeout(r, 150));
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ checkout_url: checkoutUrl }),
    });
  });

  await page.route('https://checkout.stripe.com/**', async (route) => {
    await route.fulfill({
      status: 302,
      headers: {
        Location: 'http://localhost:3000/payment/success?session_id=mock_session_123',
      },
      body: '',
    });
  });
}

/**
 * Aborts all requests matching the given URL pattern with a network failure.
 * Use this to simulate network errors and timeouts.
 */
export async function mockNetworkFailure(
  page: Page,
  urlPattern: string | RegExp
): Promise<void> {
  await page.route(urlPattern, async (route) => {
    await route.abort('failed');
  });
}
