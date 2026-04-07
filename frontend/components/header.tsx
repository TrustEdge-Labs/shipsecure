import Link from 'next/link'
import Image from 'next/image'
import { SignedIn, SignedOut, UserButton } from '@clerk/nextjs'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <nav aria-label="Main navigation">
        <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
          {/* Logo - responsive: wordmark on desktop, icon on mobile */}
          <Link href="/" className="flex items-center p-2 -m-2">
            <div className="hidden sm:block">
              <Image
                src="/logo.png"
                alt="ShipSecure"
                width={160}
                height={48}
                priority
                className="object-contain"
              />
            </div>
            <div className="sm:hidden">
              <Image
                src="/logo.png"
                alt="ShipSecure"
                width={40}
                height={40}
                priority
                className="object-contain"
              />
            </div>
          </Link>

          {/* Navigation + Auth */}
          <div className="flex items-center gap-4 sm:gap-6">
            {/* Supply Chain — visible to all users */}
            <Link
              href="/supply-chain"
              className="inline-flex items-center min-h-[44px] text-sm font-medium text-text-secondary hover:text-text-primary transition"
            >
              Supply Chain
            </Link>
            <SignedIn>
              <Link
                href="/dashboard"
                className="inline-flex items-center min-h-[44px] text-sm font-medium text-text-secondary hover:text-text-primary transition"
              >
                Dashboard
              </Link>
              <Link
                href="/#scan-form"
                className="hidden sm:inline-flex items-center min-h-[44px] text-sm font-medium text-text-secondary hover:text-text-primary transition"
              >
                New Scan
              </Link>
              <UserButton />
            </SignedIn>
            <SignedOut>
              <Link
                href="/sign-in"
                className="inline-flex items-center min-h-[44px] text-sm font-medium text-text-secondary hover:text-text-primary transition"
              >
                Sign In
              </Link>
              <Link
                href="/sign-up"
                className="inline-flex items-center min-h-[44px] px-4 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm sm:text-base"
              >
                Sign Up
              </Link>
            </SignedOut>
          </div>
        </div>
      </nav>
    </header>
  )
}
