export default function ResultsLoading() {
  return (
    <div className="min-h-screen bg-surface-secondary py-8 px-4">
      <div className="max-w-4xl mx-auto">
        {/* Header skeleton */}
        <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6 animate-pulse">
          <div className="h-8 bg-skeleton rounded w-3/4 mb-4"></div>
          <div className="space-y-2">
            <div className="h-4 bg-skeleton rounded w-full"></div>
            <div className="h-4 bg-skeleton rounded w-2/3"></div>
            <div className="h-4 bg-skeleton rounded w-1/2"></div>
          </div>
        </div>

        {/* Grade summary skeleton */}
        <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6 animate-pulse">
          <div className="h-16 bg-skeleton rounded"></div>
        </div>

        {/* Findings section skeleton */}
        <div className="bg-surface-elevated rounded-(card) shadow-md p-6 mb-6 animate-pulse">
          <div className="h-6 bg-skeleton rounded w-1/3 mb-4"></div>
          <div className="space-y-3">
            <div className="h-24 bg-skeleton rounded"></div>
            <div className="h-24 bg-skeleton rounded"></div>
            <div className="h-24 bg-skeleton rounded"></div>
          </div>
        </div>

        {/* Actions skeleton */}
        <div className="flex gap-4 flex-wrap animate-pulse">
          <div className="h-10 w-64 bg-skeleton rounded"></div>
          <div className="h-10 w-48 bg-skeleton rounded"></div>
        </div>
      </div>
    </div>
  )
}
