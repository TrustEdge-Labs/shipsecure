import { describe, it, expect, vi, afterEach } from 'vitest'
import { screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { renderWithProviders } from '../helpers/test-utils'
import { MetaTagSnippet } from '@/components/meta-tag-snippet'

const SAMPLE_META_TAG = '<meta name="shipsecure-verification" content="abc123" />'

describe('MetaTagSnippet', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('Rendering', () => {
    it('renders the meta tag text', () => {
      renderWithProviders(<MetaTagSnippet metaTag={SAMPLE_META_TAG} />)
      expect(screen.getByText(SAMPLE_META_TAG)).toBeInTheDocument()
    })

    it('renders copy button with accessible label', () => {
      renderWithProviders(<MetaTagSnippet metaTag={SAMPLE_META_TAG} />)
      expect(screen.getByLabelText('Copy meta tag')).toBeInTheDocument()
    })
  })

  describe('Copy interaction', () => {
    it('calls navigator.clipboard.writeText with the meta tag string when copy button is clicked', async () => {
      // fireEvent.click is used here because userEvent simulates the full browser pointer
      // event sequence which causes the clipboard Permissions API to reject in happy-dom.
      // fireEvent.click directly dispatches the click event, avoiding the security context issue.
      const writeTextSpy = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined)
      renderWithProviders(<MetaTagSnippet metaTag={SAMPLE_META_TAG} />)

      const copyButton = screen.getByLabelText('Copy meta tag')
      fireEvent.click(copyButton)

      // Allow the async handleCopy to resolve
      await new Promise(resolve => setTimeout(resolve, 50))

      expect(writeTextSpy).toHaveBeenCalledWith(SAMPLE_META_TAG)
      expect(writeTextSpy).toHaveBeenCalledTimes(1)
    })

    it('copy button remains in the document after clicking (component does not crash)', async () => {
      const user = userEvent.setup()
      renderWithProviders(<MetaTagSnippet metaTag={SAMPLE_META_TAG} />)

      const copyButton = screen.getByLabelText('Copy meta tag')
      await user.click(copyButton)

      expect(screen.getByLabelText('Copy meta tag')).toBeInTheDocument()
    })
  })
})
