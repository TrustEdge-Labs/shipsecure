import React from 'react'
import { beforeAll, afterEach, afterAll, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './__tests__/helpers/msw/server'
import '@testing-library/jest-dom/vitest'

// --- MSW Server Lifecycle ---
// Enable API mocking before all tests
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))

// Reset handlers to initial state between tests (critical for test isolation)
afterEach(() => {
  cleanup()
  server.resetHandlers()
})

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
    return React.createElement('img', { src, alt, ...props })
  },
}))

// Mock @clerk/nextjs so components using Clerk hooks/components render without ClerkProvider
vi.mock('@clerk/nextjs', () => ({
  useClerk: vi.fn(() => ({ openSignUp: vi.fn() })),
  useUser: vi.fn(() => ({ isSignedIn: false, user: null })),
  useAuth: vi.fn(() => ({ isSignedIn: false, userId: null })),
  SignedIn: ({ children }: { children: React.ReactNode }) => null,
  SignedOut: ({ children }: { children: React.ReactNode }) => React.createElement(React.Fragment, null, children),
  UserButton: () => null,
  ClerkProvider: ({ children }: { children: React.ReactNode }) => React.createElement(React.Fragment, null, children),
}))
