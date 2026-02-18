import { vi, describe, test, expect, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import { renderWithProviders } from '@/__tests__/helpers/test-utils'
import { ScanForm } from '@/components/scan-form'
import { ResultsDashboard } from '@/components/results-dashboard'
import { GradeSummary } from '@/components/grade-summary'
import { FindingAccordion } from '@/components/finding-accordion'
import { ProgressChecklist } from '@/components/progress-checklist'
import { Header } from '@/components/header'
import { Footer } from '@/components/footer'
import { Logo } from '@/components/logo'

// Mock state for useActionState (required by ScanForm)
let mockState = {} as any
let mockFormAction = vi.fn()
let mockPending = false

vi.mock('react', async () => {
  const actual = await vi.importActual('react')
  return {
    ...actual,
    useActionState: vi.fn(() => [mockState, mockFormAction, mockPending])
  }
})

// Helper to mock the prefers-color-scheme media query
function mockColorScheme(scheme: 'dark' | 'light') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    configurable: true,
    value: vi.fn((query: string) => ({
      matches: query === `(prefers-color-scheme: ${scheme})`,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  })
}

// Shared finding fixture
const mockFinding = {
  id: '1',
  title: 'Test Finding',
  description: 'Desc',
  severity: 'high' as const,
  remediation: 'Fix it',
  scanner_name: 'security_headers',
  vibe_code: false,
}

// Shared stages fixture
const mockStages = {
  detection: true,
  headers: false,
  tls: false,
  files: false,
  secrets: false,
  vibecode: false,
}

// Shared grade summary fixture
const mockSummary = {
  total: 1,
  critical: 0,
  high: 1,
  medium: 0,
  low: 0,
}

beforeEach(() => {
  mockState = {}
  mockFormAction = vi.fn()
  mockPending = false
})

describe('Dark mode rendering', () => {
  beforeEach(() => {
    mockColorScheme('dark')
  })

  test('ScanForm renders in dark mode', () => {
    renderWithProviders(<ScanForm />)
    const urlLabel = screen.getByText(/website url/i)
    expect(urlLabel).toBeInTheDocument()
  })

  test('ResultsDashboard renders in dark mode', () => {
    renderWithProviders(<ResultsDashboard findings={[]} />)
    const emptyText = screen.getByText(/no security issues found/i)
    expect(emptyText).toBeInTheDocument()
  })

  test('GradeSummary renders in dark mode', () => {
    renderWithProviders(<GradeSummary grade="B" summary={mockSummary} />)
    const grade = screen.getByText('B')
    expect(grade).toBeInTheDocument()
  })

  test('FindingAccordion renders in dark mode', () => {
    renderWithProviders(<FindingAccordion finding={mockFinding} />)
    const title = screen.getByText('Test Finding')
    expect(title).toBeInTheDocument()
  })

  test('ProgressChecklist renders in dark mode', () => {
    renderWithProviders(<ProgressChecklist stages={mockStages} status="in_progress" />)
    const label = screen.getByText(/detecting framework/i)
    expect(label).toBeInTheDocument()
  })

  test('Header renders in dark mode', () => {
    renderWithProviders(<Header />)
    const nav = screen.getByRole('navigation')
    expect(nav).toBeInTheDocument()
  })

  test('Footer renders in dark mode', () => {
    renderWithProviders(<Footer />)
    const copyright = screen.getByText(/shipsecure/i)
    expect(copyright).toBeInTheDocument()
  })

  test('Logo renders in dark mode', () => {
    renderWithProviders(<Logo size="small" />)
    const img = screen.getByAltText('ShipSecure')
    expect(img).toBeInTheDocument()
  })
})

describe('Light mode rendering', () => {
  beforeEach(() => {
    mockColorScheme('light')
  })

  test('ScanForm renders in light mode', () => {
    renderWithProviders(<ScanForm />)
    const urlLabel = screen.getByText(/website url/i)
    expect(urlLabel).toBeInTheDocument()
  })

  test('ResultsDashboard renders in light mode', () => {
    renderWithProviders(<ResultsDashboard findings={[]} />)
    const emptyText = screen.getByText(/no security issues found/i)
    expect(emptyText).toBeInTheDocument()
  })

  test('GradeSummary renders in light mode', () => {
    renderWithProviders(<GradeSummary grade="B" summary={mockSummary} />)
    const grade = screen.getByText('B')
    expect(grade).toBeInTheDocument()
  })

  test('FindingAccordion renders in light mode', () => {
    renderWithProviders(<FindingAccordion finding={mockFinding} />)
    const title = screen.getByText('Test Finding')
    expect(title).toBeInTheDocument()
  })

  test('ProgressChecklist renders in light mode', () => {
    renderWithProviders(<ProgressChecklist stages={mockStages} status="in_progress" />)
    const label = screen.getByText(/detecting framework/i)
    expect(label).toBeInTheDocument()
  })

  test('Header renders in light mode', () => {
    renderWithProviders(<Header />)
    const nav = screen.getByRole('navigation')
    expect(nav).toBeInTheDocument()
  })

  test('Footer renders in light mode', () => {
    renderWithProviders(<Footer />)
    const copyright = screen.getByText(/shipsecure/i)
    expect(copyright).toBeInTheDocument()
  })

  test('Logo renders in light mode', () => {
    renderWithProviders(<Logo size="small" />)
    const img = screen.getByAltText('ShipSecure')
    expect(img).toBeInTheDocument()
  })
})
