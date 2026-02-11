import { Logo } from '@/components/logo'

export default function LogoPreviewPage() {
  return (
    <div className="min-h-screen bg-surface-primary p-8">
      <div className="max-w-6xl mx-auto space-y-12">
        <div>
          <h1 className="text-3xl font-semibold text-text-primary mb-2">Logo Preview</h1>
          <p className="text-text-secondary">ShipSecure logo at multiple sizes on light and dark backgrounds.</p>
        </div>

        {/* Small sizes */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Small</h2>
            <p className="text-sm text-text-secondary">For navbar and compact contexts</p>
          </div>

          <div className="grid grid-cols-2 gap-6">
            <div className="bg-white text-gray-900 p-6 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="small" />
                  <span className="text-sm text-gray-500">Default small (96x64)</span>
                </div>
              </div>
            </div>

            <div className="bg-gray-950 text-gray-100 p-6 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <div className="space-y-4">
                <div className="flex items-center gap-4">
                  <Logo size="small" />
                  <span className="text-sm text-gray-400">Default small (96x64)</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Medium sizes */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Medium</h2>
            <p className="text-sm text-text-secondary">For headers and navigation</p>
          </div>

          <div className="grid grid-cols-2 gap-6">
            <div className="bg-white text-gray-900 p-6 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <div className="space-y-4">
                <div>
                  <Logo size="medium" />
                  <span className="text-xs text-gray-500 mt-1 block">Default medium (384x256)</span>
                </div>
              </div>
            </div>

            <div className="bg-gray-950 text-gray-100 p-6 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <div className="space-y-4">
                <div>
                  <Logo size="medium" />
                  <span className="text-xs text-gray-400 mt-1 block">Default medium (384x256)</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Large sizes */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold text-text-primary mb-1">Large</h2>
            <p className="text-sm text-text-secondary">For hero sections and full display</p>
          </div>

          <div className="space-y-6">
            <div className="bg-white text-gray-900 p-8 rounded-lg border border-border-primary">
              <h3 className="text-sm font-medium mb-4">Light Background</h3>
              <Logo size="large" />
              <span className="text-xs text-gray-500 mt-2 block">Default large (768x512)</span>
            </div>

            <div className="bg-gray-950 text-gray-100 p-8 rounded-lg">
              <h3 className="text-sm font-medium mb-4">Dark Background</h3>
              <Logo size="large" />
              <span className="text-xs text-gray-400 mt-2 block">Default large (768x512)</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
