import { describe, expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { Logo } from '@/components/logo'

describe('Logo', () => {
  describe('Size Variants', () => {
    test('renders small variant', () => {
      renderWithProviders(<Logo size="small" />)

      const img = screen.getByAltText('ShipSecure')
      expect(img).toBeInTheDocument()
      expect(img).toHaveAttribute('width', '96')
      expect(img).toHaveAttribute('height', '64')
    })

    test('renders medium variant', () => {
      renderWithProviders(<Logo size="medium" />)

      const img = screen.getByAltText('ShipSecure')
      expect(img).toBeInTheDocument()
      expect(img).toHaveAttribute('width', '384')
      expect(img).toHaveAttribute('height', '256')
    })

    test('renders large variant', () => {
      renderWithProviders(<Logo size="large" />)

      const img = screen.getByAltText('ShipSecure')
      expect(img).toBeInTheDocument()
      expect(img).toHaveAttribute('width', '768')
      expect(img).toHaveAttribute('height', '512')
    })
  })

  describe('Common Attributes', () => {
    test('all variants use correct src', () => {
      const { rerender } = renderWithProviders(<Logo size="small" />)

      let img = screen.getByAltText('ShipSecure')
      expect(img).toHaveAttribute('src', '/logo.png')

      rerender(<Logo size="medium" />)
      img = screen.getByAltText('ShipSecure')
      expect(img).toHaveAttribute('src', '/logo.png')

      rerender(<Logo size="large" />)
      img = screen.getByAltText('ShipSecure')
      expect(img).toHaveAttribute('src', '/logo.png')
    })

    test('passes className prop', () => {
      renderWithProviders(<Logo size="small" className="test-class" />)

      const img = screen.getByAltText('ShipSecure')
      expect(img).toHaveClass('test-class')
    })
  })
})
