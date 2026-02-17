/**
 * E2E fixtures for scan API responses.
 * These match the CreateScanResponse and Scan type shapes from lib/types.ts.
 * Kept separate from MSW fixtures used in component tests.
 */

export const scanFixtures = {
  created: {
    id: 'scan_e2e_001',
    status: 'pending',
    url: '/api/v1/scans/scan_e2e_001',
    target_url: 'https://example.com',
    results_token: null,
    stage_detection: false,
    stage_headers: false,
    stage_tls: false,
    stage_files: false,
    stage_secrets: false,
    stage_vibecode: false,
    error_message: null,
  },
  inProgress: {
    id: 'scan_e2e_001',
    status: 'in_progress',
    url: '/api/v1/scans/scan_e2e_001',
    target_url: 'https://example.com',
    results_token: null,
    stage_detection: true,
    stage_headers: true,
    stage_tls: false,
    stage_files: false,
    stage_secrets: false,
    stage_vibecode: false,
    error_message: null,
  },
  completed: {
    id: 'scan_e2e_001',
    status: 'completed',
    url: '/api/v1/scans/scan_e2e_001',
    target_url: 'https://example.com',
    results_token: 'tok_e2e_abc123',
    stage_detection: true,
    stage_headers: true,
    stage_tls: true,
    stage_files: true,
    stage_secrets: true,
    stage_vibecode: true,
    error_message: null,
  },
  failed: {
    id: 'scan_e2e_001',
    status: 'failed',
    url: '/api/v1/scans/scan_e2e_001',
    target_url: 'https://example.com',
    results_token: null,
    stage_detection: false,
    stage_headers: false,
    stage_tls: false,
    stage_files: false,
    stage_secrets: false,
    stage_vibecode: false,
    error_message: 'Target website is unreachable',
  },
} as const;

export type ScanFixtures = typeof scanFixtures;
