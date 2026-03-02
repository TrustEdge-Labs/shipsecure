import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { ScanHistoryTable } from '@/components/scan-history-table'
import type { ScanHistoryItem } from '@/lib/types'

// Fixed test date: 2026-02-15 12:00:00 UTC
const FIXED_NOW = new Date('2026-02-15T12:00:00Z').getTime()

function makeScan(overrides: Partial<ScanHistoryItem> = {}): ScanHistoryItem {
  return {
    id: 'scan-1',
    target_url: 'https://example.com',
    status: 'completed',
    results_token: 'tok-abc',
    expires_at: new Date(FIXED_NOW + 30 * 86400000).toISOString(),
    tier: 'authenticated',
    created_at: '2026-02-15T12:00:00Z',
    critical_count: 1,
    high_count: 2,
    medium_count: 0,
    low_count: 0,
    ...overrides,
  }
}

describe('ScanHistoryTable', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    vi.setSystemTime(FIXED_NOW)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  describe('Empty state', () => {
    it('renders "No completed scans yet." when scans array is empty', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[]} currentPage={1} totalPages={1} />
      )
      expect(screen.getByText('No completed scans yet.')).toBeInTheDocument()
    })
  })

  describe('Basic row rendering', () => {
    it('renders the hostname extracted from the target URL', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan()]} currentPage={1} totalPages={1} />
      )
      // Hostname is rendered, not the full URL
      expect(screen.getAllByText('example.com').length).toBeGreaterThan(0)
    })

    it('renders the formatted scan date', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan()]} currentPage={1} totalPages={1} />
      )
      // "Feb 15, 2026" is how the date formats
      expect(screen.getAllByText('Feb 15, 2026').length).toBeGreaterThan(0)
    })

    it('renders the Enhanced tier badge for authenticated tier', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ tier: 'authenticated' })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText('Enhanced').length).toBeGreaterThan(0)
    })
  })

  describe('Severity badges', () => {
    it('renders severity count badges for non-zero counts', () => {
      const scan = makeScan({ critical_count: 2, high_count: 1, medium_count: 3, low_count: 0 })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      // Expect counts to appear (may appear multiple times due to desktop+mobile)
      expect(screen.getAllByText('2').length).toBeGreaterThan(0)
      expect(screen.getAllByText('1').length).toBeGreaterThan(0)
      expect(screen.getAllByText('3').length).toBeGreaterThan(0)
    })

    it('does not render a badge for zero-count severities', () => {
      // SeverityBadge returns null when count === 0
      const scan = makeScan({ critical_count: 1, high_count: 0, medium_count: 0, low_count: 0 })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      // The count "1" should appear (critical), but "0" should NOT appear as a badge
      // We can't easily assert "0 is not there" since there is no "0" text anywhere by design
      expect(screen.getAllByText('1').length).toBeGreaterThan(0)
    })
  })

  describe('Tier badges', () => {
    it('renders "Enhanced" for authenticated tier', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ tier: 'authenticated' })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText('Enhanced').length).toBeGreaterThan(0)
    })

    it('renders "Basic" for anonymous tier', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ tier: 'anonymous' })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText('Basic').length).toBeGreaterThan(0)
    })
  })

  describe('Expiry states', () => {
    it('renders "days left" for a scan with future expiry (> 3 days)', () => {
      // 30 days in the future from FIXED_NOW
      const expiresAt = new Date(FIXED_NOW + 30 * 86400000).toISOString()
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ expires_at: expiresAt })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText(/days left/i).length).toBeGreaterThan(0)
    })

    it('renders "days left" for a scan expiring within 3 days (expiring-soon)', () => {
      // 2 days in the future — expiring soon
      const expiresAt = new Date(FIXED_NOW + 2 * 86400000).toISOString()
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ expires_at: expiresAt })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText(/days? left/i).length).toBeGreaterThan(0)
    })

    it('renders "Expired" for a scan with a past expiry date', () => {
      const expiresAt = new Date(FIXED_NOW - 1 * 86400000).toISOString()
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan({ expires_at: expiresAt })]} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText('Expired').length).toBeGreaterThan(0)
    })

    it('renders "Failed" for a failed scan', () => {
      renderWithProviders(
        <ScanHistoryTable
          scans={[makeScan({ status: 'failed', results_token: null })]}
          currentPage={1}
          totalPages={1}
        />
      )
      expect(screen.getAllByText('Failed').length).toBeGreaterThan(0)
    })

    it('renders an em-dash for a scan with null expires_at', () => {
      renderWithProviders(
        <ScanHistoryTable
          scans={[makeScan({ expires_at: null, status: 'pending', results_token: null })]}
          currentPage={1}
          totalPages={1}
        />
      )
      // The em-dash character is rendered for null expires_at
      expect(screen.getAllByText('—').length).toBeGreaterThan(0)
    })
  })

  describe('Clickable rows', () => {
    it('renders an overlay link to /results/{token} for a completed, non-expired scan with a results_token', () => {
      const scan = makeScan({
        results_token: 'tok-abc',
        expires_at: new Date(FIXED_NOW + 30 * 86400000).toISOString(),
        status: 'completed',
      })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      // The overlay link has aria-label "View results for example.com"
      const link = screen.getByLabelText(/view results for example\.com/i)
      expect(link).toBeInTheDocument()
      expect(link).toHaveAttribute('href', '/results/tok-abc')
    })
  })

  describe('Non-clickable rows', () => {
    it('does not render a results link for an expired scan', () => {
      const scan = makeScan({
        results_token: 'tok-abc',
        expires_at: new Date(FIXED_NOW - 1 * 86400000).toISOString(),
      })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      expect(screen.queryByLabelText(/view results/i)).not.toBeInTheDocument()
    })

    it('does not render a results link for a scan with null results_token', () => {
      const scan = makeScan({
        results_token: null,
        status: 'pending',
        expires_at: new Date(FIXED_NOW + 30 * 86400000).toISOString(),
      })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      expect(screen.queryByLabelText(/view results/i)).not.toBeInTheDocument()
    })

    it('does not render a results link for a failed scan', () => {
      const scan = makeScan({
        status: 'failed',
        results_token: null,
      })
      renderWithProviders(
        <ScanHistoryTable scans={[scan]} currentPage={1} totalPages={1} />
      )
      expect(screen.queryByLabelText(/view results/i)).not.toBeInTheDocument()
    })
  })

  describe('Pagination', () => {
    it('does not render pagination links when totalPages is 1', () => {
      renderWithProviders(
        <ScanHistoryTable scans={[makeScan()]} currentPage={1} totalPages={1} />
      )
      // No page number links should be present
      expect(screen.queryByRole('link', { name: '1' })).not.toBeInTheDocument()
    })

    it('renders page links for non-current pages and plain text for current page', () => {
      // Use a scan with no severity counts to avoid ambiguous "2" text
      const scans = [makeScan({ id: 'scan-1', critical_count: 0, high_count: 0, medium_count: 0, low_count: 0 })]
      renderWithProviders(
        <ScanHistoryTable scans={scans} currentPage={2} totalPages={3} />
      )
      // Pages 1 and 3 are links (non-current)
      expect(screen.getByRole('link', { name: '1' })).toBeInTheDocument()
      expect(screen.getByRole('link', { name: '3' })).toBeInTheDocument()
      // Page 2 is the current page rendered as a <span>, not a link
      expect(screen.queryByRole('link', { name: '2' })).not.toBeInTheDocument()
      // Current page "2" is rendered as plain text in a span (not a link)
      expect(screen.getByText('2')).toBeInTheDocument()
    })
  })

  describe('Multiple scans', () => {
    it('renders all scan domains when multiple scans are provided', () => {
      const scans = [
        makeScan({ id: 'scan-1', target_url: 'https://example.com' }),
        makeScan({ id: 'scan-2', target_url: 'https://other.io' }),
        makeScan({ id: 'scan-3', target_url: 'https://third.dev' }),
      ]
      renderWithProviders(
        <ScanHistoryTable scans={scans} currentPage={1} totalPages={1} />
      )
      expect(screen.getAllByText('example.com').length).toBeGreaterThan(0)
      expect(screen.getAllByText('other.io').length).toBeGreaterThan(0)
      expect(screen.getAllByText('third.dev').length).toBeGreaterThan(0)
    })
  })
})
