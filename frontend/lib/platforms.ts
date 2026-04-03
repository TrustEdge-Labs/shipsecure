export interface PlatformConfig {
  name: string
  slug: string
  accent: string
  placeholder: string
  heroTitle: string
  cveContext: string
  cveLink?: string
  metaTitle: string
  metaDescription: string
}

export const platforms: Record<string, PlatformConfig> = {
  lovable: {
    name: 'Lovable',
    slug: 'lovable',
    accent: '#e11d48',
    placeholder: 'https://your-app.lovable.app',
    heroTitle: 'Is your Lovable app secure?',
    cveContext:
      'CVE-2025-48757 exposed 170+ Lovable apps with RLS misconfigurations leaking PII and API keys.',
    cveLink: 'https://nvd.nist.gov/vuln/detail/CVE-2025-48757',
    metaTitle: 'Is your Lovable app secure? | ShipSecure',
    metaDescription:
      'CVE-2025-48757 exposed 170+ Lovable apps. Scan yours free in 30 seconds.',
  },
  bolt: {
    name: 'Bolt',
    slug: 'bolt',
    accent: '#3b82f6',
    placeholder: 'https://your-project.bolt.new',
    heroTitle: 'Is your Bolt app secure?',
    cveContext:
      '45% of AI-generated code contains security vulnerabilities. Bolt apps ship fast but skip security defaults.',
    metaTitle: 'Is your Bolt app secure? | ShipSecure',
    metaDescription:
      '45% of AI-generated code has vulnerabilities. Scan your Bolt app free in 30 seconds.',
  },
  v0: {
    name: 'v0',
    slug: 'v0',
    accent: '#fafafa',
    placeholder: 'https://your-app.vercel.app',
    heroTitle: 'Is your v0 app secure?',
    cveContext:
      'Vercel/v0 apps inherit Next.js defaults but often miss security headers, exposed .env files, and CSP configuration.',
    metaTitle: 'Is your v0 app secure? | ShipSecure',
    metaDescription:
      'v0 apps often miss security headers and CSP config. Scan yours free in 30 seconds.',
  },
}
