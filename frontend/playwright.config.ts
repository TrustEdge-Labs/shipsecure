import { defineConfig } from 'next/experimental/testmode/playwright';

// Use port 3001 for E2E tests to avoid conflicts with other services on port 3000
const E2E_PORT = process.env.E2E_PORT ? parseInt(process.env.E2E_PORT) : 3001;
const E2E_BASE_URL = `http://localhost:${E2E_PORT}`;

export default defineConfig({
  testDir: './e2e',
  // Override default testMatch from next/experimental/testmode/playwright
  // which defaults to "{app,pages}/**/*.spec.{t,j}s" and ignores the testDir
  testMatch: '**/*.spec.ts',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'dot' : 'html',
  use: {
    baseURL: E2E_BASE_URL,
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        viewport: { width: 1280, height: 720 },
      },
    },
  ],
  webServer: {
    command: `npm run start -- -p ${E2E_PORT}`,
    url: E2E_BASE_URL,
    reuseExistingServer: false,
    timeout: 120_000,
    stdout: 'ignore',
    stderr: 'pipe',
    env: {
      PLAYWRIGHT_TEST: '1',
      PORT: String(E2E_PORT),
      // Both server-side (BACKEND_URL) and client-side (NEXT_PUBLIC_BACKEND_URL) point
      // back to the same Next.js server so page.route() intercepts work correctly
      BACKEND_URL: E2E_BASE_URL,
      NEXT_PUBLIC_BACKEND_URL: E2E_BASE_URL,
    },
  },
});
