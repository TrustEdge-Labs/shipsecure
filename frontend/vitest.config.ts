import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tsconfigPaths from 'vite-tsconfig-paths'
import { loadEnvConfig } from '@next/env'

// Load Next.js environment variables (.env.test when NODE_ENV=test)
loadEnvConfig(process.cwd())

export default defineConfig({
  plugins: [
    tsconfigPaths(),
    react(),
  ],
  test: {
    environment: 'happy-dom',
    setupFiles: ['./vitest.setup.ts'],
    include: ['__tests__/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      include: ['components/**'],
      exclude: [
        'node_modules/',
        '__tests__/',
        '*.config.*',
        '.next/',
        'app/**/layout.tsx',
        'app/**/loading.tsx',
        'app/**/error.tsx',
        'app/**/global-error.tsx',
        'app/**/opengraph-image.tsx',
        'app/robots.ts',
        'app/sitemap.ts',
      ],
      thresholds: {
        lines: 80,
        functions: 80,
        branches: 75,
      },
    },
  },
})
