import { describe, test, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ShareButton } from '@/components/share-button'

describe('ShareButton', () => {
  beforeEach(() => {
    vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined)
    window.plausible = vi.fn()
  })

  test('renders Copy Link button', () => {
    render(<ShareButton url="https://shipsecure.ai/results/abc123" />)
    expect(screen.getByText('Copy Link')).toBeInTheDocument()
  })

  test('copies URL to clipboard on click', async () => {
    render(<ShareButton url="https://shipsecure.ai/results/abc123" />)
    fireEvent.click(screen.getByRole('button'))
    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith('https://shipsecure.ai/results/abc123')
    })
  })

  test('shows Copied! after click', async () => {
    render(<ShareButton url="https://shipsecure.ai/results/abc123" />)
    fireEvent.click(screen.getByRole('button'))
    await waitFor(() => {
      expect(screen.getByText('Copied!')).toBeInTheDocument()
    })
  })

  test('fires Share Clicked plausible event', async () => {
    render(<ShareButton url="https://shipsecure.ai/results/abc123" />)
    fireEvent.click(screen.getByRole('button'))
    await waitFor(() => {
      expect(window.plausible).toHaveBeenCalledWith('Share Clicked')
    })
  })

  test('has accessible label', () => {
    render(<ShareButton url="https://shipsecure.ai/results/abc123" />)
    expect(screen.getByLabelText('Copy results link to clipboard')).toBeInTheDocument()
  })
})
