import { describe, test, expect, vi, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { SupplyChainSummary } from '@/components/supply-chain-summary'
import type { SupplyChainResults } from '@/lib/supply-chain-types'

const fixture: SupplyChainResults = {
  total_deps: 5,
  infected: [
    {
      name: 'evil',
      version: '1.0.0',
      osv_id: 'MAL-001',
      description: 'Malware',
      tier: 'Infected',
    },
  ],
  vulnerable: [],
  advisory: [
    {
      name: 'old-lib',
      version: '2.0.0',
      osv_id: 'GHSA-1234',
      description: 'Moderate',
      tier: 'Advisory',
    },
  ],
  no_known_issues: ['safe@1.0.0', 'good@2.0.0'],
  unscanned: [
    {
      name: 'git-dep',
      version: '3.0.0',
      source: 'Git',
      is_dev: false,
    },
  ],
  scanned_at: '2026-01-01T00:00:00',
}

describe('SupplyChainSummary', () => {
  beforeEach(() => {
    window.plausible = vi.fn()
  })

  test('renders summary cards with correct counts', () => {
    renderWithProviders(<SupplyChainSummary results={fixture} />)

    // All 5 card labels are present
    expect(screen.getByText('Infected')).toBeInTheDocument()
    expect(screen.getByText('Vulnerable')).toBeInTheDocument()
    expect(screen.getByText('Advisory')).toBeInTheDocument()
    expect(screen.getByText('No Known Issues')).toBeInTheDocument()
    expect(screen.getByText('Unscanned')).toBeInTheDocument()

    // Counts — use getAllByText for '1' since it appears multiple times
    const ones = screen.getAllByText('1')
    // Infected = 1, Advisory = 1, Unscanned = 1 → at least 3 occurrences
    expect(ones.length).toBeGreaterThanOrEqual(3)

    expect(screen.getByText('2')).toBeInTheDocument() // No Known Issues
    expect(screen.getByText('0')).toBeInTheDocument() // Vulnerable
  })
})
