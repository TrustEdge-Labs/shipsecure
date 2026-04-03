import { describe, test, expect, vi, beforeEach } from 'vitest'
import { render } from '@testing-library/react'
import { SignupTracker } from '@/components/signup-tracker'

// Mock next/navigation
const mockGet = vi.fn()
vi.mock('next/navigation', () => ({
  useSearchParams: () => ({
    get: mockGet,
  }),
}))

describe('SignupTracker', () => {
  beforeEach(() => {
    mockGet.mockReset()
    window.plausible = vi.fn()
  })

  test('fires Signup Completed when source=signup', () => {
    mockGet.mockReturnValue('signup')
    render(<SignupTracker />)
    expect(window.plausible).toHaveBeenCalledWith('Signup Completed')
  })

  test('does not fire event without source param', () => {
    mockGet.mockReturnValue(null)
    render(<SignupTracker />)
    expect(window.plausible).not.toHaveBeenCalled()
  })

  test('does not fire event with different source', () => {
    mockGet.mockReturnValue('other')
    render(<SignupTracker />)
    expect(window.plausible).not.toHaveBeenCalled()
  })

  test('renders nothing', () => {
    mockGet.mockReturnValue(null)
    const { container } = render(<SignupTracker />)
    expect(container.innerHTML).toBe('')
  })
})
