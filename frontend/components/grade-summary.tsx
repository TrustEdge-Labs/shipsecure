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
}

export function GradeSummary({ grade, summary }: GradeSummaryProps) {
  const getGradeColor = (grade: string) => {
    const normalized = grade.toUpperCase()
    if (normalized === 'A+' || normalized === 'A') return 'text-green-600 dark:text-green-400 bg-green-100 dark:bg-green-900'
    if (normalized === 'B') return 'text-yellow-600 dark:text-yellow-400 bg-yellow-100 dark:bg-yellow-900'
    if (normalized === 'C') return 'text-orange-600 dark:text-orange-400 bg-orange-100 dark:bg-orange-900'
    return 'text-red-600 dark:text-red-400 bg-red-100 dark:bg-red-900' // D or F
  }

  return (
    <div className="flex items-center gap-6 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
      {/* Grade Circle */}
      <div className={`flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center text-xl font-bold ${getGradeColor(grade)}`}>
        {grade}
      </div>

      {/* Finding Counts */}
      <div className="flex flex-wrap gap-3 items-center flex-1">
        {summary.critical > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300">
            {summary.critical} Critical
          </span>
        )}
        {summary.high > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-orange-100 dark:bg-orange-900 text-orange-700 dark:text-orange-300">
            {summary.high} High
          </span>
        )}
        {summary.medium > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-yellow-100 dark:bg-yellow-900 text-yellow-700 dark:text-yellow-300">
            {summary.medium} Medium
          </span>
        )}
        {summary.low > 0 && (
          <span className="px-3 py-1 text-sm font-medium rounded-full bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300">
            {summary.low} Low
          </span>
        )}

        <span className="text-sm text-gray-600 dark:text-gray-400 font-medium ml-2">
          {summary.total} {summary.total === 1 ? 'finding' : 'findings'}
        </span>
      </div>
    </div>
  )
}
