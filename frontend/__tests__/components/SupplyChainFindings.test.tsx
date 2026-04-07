import { describe, test, expect, vi, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { SupplyChainFindings } from '@/components/supply-chain-findings'
import type { SupplyChainResults } from '@/lib/supply-chain-types'

describe('SupplyChainFindings', () => {
  beforeEach(() => {
    window.plausible = vi.fn()
  })

  test("shows 'No compromised packages found' when no findings", () => {
    const fixture: SupplyChainResults = {
      total_deps: 1,
      infected: [],
      vulnerable: [],
      advisory: [],
      no_known_issues: ['safe@1.0.0'],
      unscanned: [],
      scanned_at: '2026-01-01T00:00:00',
    }

    renderWithProviders(<SupplyChainFindings results={fixture} />)

    expect(screen.getByText('No compromised packages found')).toBeInTheDocument()
    expect(screen.getByText(/1 package checked, all clear/i)).toBeInTheDocument()
  })
})
