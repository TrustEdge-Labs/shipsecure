import { describe, test, expect, vi, beforeEach } from 'vitest'
import { screen, fireEvent } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { SupplyChainForm } from '@/components/supply-chain-form'

vi.mock('@/app/actions/supply-chain-scan', () => ({
  submitSupplyChainScan: vi.fn(),
}))

describe('SupplyChainForm', () => {
  beforeEach(() => {
    window.plausible = vi.fn()
  })

  test('renders three tab buttons and GitHub tab is active by default', () => {
    renderWithProviders(<SupplyChainForm />)

    expect(screen.getByText('GitHub URL')).toBeInTheDocument()
    expect(screen.getByText('Upload File')).toBeInTheDocument()
    expect(screen.getByText('Paste Content')).toBeInTheDocument()

    // GitHub tab is default — input should be visible
    expect(screen.getByLabelText(/github repository url/i)).toBeInTheDocument()

    // Switch to Paste Content tab
    fireEvent.click(screen.getByText('Paste Content'))

    // Textarea for paste content should now appear
    expect(screen.getByLabelText(/package-lock\.json content/i)).toBeInTheDocument()
  })

  test('shows validation error on empty GitHub URL submit', async () => {
    renderWithProviders(<SupplyChainForm />)

    // Click submit without entering a URL
    fireEvent.click(screen.getByText('Scan Dependencies'))

    expect(
      await screen.findByText('Please enter a GitHub repository URL')
    ).toBeInTheDocument()
  })
})
