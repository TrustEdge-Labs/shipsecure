'use client'

interface GradeSummaryProps {
  grade: string
  summary: {
    critical: number
    high: number
    medium: number
    low: number
    total: number
  }
  framework?: string | null
  platform?: string | null
}

export function GradeSummary({ grade, summary, framework, platform }: GradeSummaryProps) {
  const getGradeColor = (grade: string) => {
    const normalized = grade.toUpperCase()
    if (normalized === 'A+' || normalized === 'A') return 'text-grade-a-text bg-grade-a-bg'
    if (normalized === 'B') return 'text-grade-b-text bg-grade-b-bg'
    if (normalized === 'C') return 'text-grade-c-text bg-grade-c-bg'
    return 'text-grade-df-text bg-grade-df-bg' // D or F
  }

  const formatFramework = (fw: string): string => {
    const mapping: Record<string, string> = {
      nextjs: 'Next.js',
      vite_react: 'Vite/React',
      sveltekit: 'SvelteKit',
      nuxt: 'Nuxt',
    }
    return mapping[fw] || fw
  }

  const formatPlatform = (pl: string): string => {
    const mapping: Record<string, string> = {
      vercel: 'Vercel',
      netlify: 'Netlify',
      railway: 'Railway',
    }
    return mapping[pl] || pl
  }

  return (
    <div className="flex flex-col sm:flex-row sm:items-center gap-4 sm:gap-6 p-4 bg-surface-secondary rounded-lg border border-border-subtle">
      {/* Grade Circle */}
      <div className={`flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center text-xl font-bold ${getGradeColor(grade)}`}>
        {grade}
      </div>

      {/* Framework + Platform Badge */}
      <div className="flex flex-col">
        <span className="text-lg font-bold text-text-primary">
          {(framework || platform) && (
            <span className="text-base font-normal text-text-secondary">
              {framework ? formatFramework(framework) : ''}
              {framework && platform ? ' on ' : ''}
              {platform ? formatPlatform(platform) : ''}
            </span>
          )}
        </span>
        {!framework && (
          <span className="text-xs text-text-tertiary">
            Framework: Not detected
          </span>
        )}
      </div>

      {/* Finding Counts */}
      <div className="flex flex-wrap gap-3 items-center w-full sm:flex-1 mt-2 sm:mt-0">
        {summary.critical > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-severity-critical-bg text-severity-critical-text">
            {summary.critical} Critical
          </span>
        )}
        {summary.high > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-severity-high-bg text-severity-high-text">
            {summary.high} High
          </span>
        )}
        {summary.medium > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-severity-medium-bg text-severity-medium-text">
            {summary.medium} Medium
          </span>
        )}
        {summary.low > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-severity-info-bg text-severity-info-text">
            {summary.low} Low
          </span>
        )}

        <span className="text-sm text-text-secondary font-medium ml-2">
          {summary.total} {summary.total === 1 ? 'finding' : 'findings'}
        </span>
      </div>
    </div>
  )
}
