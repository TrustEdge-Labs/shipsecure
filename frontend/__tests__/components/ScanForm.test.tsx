import { vi, describe, test, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ScanForm } from '@/components/scan-form'

// Mock state for useActionState
let mockState = {} as any
let mockFormAction = vi.fn()
let mockPending = false

vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual,
    useActionState: vi.fn(() => [mockState, mockFormAction, mockPending])
  }
})

describe('ScanForm', () => {
  beforeEach(() => {
    mockState = {}
    mockFormAction = vi.fn()
    mockPending = false
  })

  describe('Form Fields', () => {
    test('renders disabled URL input pre-filled with demo target for anonymous users', () => {
      renderWithProviders(<ScanForm />)
      const urlInput = screen.getByLabelText(/website url/i)
      expect(urlInput).toBeInTheDocument()
      expect(urlInput).toHaveAttribute('type', 'url')
      expect(urlInput).toBeDisabled()
      expect(urlInput).toHaveValue('https://demo.owasp-juice.shop')
    })

    test('renders editable URL input for authenticated users', () => {
      renderWithProviders(<ScanForm isAuthenticated />)
      const urlInput = screen.getByLabelText(/website url/i)
      expect(urlInput).toBeInTheDocument()
      expect(urlInput).toHaveAttribute('type', 'url')
      expect(urlInput).toHaveAttribute('name', 'url')
      expect(urlInput).not.toBeDisabled()
    })

    test('shows demo target explanation for anonymous users', () => {
      renderWithProviders(<ScanForm />)
      expect(screen.getByText(/anonymous scans are limited to our live demo target/i)).toBeInTheDocument()
      expect(screen.getByText(/sign up for free/i)).toBeInTheDocument()
    })

    test('renders email input with label containing "Email"', () => {
      renderWithProviders(<ScanForm />)
      const emailInput = screen.getByLabelText(/email/i)
      expect(emailInput).toBeInTheDocument()
      expect(emailInput).toHaveAttribute('type', 'email')
      expect(emailInput).toHaveAttribute('name', 'email')
    })

    test('renders CFAA consent checkbox with authorization label', () => {
      renderWithProviders(<ScanForm />)
      const checkbox = screen.getByRole('checkbox')
      expect(checkbox).toBeInTheDocument()
      expect(checkbox).toHaveAttribute('name', 'authorization')

      const label = screen.getByText(/I confirm I own this website/i)
      expect(label).toBeInTheDocument()
    })

    test('renders submit button with "Scan Now" text', () => {
      renderWithProviders(<ScanForm />)
      const button = screen.getByRole('button', { name: /scan now/i })
      expect(button).toBeInTheDocument()
      expect(button).toHaveAttribute('type', 'submit')
    })
  })

  describe('Validation Errors', () => {
    test('displays URL validation error when present in state', () => {
      mockState = { errors: { url: ['Please enter a valid URL'] } }
      renderWithProviders(<ScanForm />)

      expect(screen.getByText(/please enter a valid url/i)).toBeInTheDocument()
    })

    test('displays email validation error when present in state', () => {
      mockState = { errors: { email: ['Please enter a valid email address'] } }
      renderWithProviders(<ScanForm />)

      expect(screen.getByText(/please enter a valid email address/i)).toBeInTheDocument()
    })

    test('displays authorization error when present in state', () => {
      mockState = { errors: { authorization: ['You must confirm you have authorization'] } }
      renderWithProviders(<ScanForm />)

      expect(screen.getByText(/you must confirm you have authorization/i)).toBeInTheDocument()
    })

    test('displays form-level error in error banner', () => {
      mockState = { errors: { _form: ['Unable to connect to the scanning service'] } }
      renderWithProviders(<ScanForm />)

      expect(screen.getByText(/unable to connect to the scanning service/i)).toBeInTheDocument()
    })
  })

  describe('Loading State', () => {
    test('button text changes to "Starting scan..." when pending', () => {
      mockPending = true
      renderWithProviders(<ScanForm />)

      const button = screen.getByRole('button', { name: /starting scan/i })
      expect(button).toBeInTheDocument()
    })

    test('button is disabled when pending', () => {
      mockPending = true
      renderWithProviders(<ScanForm />)

      const button = screen.getByRole('button', { name: /starting scan/i })
      expect(button).toBeDisabled()
    })
  })

  describe('Success State', () => {
    test('renders success message when scanId is present', () => {
      mockState = { scanId: 'scan-123' }
      renderWithProviders(<ScanForm />)

      expect(screen.getByText(/scan started!/i)).toBeInTheDocument()
      expect(screen.getByText(/redirecting to your scan progress/i)).toBeInTheDocument()
    })
  })

  describe('User Interactions', () => {
    test('user can type in URL field when authenticated', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm isAuthenticated />)

      const urlInput = screen.getByLabelText(/website url/i)
      await user.type(urlInput, 'https://example.com')

      expect(urlInput).toHaveValue('https://example.com')
    })

    test('user can type in email field', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm />)

      const emailInput = screen.getByLabelText(/email/i)
      await user.type(emailInput, 'test@example.com')

      expect(emailInput).toHaveValue('test@example.com')
    })

    test('user can toggle consent checkbox', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ScanForm />)

      const checkbox = screen.getByRole('checkbox')

      // Initially unchecked
      expect(checkbox).not.toBeChecked()

      // Click to check
      await user.click(checkbox)
      expect(checkbox).toBeChecked()

      // Click to uncheck
      await user.click(checkbox)
      expect(checkbox).not.toBeChecked()
    })
  })
})
