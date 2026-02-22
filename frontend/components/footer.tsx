import Link from 'next/link'

export function Footer() {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="border-t border-border-subtle py-8 mt-auto">
      <div className="container mx-auto px-4">
        <nav className="flex flex-col sm:flex-row items-center justify-center gap-2 sm:gap-4 mb-3">
          <Link
            href="/"
            className="text-sm text-text-tertiary hover:text-brand-primary transition"
          >
            Scan
          </Link>
          <span className="hidden sm:inline text-text-muted">&middot;</span>
          <Link
            href="/privacy"
            className="text-sm text-text-tertiary hover:text-brand-primary transition"
          >
            Privacy Policy
          </Link>
          <span className="hidden sm:inline text-text-muted">&middot;</span>
          <Link
            href="/terms"
            className="text-sm text-text-tertiary hover:text-brand-primary transition"
          >
            Terms of Service
          </Link>
          <span className="hidden sm:inline text-text-muted">&middot;</span>
          <a
            href="mailto:support@shipsecure.ai"
            className="text-sm text-text-tertiary hover:text-brand-primary transition"
          >
            Contact
          </a>
        </nav>
        <p className="text-center text-sm text-text-tertiary">
          &copy; {currentYear} ShipSecure. All rights reserved.
        </p>
        <div className="mt-4 text-center text-xs text-text-muted">
          <p className="mb-1">Powered by open source:</p>
          <div className="flex flex-wrap justify-center gap-x-3 gap-y-1">
            <span>
              <a href="https://github.com/projectdiscovery/nuclei" target="_blank" rel="noopener noreferrer" className="hover:text-brand-primary underline">Nuclei</a>
              {' '}by{' '}
              <a href="https://projectdiscovery.io" target="_blank" rel="noopener noreferrer" className="hover:text-brand-primary">ProjectDiscovery</a>
              {' '}(<a href="https://github.com/projectdiscovery/nuclei/blob/main/LICENSE.md" target="_blank" rel="noopener noreferrer" className="hover:text-brand-primary">MIT</a>)
            </span>
            <span className="hidden sm:inline text-text-muted">&middot;</span>
            <span>
              <a href="https://testssl.sh" target="_blank" rel="noopener noreferrer" className="hover:text-brand-primary underline">testssl.sh</a>
              {' '}(<a href="https://github.com/testssl/testssl.sh/blob/3.3dev/LICENSE" target="_blank" rel="noopener noreferrer" className="hover:text-brand-primary">GPLv2</a>)
            </span>
          </div>
        </div>
      </div>
    </footer>
  )
}
