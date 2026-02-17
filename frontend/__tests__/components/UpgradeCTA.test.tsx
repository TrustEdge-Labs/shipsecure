import { describe, expect, test, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { UpgradeCTA } from '@/components/upgrade-cta'
import { server } from '@/__tests__/helpers/msw/server'
import { http, HttpResponse } from 'msw'

describe('UpgradeCTA', () => {
  beforeEach(() => {
    // Mock window.location to prevent navigation errors in tests
    Object.defineProperty(window, 'location', {
      value: { href: '' },
      writable: true,
      configurable: true,
    })
  })

  describe('Rendering', () => {
    test('renders "Upgrade to Deep Audit" heading', () => {
      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const heading = screen.getByRole('heading', {
        name: /upgrade to deep audit/i,
      })
      expect(heading).toBeInTheDocument()
    })

    test('renders upgrade button with "$49" price', () => {
      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      expect(button).toBeInTheDocument()
    })

    test('renders feature list items', () => {
      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      expect(screen.getByText(/10x more checks/i)).toBeInTheDocument()
      expect(screen.getByText(/SQL injection, auth bypass/i)).toBeInTheDocument()
      expect(screen.getByText(/professional executive summary/i)).toBeInTheDocument()
      expect(screen.getByText(/scan 50 files vs 20/i)).toBeInTheDocument()
    })
  })

  describe('Checkout Flow', () => {
    test('button shows loading text during checkout', async () => {
      const user = userEvent.setup()

      // Mock successful checkout response
      server.use(
        http.post('http://localhost:3000/api/v1/checkout', () => {
          return HttpResponse.json({
            checkout_url: 'https://checkout.stripe.com/test',
          })
        })
      )

      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      await user.click(button)

      // Button should show loading text
      expect(
        await screen.findByText(/redirecting to checkout/i)
      ).toBeInTheDocument()
    })

    test('calls checkout API on click', async () => {
      const user = userEvent.setup()
      let apiCalled = false

      // Mock checkout endpoint and track if it's called
      server.use(
        http.post('http://localhost:3000/api/v1/checkout', () => {
          apiCalled = true
          return HttpResponse.json({
            checkout_url: 'https://checkout.stripe.com/test-session',
          })
        })
      )

      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      await user.click(button)

      // Wait for loading state to appear (indicates API was called)
      await waitFor(() => {
        expect(apiCalled).toBe(true)
      })
    })

    test('shows error message when checkout fails', async () => {
      const user = userEvent.setup()

      // Mock checkout failure
      server.use(
        http.post('http://localhost:3000/api/v1/checkout', () => {
          return HttpResponse.json(
            { title: 'Payment service unavailable' },
            { status: 500 }
          )
        })
      )

      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      await user.click(button)

      // Error message should appear
      expect(
        await screen.findByText(/payment service unavailable/i)
      ).toBeInTheDocument()
    })
  })

  describe('Error Display', () => {
    test('error message is displayed in error banner', async () => {
      const user = userEvent.setup()

      // Mock checkout failure
      server.use(
        http.post('http://localhost:3000/api/v1/checkout', () => {
          return HttpResponse.json(
            { title: 'Payment service unavailable' },
            { status: 500 }
          )
        })
      )

      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      await user.click(button)

      // Wait for error to appear
      const errorText = await screen.findByText(/payment service unavailable/i)

      // Error should be in a visible container
      expect(errorText).toBeVisible()
    })

    test('button re-enables after error', async () => {
      const user = userEvent.setup()

      // Mock checkout failure
      server.use(
        http.post('http://localhost:3000/api/v1/checkout', () => {
          return HttpResponse.json(
            { title: 'Checkout failed' },
            { status: 500 }
          )
        })
      )

      renderWithProviders(<UpgradeCTA scanId="scan_123" token="tok_abc" />)

      const button = screen.getByRole('button', { name: /upgrade for \$49/i })
      await user.click(button)

      // Wait for error to appear
      await screen.findByText(/checkout failed/i)

      // Button should no longer be disabled (loading state cleared)
      expect(button).not.toBeDisabled()
    })
  })
})
