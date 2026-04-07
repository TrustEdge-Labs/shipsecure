/**
 * Reusable next.onFetch() interceptors for E2E tests.
 * These intercept server-side fetch calls made from Next.js Server Actions
 * and Server Components via the Next.js testProxy.
 */

import type { NextFixture } from 'next/experimental/testmode/playwright';
import type { ScanFixtures } from '../fixtures/scan';
import type { ResultsFixtures } from '../fixtures/results';
import type { SupplyChainFixtures } from '../fixtures/supply-chain';

/**
 * Intercepts POST to /api/v1/scans server-side (the Server Action's outbound fetch).
 * Returns 201 with the created scan fixture.
 * Adds 200ms delay to simulate real scan submission timing.
 */
export function mockScanSubmission(
  next: NextFixture,
  scanFixtures: ScanFixtures
): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname === '/api/v1/scans' && request.method === 'POST') {
      await new Promise((r) => setTimeout(r, 200));
      return new Response(JSON.stringify(scanFixtures.created), {
        status: 201,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return undefined;
  });
}

/**
 * Intercepts GET to /api/v1/results/{token} server-side.
 * Returns 200 with the provided results fixture.
 * Adds 150ms delay.
 */
export function mockResultsPage(
  next: NextFixture,
  resultsFixture: ResultsFixtures[keyof ResultsFixtures]
): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname.startsWith('/api/v1/results/') && request.method === 'GET') {
      await new Promise((r) => setTimeout(r, 150));
      return new Response(JSON.stringify(resultsFixture), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return undefined;
  });
}

/**
 * Intercepts GET to /api/v1/stats/scan-count server-side.
 * The home page fetches scan count as a Server Component.
 * Returns a response with the given count value.
 */
export function mockScanCount(next: NextFixture, count: number): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname === '/api/v1/stats/scan-count' && request.method === 'GET') {
      return new Response(JSON.stringify({ count }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return undefined;
  });
}

/**
 * Intercepts GET to /api/v1/results/{token} and returns 404.
 * Use this to test the results-not-found error state.
 */
export function mockResultsNotFound(next: NextFixture): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname.startsWith('/api/v1/results/') && request.method === 'GET') {
      await new Promise((r) => setTimeout(r, 150));
      return new Response(
        JSON.stringify({ detail: 'Results not found or expired' }),
        {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }
    return undefined;
  });
}

/**
 * Intercepts any backend API call and returns a 500 Internal Server Error.
 * Use this to test error boundary and server error handling.
 */
export function mockServerError(next: NextFixture): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname.startsWith('/api/')) {
      await new Promise((r) => setTimeout(r, 100));
      return new Response(
        JSON.stringify({ detail: 'Internal server error' }),
        {
          status: 500,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }
    return undefined;
  });
}

/**
 * Intercepts POST to /api/v1/scans/supply-chain server-side.
 * Returns 200 with the supply chain scan response fixture.
 * Adds 100ms delay to simulate real scan submission timing.
 */
export function mockSupplyChainScan(
  next: NextFixture,
  fixtures: SupplyChainFixtures
): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname === '/api/v1/scans/supply-chain' && request.method === 'POST') {
      await new Promise((r) => setTimeout(r, 100));
      return new Response(JSON.stringify(fixtures.scanResponse), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return undefined;
  });
}

/**
 * Intercepts GET to /api/v1/results/{token} server-side for supply chain results pages.
 * Returns 200 with the supply chain results page fixture.
 */
export function mockSupplyChainResults(
  next: NextFixture,
  fixtures: SupplyChainFixtures
): void {
  next.onFetch(async (request) => {
    const url = new URL(request.url);
    if (url.pathname.startsWith('/api/v1/results/') && request.method === 'GET') {
      return new Response(JSON.stringify(fixtures.resultsPageResponse), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    return undefined;
  });
}
