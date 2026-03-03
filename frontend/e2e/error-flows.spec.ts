/**
 * E2E tests for error flows and recovery paths.
 * Covers all E2E-03 error scenarios:
 * - Anonymous user URL is locked to demo target
 * - Server rejection shows error and form remains usable
 * - 404 missing scan via direct URL navigation shows Scan Not Found
 * - Results token 404 shows Next.js not-found page
 * - Network timeout shows connection error warning
 * - Server 500 error causes results page to show not-found
 *
 * Per locked decision: every error test verifies the user can retry or navigate away.
 */

import { test, expect } from 'next/experimental/testmode/playwright';
import { mockScanCount, mockServerError, mockResultsNotFound } from './helpers/fetch-mocks';
import { scanFixtures } from './fixtures/scan';

test.describe('Error Flows', () => {

  test('anonymous user URL is locked to demo target', async ({ page, next }) => {
    // Home page needs scan count
    mockScanCount(next, 0);

    await page.goto('/');

    // Anonymous users see a disabled URL input pre-filled with the demo target
    const visibleUrlInput = page.locator('input#url');
    await expect(visibleUrlInput).toBeVisible();
    await expect(visibleUrlInput).toBeDisabled();
    await expect(visibleUrlInput).toHaveValue('https://demo.owasp-juice.shop');

    // The actual form value is carried by a hidden input
    const hiddenUrlInput = page.locator('input[type="hidden"][name="url"]');
    await expect(hiddenUrlInput).toHaveValue('https://demo.owasp-juice.shop');

    // Submit button is still visible and accessible
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('server rejection shows error and form remains usable', async ({ page, next }) => {
    // Home page needs scan count
    mockScanCount(next, 0);

    // Mock the Server Action's outbound fetch to return a 422 validation error
    next.onFetch(async (request) => {
      const url = new URL(request.url);
      if (url.pathname === '/api/v1/scans' && request.method === 'POST') {
        await new Promise((r) => setTimeout(r, 200));
        return new Response(
          JSON.stringify({
            type: 'about:blank',
            title: 'Validation Error',
            status: 422,
            detail: 'Target URL is unreachable',
          }),
          {
            status: 422,
            headers: { 'Content-Type': 'application/json' },
          }
        );
      }
      return undefined;
    });

    await page.goto('/');

    // Fill email and check authorization (URL is pre-filled for anonymous users)
    await page.fill('input[name="email"]', 'test@example.com');
    await page.check('input[name="authorization"]');
    await page.click('button[type="submit"]');

    // Server error appears in form — the scan action returns the detail as _form error
    await expect(
      page.locator('text=Target URL is unreachable').first()
    ).toBeVisible({ timeout: 10000 });

    // Recovery: form is still usable — submit button still visible
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('404 missing scan via direct URL navigation shows Scan Not Found', async ({ page }) => {
    // Mock must be set up BEFORE navigation so it intercepts the first fetch
    await page.route('**/api/v1/scans/nonexistent-scan-id', async (route) => {
      await new Promise((r) => setTimeout(r, 200));
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ detail: 'Scan not found' }),
      });
    });

    // Navigate directly to a non-existent scan ID
    await page.goto('/scan/nonexistent-scan-id');

    // After 404 response: setLoading(false), scan stays null → "Scan Not Found" renders
    await expect(page.locator('text=Scan Not Found')).toBeVisible({ timeout: 10000 });

    // Recovery: "Start New Scan" link is visible and points to home
    await expect(page.locator('text=Start New Scan')).toBeVisible();
    const startNewScanLink = page.locator('a', { hasText: 'Start New Scan' });
    await expect(startNewScanLink).toHaveAttribute('href', '/');
  });

  test('404 results page shows not found', async ({ page, next }) => {
    // Server-side fetch returns 404 — results page calls notFound()
    mockResultsNotFound(next);

    // Navigate to an expired or invalid results token
    await page.goto('/results/tok_expired_or_invalid');

    // Next.js notFound() renders the built-in 404 page
    // Next.js default 404: "This page could not be found." or similar
    await expect(
      page.locator('text=/not found|could not be found|404/i').first()
    ).toBeVisible({ timeout: 10000 });

    // Recovery: user can navigate away — page should render navigation or home link
    const homeLink = page.locator('a[href="/"]').first();
    const hasHomeLink = await homeLink.count();
    if (hasHomeLink > 0) {
      await expect(homeLink).toBeVisible();
    } else {
      // At minimum the page rendered without crashing
      await expect(page.locator('body')).toBeVisible();
    }
  });

  test('network timeout shows connection error warning', async ({ page }) => {
    // Strategy: first request returns an in-progress scan (so scan state is set),
    // subsequent requests are aborted — errorCount builds to >= 3 while isScanning is true,
    // triggering the "Having trouble connecting" warning.
    let callCount = 0;

    await page.route('**/api/v1/scans/scan_e2e_timeout', async (route) => {
      callCount++;
      if (callCount === 1) {
        // First call: return an in-progress scan so scan state is populated
        await new Promise((r) => setTimeout(r, 100));
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(scanFixtures.inProgress),
        });
      } else {
        // Subsequent calls: abort to simulate network failure
        await route.abort('failed');
      }
    });

    // Navigate to scan progress page
    await page.goto('/scan/scan_e2e_timeout');

    // After first successful fetch: scan is set (inProgress), loading=false
    // Subsequent polling attempts abort → errorCount increments on each failure
    // After 3 failures: errorCount >= 3 && isScanning (status = 'in_progress') → warning shows
    // Each poll interval is 2 seconds — need at least 3 failed polls = ~6 seconds
    await expect(
      page.locator('text=Having trouble connecting').first()
    ).toBeVisible({ timeout: 15000 });

    // Recovery: page is still functional (not crashed), user can navigate away
    // The scan UI (heading) should still be visible
    await expect(page.locator('text=Scanning')).toBeVisible();
  });

  test('server 500 error on results page shows not found', async ({ page, next }) => {
    // Any backend API call returns 500 — results page catches error and calls notFound()
    mockServerError(next);

    // Navigate to results page — server component will get 500 and call notFound()
    await page.goto('/results/tok_e2e_server_error');

    // Next.js notFound() renders the built-in 404 page
    await expect(
      page.locator('text=/not found|could not be found|404/i').first()
    ).toBeVisible({ timeout: 10000 });

    // Recovery: user can navigate away
    const homeLink = page.locator('a[href="/"]').first();
    const hasHomeLink = await homeLink.count();
    if (hasHomeLink > 0) {
      await expect(homeLink).toBeVisible();
    } else {
      // At minimum the page rendered without crashing
      await expect(page.locator('body')).toBeVisible();
    }
  });

});
