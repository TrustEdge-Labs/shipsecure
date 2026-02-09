import { MetadataRoute } from 'next'

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: ['/results/', '/scan/', '/api/', '/payment/'],
    },
    sitemap: 'https://shipsecure.ai/sitemap.xml',
  }
}
