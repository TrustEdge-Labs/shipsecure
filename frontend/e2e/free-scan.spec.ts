import { test, expect } from 'next/experimental/testmode/playwright';
import { scanFixtures } from './fixtures/scan';
import { resultsFixtures } from './fixtures/results';
import { mockScanCount, mockScanSubmission, mockResultsPage } from './helpers/fetch-mocks';
import { mockScanPolling } from './helpers/route-mocks';

test.describe('Free Scan Flow', () => {
  test('complete free scan flow from home to results', async ({ page, next }) => {
    // Setup server-side mocks
    mockScanCount(next, 42);
    mockScanSubmission(next, scanFixtures);
    mockResultsPage(next, resultsFixtures.freeGradeB);

    // Setup client-side polling mock
    await mockScanPolling(page, scanFixtures);

    // 1. Navigate to home page
    await page.goto('/');

    // 2. Verify home page loaded
    await expect(page.locator('h1')).toContainText('Security scanning');

    // 3. Fill the form
    await page.fill('input[name="url"]', 'https://example.com');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.check('input[name="authorization"]');

    // 4. Submit
    await page.click('button[type="submit"]');

    // 5. Wait for navigation to scan progress page
    await page.waitForURL('**/scan/**', { timeout: 15000 });

    // 6. Verify progress UI — the h1 shows "Scanning" when scan is in_progress
    await expect(page.locator('h1')).toContainText('Scanning');

    // 7. Wait for scan to complete and redirect to results
    await page.waitForURL('**/results/**', { timeout: 30000 });

    // 8. Verify results page content
    await expect(page.locator('h1')).toContainText('Security Scan Results');

    // Grade display — freeGradeB fixture has score 'B'
    // Use exact text match scoped to the grade circle element to avoid ambiguity
    await expect(page.locator('[class*="grade-b-bg"]')).toBeVisible();

    // Severity counts in grade summary — freeGradeB has 1 high and 1 medium
    await expect(page.locator('text=1 High')).toBeVisible();
    await expect(page.locator('text=1 Medium')).toBeVisible();

    // Verify finding title from fixture is visible
    await expect(page.locator('text=Missing Content-Security-Policy Header')).toBeVisible();

    // UpgradeCTA for free tier
    await expect(page.locator('text=Upgrade to Deep Audit')).toBeVisible();
  });

  test('CFAA consent required before submission', async ({ page, next }) => {
    // Setup server-side mocks
    mockScanCount(next, 0);
    mockScanSubmission(next, scanFixtures);

    // 1. Navigate to home page
    await page.goto('/');

    // 2. Fill URL and email but do NOT check the authorization checkbox
    await page.fill('input[name="url"]', 'https://example.com');
    await page.fill('input[name="email"]', 'test@example.com');

    // 3. Click submit without checking authorization
    await page.click('button[type="submit"]');

    // 4. Browser native validation prevents form submission — authorization checkbox is invalid
    const authCheckbox = page.locator('input[name="authorization"]');
    const authValid = await authCheckbox.evaluate((el: HTMLInputElement) => el.validity.valid);
    expect(authValid).toBe(false);

    // Page stays on home page (not navigated to scan page)
    expect(page.url()).not.toContain('/scan/');

    // 5. Now check the authorization checkbox to consent
    await page.check('input[name="authorization"]');

    // 6. Set up polling mock for the successful submission
    await mockScanPolling(page, scanFixtures);

    // 7. Click submit again — all fields valid now
    await page.click('button[type="submit"]');

    // 8. Verify form progresses to scan page
    await page.waitForURL('**/scan/**', { timeout: 15000 });
  });
});
