import { render, RenderOptions } from '@testing-library/react'
import { ReactElement, ReactNode } from 'react'

interface AllTheProvidersProps {
  children: ReactNode
}

/**
 * Wrapper component with all providers.
 * Add providers here as the app grows (e.g., ThemeProvider, QueryClientProvider).
 */
function AllTheProviders({ children }: AllTheProvidersProps) {
  return <>{children}</>
}

/**
 * Custom render that wraps components in all required providers.
 * Use this instead of RTL's render() in all component tests.
 */
export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllTheProviders, ...options })
}

// Re-export everything from RTL for convenience
export * from '@testing-library/react'
