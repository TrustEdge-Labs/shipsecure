import { describe, test, expect } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import Loading from '@/app/loading'
import ResultsLoading from '@/app/results/[token]/loading'

describe('Root Loading', () => {
  test('renders loading spinner text', () => {
    renderWithProviders(<Loading />)
    const loadingText = screen.getByText(/loading/i)
    expect(loadingText).toBeInTheDocument()
  })

  test('renders without errors', () => {
    const { container } = renderWithProviders(<Loading />)
    expect(container.firstChild).toBeInTheDocument()
  })
})

describe('Results Loading Skeleton', () => {
  test('renders skeleton structure without errors', () => {
    const { container } = renderWithProviders(<ResultsLoading />)
    expect(container.firstChild).toBeInTheDocument()
  })

  test('contains animate-pulse skeleton elements', () => {
    const { container } = renderWithProviders(<ResultsLoading />)
    const skeletonElements = container.querySelectorAll('.animate-pulse')
    expect(skeletonElements.length).toBeGreaterThan(0)
  })
})
