import { describe, expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { Footer } from '@/components/footer'

describe('Footer', () => {
  describe('Legal Links', () => {
    test('renders Privacy Policy link', () => {
      renderWithProviders(<Footer />)

      const privacyLink = screen.getByRole('link', { name: /privacy policy/i })
      expect(privacyLink).toBeInTheDocument()
      expect(privacyLink).toHaveAttribute('href', '/privacy')
    })

    test('renders Terms of Service link', () => {
      renderWithProviders(<Footer />)

      const termsLink = screen.getByRole('link', { name: /terms of service/i })
      expect(termsLink).toBeInTheDocument()
      expect(termsLink).toHaveAttribute('href', '/terms')
    })
  })

  describe('Copyright', () => {
    test('renders copyright with current year', () => {
      renderWithProviders(<Footer />)

      const currentYear = new Date().getFullYear()
      const copyrightPattern = new RegExp(String(currentYear))

      expect(screen.getByText(copyrightPattern)).toBeInTheDocument()
    })
  })

  describe('OSS Attribution', () => {
    test('renders "Powered by open source" text', () => {
      renderWithProviders(<Footer />)

      expect(screen.getByText(/powered by open source/i)).toBeInTheDocument()
    })

    test('renders Nuclei attribution link', () => {
      renderWithProviders(<Footer />)

      const nucleiLink = screen.getByRole('link', { name: /^nuclei$/i })
      expect(nucleiLink).toBeInTheDocument()
      expect(nucleiLink).toHaveAttribute(
        'href',
        'https://github.com/projectdiscovery/nuclei'
      )
    })

    test('renders testssl.sh attribution link', () => {
      renderWithProviders(<Footer />)

      const testsslLink = screen.getByRole('link', { name: /testssl\.sh/i })
      expect(testsslLink).toBeInTheDocument()
      expect(testsslLink).toHaveAttribute('href', 'https://testssl.sh')
    })
  })
})
