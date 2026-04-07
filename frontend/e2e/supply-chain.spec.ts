import { test, expect } from 'next/experimental/testmode/playwright';
import { supplyChainFixtures } from './fixtures/supply-chain';
import { mockSupplyChainResults } from './helpers/fetch-mocks';

test.describe('Supply Chain Scan', () => {

  test('happy path: paste lockfile content and see results', async ({ page, next }) => {
    // The scan action (supply-chain-scan.ts) runs client-side (no 'use server' directive),
    // so we intercept with page.route() instead of next.onFetch().
    // The results page is a Server Component, so we use next.onFetch() for it.
    await page.route('**/api/v1/scans/supply-chain', async (route) => {
      await new Promise((r) => setTimeout(r, 100));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(supplyChainFixtures.scanResponse),
      });
    });

    // Mock the server-side results page fetch
    mockSupplyChainResults(next, supplyChainFixtures);

    // 1. Navigate to supply chain page
    await page.goto('/supply-chain');

    // 2. Verify page loaded
    await expect(page.locator('h1')).toBeVisible();

    // 3. Click 'Paste Content' tab
    await page.click('text=Paste Content');

    // 4. Fill in the paste textarea with a minimal lockfile
    const minimalLockfile = JSON.stringify({
      lockfileVersion: 3,
      packages: {
        '': { name: 'test-app', version: '1.0.0' },
        'node_modules/lodash': {
          version: '4.17.20',
          resolved: 'https://registry.npmjs.org/lodash/-/lodash-4.17.20.tgz',
        },
      },
    });
    await page.fill('#paste-content', minimalLockfile);

    // 5. Submit the form
    await page.click('text=Scan Dependencies');

    // 6. Wait for navigation to results page
    await page.waitForURL('**/supply-chain/results/**', { timeout: 15000 });

    // 7. Verify results page shows summary cards (use .first() to avoid strict mode
    //    violation — 'Vulnerable' appears in both the summary card label and findings badge)
    await expect(page.locator('text=Vulnerable').first()).toBeVisible();
    await expect(page.locator('text=No Known Issues').first()).toBeVisible();

    // 8. Verify the specific finding appears
    await expect(page.locator('text=lodash').first()).toBeVisible();
    await expect(page.locator('text=GHSA-jf85-cpcp-j695').first()).toBeVisible();
  });

  test('error state: empty paste content shows validation error', async ({ page }) => {
    // 1. Navigate to supply chain page
    await page.goto('/supply-chain');

    // 2. Click 'Paste Content' tab
    await page.click('text=Paste Content');

    // 3. Submit without entering any content
    await page.click('text=Scan Dependencies');

    // 4. Verify validation error appears
    await expect(page.locator('text=Please paste your package-lock.json content')).toBeVisible();
  });

});
