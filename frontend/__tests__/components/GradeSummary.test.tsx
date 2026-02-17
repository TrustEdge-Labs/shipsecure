import { describe, it, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '../helpers/test-utils'
import { GradeSummary } from '@/components/grade-summary'

const defaultSummary = {
  total: 3,
  critical: 1,
  high: 1,
  medium: 1,
  low: 0,
}

describe('GradeSummary', () => {
  describe('Grade Display', () => {
    it('renders grade letter "A"', () => {
      renderWithProviders(
        <GradeSummary
          grade="A"
          summary={{ total: 0, critical: 0, high: 0, medium: 0, low: 0 }}
        />
      )
      expect(screen.getByText('A')).toBeInTheDocument()
    })

    it('renders grade letter "B"', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={{ total: 0, critical: 0, high: 0, medium: 0, low: 0 }}
        />
      )
      expect(screen.getByText('B')).toBeInTheDocument()
    })

    it('renders grade letter "F"', () => {
      renderWithProviders(
        <GradeSummary
          grade="F"
          summary={{ total: 0, critical: 0, high: 0, medium: 0, low: 0 }}
        />
      )
      expect(screen.getByText('F')).toBeInTheDocument()
    })

    it('renders grade "A+"', () => {
      renderWithProviders(
        <GradeSummary
          grade="A+"
          summary={{ total: 0, critical: 0, high: 0, medium: 0, low: 0 }}
        />
      )
      expect(screen.getByText('A+')).toBeInTheDocument()
    })
  })

  describe('Severity Counts', () => {
    it('shows severity badges when counts > 0', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={{ total: 5, critical: 1, high: 2, medium: 1, low: 1 }}
        />
      )

      expect(screen.getByText('1 Critical')).toBeInTheDocument()
      expect(screen.getByText('2 High')).toBeInTheDocument()
      expect(screen.getByText('1 Medium')).toBeInTheDocument()
      expect(screen.getByText('1 Low')).toBeInTheDocument()
    })

    it('hides severity badge when count is 0', () => {
      renderWithProviders(
        <GradeSummary
          grade="A"
          summary={{ total: 1, critical: 0, high: 1, medium: 0, low: 0 }}
        />
      )

      expect(screen.queryByText(/Critical/)).not.toBeInTheDocument()
      expect(screen.queryByText(/Medium/)).not.toBeInTheDocument()
      expect(screen.queryByText(/Low/)).not.toBeInTheDocument()
      expect(screen.getByText('1 High')).toBeInTheDocument()
    })

    it('shows total findings count with plural', () => {
      renderWithProviders(
        <GradeSummary grade="B" summary={defaultSummary} />
      )
      expect(screen.getByText('3 findings')).toBeInTheDocument()
    })

    it('shows total findings count with singular', () => {
      renderWithProviders(
        <GradeSummary
          grade="A"
          summary={{ total: 1, critical: 0, high: 1, medium: 0, low: 0 }}
        />
      )
      expect(screen.getByText('1 finding')).toBeInTheDocument()
    })
  })

  describe('Framework/Platform Display', () => {
    it('shows framework name when detected', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={defaultSummary}
          framework="nextjs"
        />
      )
      expect(screen.getByText('Next.js')).toBeInTheDocument()
    })

    it('shows platform name when detected', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={defaultSummary}
          platform="vercel"
        />
      )
      expect(screen.getByText('Vercel')).toBeInTheDocument()
    })

    it('shows both framework and platform when both detected', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={defaultSummary}
          framework="nextjs"
          platform="vercel"
        />
      )
      expect(screen.getByText(/Next\.js/)).toBeInTheDocument()
      expect(screen.getByText(/on/)).toBeInTheDocument()
      expect(screen.getByText(/Vercel/)).toBeInTheDocument()
    })

    it('shows "Framework: Not detected" when framework is null', () => {
      renderWithProviders(
        <GradeSummary
          grade="B"
          summary={defaultSummary}
          framework={null}
        />
      )
      expect(screen.getByText('Framework: Not detected')).toBeInTheDocument()
    })
  })
})
