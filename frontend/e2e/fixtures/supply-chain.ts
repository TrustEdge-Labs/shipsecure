/**
 * E2E fixtures for supply chain scan API responses.
 * These match the SupplyChainScanResponse and SupplyChainResultsPageData
 * type shapes from lib/supply-chain-types.ts.
 * Kept separate from MSW fixtures used in component tests.
 */

export const supplyChainFixtures = {
  scanResponse: {
    status: 'completed',
    results_token: 'test-sc-token-abc123',
    share_url: '/supply-chain/results/test-sc-token-abc123',
    share_unavailable: false,
    results: {
      total_deps: 3,
      infected: [],
      vulnerable: [
        {
          name: 'lodash',
          version: '4.17.20',
          osv_id: 'GHSA-jf85-cpcp-j695',
          description: 'Prototype Pollution in lodash',
          tier: 'Vulnerable',
        },
      ],
      advisory: [],
      no_known_issues: ['express@4.18.2', 'react@18.2.0'],
      unscanned: [],
      scanned_at: '2026-01-01T00:00:00',
    },
  },
  resultsPageResponse: {
    id: 'test-scan-id',
    target_url: 'paste',
    status: 'completed',
    kind: 'supply_chain',
    expires_at: '2026-02-01T00:00:00',
    created_at: '2026-01-01T00:00:00',
    completed_at: '2026-01-01T00:00:01',
    supply_chain_results: {
      total_deps: 3,
      infected: [],
      vulnerable: [
        {
          name: 'lodash',
          version: '4.17.20',
          osv_id: 'GHSA-jf85-cpcp-j695',
          description: 'Prototype Pollution in lodash',
          tier: 'Vulnerable',
        },
      ],
      advisory: [],
      no_known_issues: ['express@4.18.2', 'react@18.2.0'],
      unscanned: [],
      scanned_at: '2026-01-01T00:00:00',
    },
  },
};

export type SupplyChainFixtures = typeof supplyChainFixtures;
