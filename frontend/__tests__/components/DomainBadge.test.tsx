import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { DomainBadge } from '@/components/domain-badge'

describe('DomainBadge', () => {
  describe('Verified state', () => {
    it('renders "Verified" when status is verified and expiry is more than 7 days away', () => {
      const futureDate = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString()
      renderWithProviders(<DomainBadge status="verified" expiresAt={futureDate} />)
      expect(screen.getByText('Verified')).toBeInTheDocument()
    })
  })

  describe('Expiring soon state', () => {
    it('renders "Expires in 3d" when expiry is 3 days away', () => {
      const soonDate = new Date(Date.now() + 3 * 24 * 60 * 60 * 1000).toISOString()
      renderWithProviders(<DomainBadge status="verified" expiresAt={soonDate} />)
      expect(screen.getByText('Expires in 3d')).toBeInTheDocument()
    })

    it('renders "Expires in 1d" when expiry is 1 day away', () => {
      const soonDate = new Date(Date.now() + 1 * 24 * 60 * 60 * 1000).toISOString()
      renderWithProviders(<DomainBadge status="verified" expiresAt={soonDate} />)
      expect(screen.getByText('Expires in 1d')).toBeInTheDocument()
    })
  })

  describe('Expired state', () => {
    it('renders "Expired" when expiry date is in the past', () => {
      const pastDate = new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString()
      renderWithProviders(<DomainBadge status="verified" expiresAt={pastDate} />)
      expect(screen.getByText('Expired')).toBeInTheDocument()
    })
  })

  describe('Pending state', () => {
    it('renders "Pending" when status is pending', () => {
      renderWithProviders(<DomainBadge status="pending" expiresAt={null} />)
      expect(screen.getByText('Pending')).toBeInTheDocument()
    })
  })

  describe('Edge cases', () => {
    it('renders "Pending" when status is verified but expiresAt is null (fallback)', () => {
      renderWithProviders(<DomainBadge status="verified" expiresAt={null} />)
      expect(screen.getByText('Pending')).toBeInTheDocument()
    })
  })
})
