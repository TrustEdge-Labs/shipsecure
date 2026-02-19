import { describe, expect, test } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { Header } from '@/components/header'

describe('Header', () => {
  test('renders logo image', () => {
    renderWithProviders(<Header />)

    // Header has two logos (desktop and mobile responsive variants)
    const logos = screen.getAllByAltText('ShipSecure')
    expect(logos.length).toBeGreaterThan(0)
  })

  test('renders navigation landmark', () => {
    renderWithProviders(<Header />)

    const nav = screen.getByRole('navigation', { name: /main navigation/i })
    expect(nav).toBeInTheDocument()
  })

  test('renders Sign In link when signed out', () => {
    renderWithProviders(<Header />)

    const cta = screen.getByRole('link', { name: /sign in/i })
    expect(cta).toBeInTheDocument()
    expect(cta).toHaveAttribute('href', '/sign-in')
  })

  test('renders logo as link to home', () => {
    renderWithProviders(<Header />)

    // The logo is wrapped in a Link to "/"
    const homeLinks = screen.getAllByRole('link')
    const homeLink = homeLinks.find(link => link.getAttribute('href') === '/')
    expect(homeLink).toBeDefined()
  })
})
