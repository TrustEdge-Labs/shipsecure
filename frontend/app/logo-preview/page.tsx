import { Logo } from '@/components/logo'

export default function LogoPreviewPage() {
  return (
    <div className="min-h-screen bg-surface-primary p-8">
      <div className="max-w-6xl mx-auto space-y-12">
        <div>
          <h1 className="text-3xl font-semibold text-text-primary mb-2">Logo Preview</h1>
          <p className="text-text-secondary">Visual verification of all three ShipSecure logo variants at multiple sizes in both light and dark modes.</p>
        </div>

        {/* Section 1: Small variant (lettermark) */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Small Variant (Lettermark S)</h2>
            <p className="text-sm text-text-secondary">For favicon and 16px contexts</p>
          </div>

          <div className="grid grid-cols-2 gap-6">
            {/* Light background */}
            <div className="bg-white text-gray-900 p-6 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-4 h-4" />
                  <span className="text-sm text-gray-600">16px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-6 h-6" />
                  <span className="text-sm text-gray-600">24px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-8 h-8" />
                  <span className="text-sm text-gray-600">32px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-12 h-12" />
                  <span className="text-sm text-gray-600">48px</span>
                </div>
              </div>
            </div>

            {/* Dark background */}
            <div className="bg-gray-950 text-gray-100 p-6 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-4 h-4" />
                  <span className="text-sm text-gray-400">16px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-6 h-6" />
                  <span className="text-sm text-gray-400">24px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-8 h-8" />
                  <span className="text-sm text-gray-400">32px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="small" className="w-12 h-12" />
                  <span className="text-sm text-gray-400">48px</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Section 2: Medium variant (shield mark) */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Medium Variant (Shield Mark)</h2>
            <p className="text-sm text-text-secondary">For mobile header and 32-48px contexts</p>
          </div>

          <div className="grid grid-cols-2 gap-6">
            {/* Light background */}
            <div className="bg-white text-gray-900 p-6 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-8 h-9" />
                  <span className="text-sm text-gray-600">32px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-12 h-14" />
                  <span className="text-sm text-gray-600">48px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-16 h-18" />
                  <span className="text-sm text-gray-600">64px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-24 h-27" />
                  <span className="text-sm text-gray-600">96px</span>
                </div>
              </div>
            </div>

            {/* Dark background */}
            <div className="bg-gray-950 text-gray-100 p-6 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-8 h-9" />
                  <span className="text-sm text-gray-400">32px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-12 h-14" />
                  <span className="text-sm text-gray-400">48px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-16 h-18" />
                  <span className="text-sm text-gray-400">64px</span>
                </div>
                <div className="flex items-center gap-4">
                  <Logo size="medium" className="w-24 h-27" />
                  <span className="text-sm text-gray-400">96px</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Section 3: Large variant (shield + wordmark) */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Large Variant (Shield + Wordmark)</h2>
            <p className="text-sm text-text-secondary">For desktop header and full-size contexts</p>
          </div>

          <div className="grid grid-cols-2 gap-6">
            {/* Light background */}
            <div className="bg-white text-gray-900 p-6 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <div className="space-y-6">
                <div>
                  <Logo size="large" className="w-[150px] h-[27px] mb-2" />
                  <span className="text-sm text-gray-600">150px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[200px] h-[36px] mb-2" />
                  <span className="text-sm text-gray-600">200px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[300px] h-[54px] mb-2" />
                  <span className="text-sm text-gray-600">300px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[400px] h-[72px] mb-2" />
                  <span className="text-sm text-gray-600">400px width</span>
                </div>
              </div>
            </div>

            {/* Dark background */}
            <div className="bg-gray-950 text-gray-100 p-6 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <div className="space-y-6">
                <div>
                  <Logo size="large" className="w-[150px] h-[27px] mb-2" />
                  <span className="text-sm text-gray-400">150px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[200px] h-[36px] mb-2" />
                  <span className="text-sm text-gray-400">200px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[300px] h-[54px] mb-2" />
                  <span className="text-sm text-gray-400">300px width</span>
                </div>
                <div>
                  <Logo size="large" className="w-[400px] h-[72px] mb-2" />
                  <span className="text-sm text-gray-400">400px width</span>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
