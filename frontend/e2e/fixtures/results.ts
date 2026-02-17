/**
 * E2E fixtures for results API responses.
 * These match the ScanResponse type shape from lib/types.ts exactly.
 * Kept separate from MSW fixtures used in component tests.
 */

export const resultsFixtures = {
  freeGradeB: {
    id: 'scan_e2e_001',
    target_url: 'https://example.com',
    status: 'completed',
    score: 'B',
    tier: 'free',
    results_token: 'tok_e2e_abc123',
    expires_at: '2026-03-17T04:52:14Z',
    stage_detection: true,
    stage_headers: true,
    stage_tls: true,
    stage_files: true,
    stage_secrets: true,
    stage_vibecode: true,
    detected_framework: 'Next.js',
    detected_platform: 'Vercel',
    created_at: '2026-02-17T04:52:14Z',
    started_at: '2026-02-17T04:52:20Z',
    completed_at: '2026-02-17T04:53:14Z',
    findings: [
      {
        id: 'finding_e2e_001',
        title: 'Missing Content-Security-Policy Header',
        description: 'The application does not set a Content-Security-Policy header, which allows cross-site scripting attacks to load external scripts.',
        severity: 'high' as const,
        remediation: 'Add a Content-Security-Policy header to all responses. At minimum, set "default-src \'self\'".',
        scanner_name: 'headers',
        vibe_code: false,
      },
      {
        id: 'finding_e2e_002',
        title: 'Weak SSL/TLS Configuration',
        description: 'The server supports TLS 1.0 which is deprecated and has known vulnerabilities.',
        severity: 'medium' as const,
        remediation: 'Disable TLS 1.0 and 1.1. Configure the server to only accept TLS 1.2 and 1.3.',
        scanner_name: 'tls',
        vibe_code: false,
      },
      {
        id: 'finding_e2e_003',
        title: 'Missing X-Frame-Options Header',
        description: 'The application does not set X-Frame-Options, making it vulnerable to clickjacking attacks.',
        severity: 'low' as const,
        remediation: 'Add "X-Frame-Options: DENY" or "X-Frame-Options: SAMEORIGIN" header to all responses.',
        scanner_name: 'headers',
        vibe_code: false,
      },
    ],
    summary: {
      total: 3,
      critical: 0,
      high: 1,
      medium: 1,
      low: 1,
    },
  },
  paidGradeA: {
    id: 'scan_e2e_002',
    target_url: 'https://secure-app.example.com',
    status: 'completed',
    score: 'A',
    tier: 'paid',
    results_token: 'tok_e2e_paid456',
    expires_at: '2026-03-17T04:52:14Z',
    stage_detection: true,
    stage_headers: true,
    stage_tls: true,
    stage_files: true,
    stage_secrets: true,
    stage_vibecode: true,
    detected_framework: 'Next.js',
    detected_platform: 'Vercel',
    created_at: '2026-02-17T04:52:14Z',
    started_at: '2026-02-17T04:52:20Z',
    completed_at: '2026-02-17T04:53:14Z',
    findings: [
      {
        id: 'finding_e2e_004',
        title: 'Cookie Without SameSite Attribute',
        description: 'A session cookie is missing the SameSite attribute, which may allow cross-site request forgery.',
        severity: 'low' as const,
        remediation: 'Set SameSite=Strict or SameSite=Lax on all session cookies.',
        scanner_name: 'headers',
        vibe_code: false,
      },
    ],
    summary: {
      total: 1,
      critical: 0,
      high: 0,
      medium: 0,
      low: 1,
    },
  },
} as const;

export type ResultsFixtures = typeof resultsFixtures;
