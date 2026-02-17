import { vi, describe, test, expect, beforeEach, afterEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import ErrorBoundary from '@/app/error'

beforeEach(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('Fallback UI Rendering', () => {
  test('renders "Something went wrong" heading', () => {
    renderWithProviders(
      <ErrorBoundary error={new Error('Test error')} reset={vi.fn()} />
    )
    const heading = screen.getByRole('heading', { name: /something went wrong/i })
    expect(heading).toBeInTheDocument()
  })

  test('renders error description text', () => {
    renderWithProviders(
      <ErrorBoundary error={new Error('Test error')} reset={vi.fn()} />
    )
    const description = screen.getByText(/an unexpected error occurred/i)
    expect(description).toBeInTheDocument()
  })
})

describe('User Actions', () => {
  test('Try again button calls reset function', async () => {
    const mockReset = vi.fn()
    const user = userEvent.setup()

    renderWithProviders(
      <ErrorBoundary error={new Error('Test error')} reset={mockReset} />
    )

    const tryAgainButton = screen.getByRole('button', { name: /try again/i })
    await user.click(tryAgainButton)

    expect(mockReset).toHaveBeenCalledOnce()
  })

  test('Return to Home link points to "/"', () => {
    renderWithProviders(
      <ErrorBoundary error={new Error('Test error')} reset={vi.fn()} />
    )

    const homeLink = screen.getByRole('link', { name: /return to home/i })
    expect(homeLink).toBeInTheDocument()
    expect(homeLink).toHaveAttribute('href', '/')
  })
})
