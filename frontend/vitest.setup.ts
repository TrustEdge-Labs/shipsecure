import { beforeAll, afterEach, afterAll, vi } from 'vitest'
import { server } from './__tests__/helpers/msw/server'

// --- MSW Server Lifecycle ---
// Enable API mocking before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))

// Reset handlers to initial state between tests (critical for test isolation)
afterEach(() => server.resetHandlers())

// Clean up after all tests
afterAll(() => server.close())

// --- Global Mocks ---

// Mock next/navigation hooks globally for all component tests
// Components using useRouter, usePathname, or useSearchParams will render without errors
vi.mock('next/navigation', () => ({
  useRouter: vi.fn(() => ({
    push: vi.fn(),
    replace: vi.fn(),
    prefetch: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    refresh: vi.fn(),
  })),
  usePathname: vi.fn(() => '/'),
  useSearchParams: vi.fn(() => new URLSearchParams()),
}))

// Mock next/image to render a plain img tag (happy-dom doesn't support Next.js Image optimization)
vi.mock('next/image', () => ({
  default: ({ src, alt, ...props }: { src: string; alt: string; [key: string]: unknown }) => {
    // eslint-disable-next-line @next/next/no-img-element, jsx-a11y/alt-text
    return <img src={src} alt={alt} {...props} />
  },
}))
