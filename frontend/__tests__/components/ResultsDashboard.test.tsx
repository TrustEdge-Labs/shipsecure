import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '../helpers/test-utils'
import { ResultsDashboard } from '@/components/results-dashboard'
import { Finding } from '@/lib/types'

const testFindings: Finding[] = [
  {
    id: '1',
    title: 'Missing CSP header',
    description: 'No CSP detected.',
    severity: 'high',
    remediation: 'Add CSP header.',
    scanner_name: 'security_headers',
    vibe_code: false,
  },
  {
    id: '2',
    title: 'Exposed .env file',
    description: '.env file accessible.',
    severity: 'critical',
    remediation: 'Block .env access.',
    scanner_name: 'exposed_files',
    vibe_code: true,
  },
  {
    id: '3',
    title: 'Supabase key in bundle',
    description: 'Found anon key.',
    severity: 'medium',
    remediation: 'Move to server-side.',
    scanner_name: 'js_secrets',
    vibe_code: true,
  },
]

describe('ResultsDashboard', () => {
  describe('Empty State', () => {
    it('renders "No Security Issues Found!" heading when findings array is empty', () => {
      renderWithProviders(<ResultsDashboard findings={[]} />)
      expect(screen.getByText('No Security Issues Found!')).toBeInTheDocument()
    })

    it('renders "Your application passed all checks" text when empty', () => {
      renderWithProviders(<ResultsDashboard findings={[]} />)
      expect(screen.getByText('Your application passed all checks.')).toBeInTheDocument()
    })
  })

  describe('Findings Rendering', () => {
    it('renders finding titles when findings are present', () => {
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      expect(screen.getByText('Missing CSP header')).toBeInTheDocument()
      expect(screen.getByText('Exposed .env file')).toBeInTheDocument()
      expect(screen.getByText('Supabase key in bundle')).toBeInTheDocument()
    })

    it('groups by severity by default', () => {
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      expect(screen.getByText('Critical (1)')).toBeInTheDocument()
      expect(screen.getByText('High (1)')).toBeInTheDocument()
      expect(screen.getByText('Medium (1)')).toBeInTheDocument()
    })

    it('shows correct severity group counts', () => {
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      const criticalHeading = screen.getByText('Critical (1)')
      const highHeading = screen.getByText('High (1)')
      const mediumHeading = screen.getByText('Medium (1)')

      expect(criticalHeading).toBeInTheDocument()
      expect(highHeading).toBeInTheDocument()
      expect(mediumHeading).toBeInTheDocument()
    })
  })

  describe('Grouping Toggle', () => {
    it('renders "By Severity" and "By Category" toggle buttons', () => {
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      const severityButton = screen.getByRole('button', { name: /by severity/i })
      const categoryButton = screen.getByRole('button', { name: /by category/i })

      expect(severityButton).toBeInTheDocument()
      expect(categoryButton).toBeInTheDocument()
    })

    it('switches to category grouping on click', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      const categoryButton = screen.getByRole('button', { name: /by category/i })
      await user.click(categoryButton)

      expect(screen.getByText('Headers (1)')).toBeInTheDocument()
      expect(screen.getByText('Exposed Files (1)')).toBeInTheDocument()
      expect(screen.getByText('JavaScript Secrets (1)')).toBeInTheDocument()
    })

    it('switches back to severity grouping', async () => {
      const user = userEvent.setup()
      renderWithProviders(<ResultsDashboard findings={testFindings} />)

      const categoryButton = screen.getByRole('button', { name: /by category/i })
      const severityButton = screen.getByRole('button', { name: /by severity/i })

      await user.click(categoryButton)
      expect(screen.getByText('Headers (1)')).toBeInTheDocument()

      await user.click(severityButton)
      expect(screen.getByText('Critical (1)')).toBeInTheDocument()
      expect(screen.getByText('High (1)')).toBeInTheDocument()
      expect(screen.getByText('Medium (1)')).toBeInTheDocument()
    })
  })
})
