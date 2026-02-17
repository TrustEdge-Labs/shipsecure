import { describe, expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ProgressChecklist } from '@/components/progress-checklist'

describe('ProgressChecklist', () => {
  describe('Stage Labels', () => {
    test('renders all 6 stage labels', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: false,
            headers: false,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      expect(screen.getByText('Detecting Framework')).toBeInTheDocument()
      expect(screen.getByText('Security Headers')).toBeInTheDocument()
      expect(screen.getByText('TLS Configuration')).toBeInTheDocument()
      expect(screen.getByText('Exposed Files')).toBeInTheDocument()
      expect(screen.getByText('JavaScript Secrets')).toBeInTheDocument()
      expect(screen.getByText('Vibe-Code Scan')).toBeInTheDocument()
    })
  })

  describe('Completed Stages', () => {
    test('shows checkmark for completed stages', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // Check for checkmark characters (✓) in the document
      const checkmarks = screen.getAllByText('✓')
      expect(checkmarks).toHaveLength(2)
    })

    test('shows pending indicator for incomplete stages', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // Check for pending indicator characters (○)
      const pendingIndicators = screen.getAllByText('○')
      expect(pendingIndicators).toHaveLength(4)
    })
  })

  describe('Active Stage Description', () => {
    test('shows description for active stage (first incomplete)', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // TLS Configuration is the first incomplete stage, should show its description
      expect(
        screen.getByText(/Analyzing certificate validity/)
      ).toBeInTheDocument()
    })

    test('does NOT show descriptions for non-active incomplete stages', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // Files stage description should NOT be visible
      expect(screen.queryByText(/Probing for \.env/)).not.toBeInTheDocument()
    })
  })

  describe('State Transitions via Rerender', () => {
    test('re-render with more stages complete updates display', () => {
      const { rerender } = renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // Initially should have 2 checkmarks
      let checkmarks = screen.getAllByText('✓')
      expect(checkmarks).toHaveLength(2)

      // Re-render with 4 stages done
      rerender(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: true,
            tls: true,
            files: true,
            secrets: false,
            vibecode: false,
          }}
          status="in_progress"
        />
      )

      // Now should have 4 checkmarks
      checkmarks = screen.getAllByText('✓')
      expect(checkmarks).toHaveLength(4)
    })

    test('failed status shows failure indicators', () => {
      renderWithProviders(
        <ProgressChecklist
          stages={{
            detection: true,
            headers: false,
            tls: false,
            files: false,
            secrets: false,
            vibecode: false,
          }}
          status="failed"
        />
      )

      // Should show ✗ failure indicators
      const failureIndicators = screen.getAllByText('✗')
      expect(failureIndicators.length).toBeGreaterThan(0)
    })
  })
})
