import Link from 'next/link'
import Image from 'next/image'
import { SignedIn, SignedOut, UserButton } from '@clerk/nextjs'

export function Header() {
  return (
    <header className="sticky top-0 z-50 bg-surface-primary border-b border-border-subtle">
      <nav aria-label="Main navigation">
        <div className="container mx-auto px-4 h-[var(--header-height)] flex items-center justify-between">
          {/* Logo - responsive: wordmark on desktop, icon on mobile */}
          <Link href="/" className="flex items-center">
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

          {/* Auth CTA */}
          <SignedOut>
            <Link
              href="/sign-in"
              className="px-4 py-2 bg-brand-primary hover:bg-brand-primary-hover text-text-inverse font-semibold rounded-lg transition text-sm sm:text-base"
            >
              Sign In
            </Link>
          </SignedOut>
          <SignedIn>
            <UserButton />
          </SignedIn>
        </div>
      </nav>
    </header>
  )
}
