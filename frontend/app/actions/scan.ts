'use server'

import { z } from 'zod'
import { auth } from '@clerk/nextjs/server'

const scanSchema = z.object({
  url: z.string()
    .min(1, 'URL is required')
    .url('Please enter a valid URL (e.g., https://example.com)')
    .refine(url => url.startsWith('http://') || url.startsWith('https://'), {
      message: 'URL must start with http:// or https://'
    }),
  email: z.string()
    .min(1, 'Email is required')
    .email('Please enter a valid email address'),
  authorization: z
    .string()
    .transform(val => val === 'on')
    .refine(val => val === true, {
      message: 'You must confirm you have authorization to scan this website'
    })
})

export interface ScanFormState {
  errors?: {
    url?: string[]
    email?: string[]
    authorization?: string[]
    _form?: string[]
  }
  scanId?: string
}

export async function submitScan(
  prevState: ScanFormState,
  formData: FormData
): Promise<ScanFormState> {
  const validatedFields = scanSchema.safeParse({
    url: formData.get('url'),
    email: formData.get('email'),
    authorization: formData.get('authorization') ?? '',
  })

  if (!validatedFields.success) {
    return {
      errors: validatedFields.error.flatten().fieldErrors,
    }
  }

  const backendUrl = process.env.BACKEND_URL || 'http://localhost:3000'

  // Extract Clerk auth — anonymous users get null token
  const { getToken, userId } = await auth()
  const token = userId ? await getToken() : null

  // If authenticated, check domain verification before submitting the scan
  if (userId && token) {
    try {
      const urlObj = new URL(validatedFields.data.url)
      const domain = urlObj.hostname.replace(/^www\./, '').toLowerCase()

      const domainsRes = await fetch(`${backendUrl}/api/v1/domains`, {
        headers: { 'Authorization': `Bearer ${token}` },
      })
      const domains = domainsRes.ok ? await domainsRes.json() : []
      const isVerified = domains.some((d: { domain: string; status: string }) =>
        d.domain === domain && d.status === 'verified'
      )
      if (!isVerified) {
        return {
          errors: { _form: ['You must verify ownership of this domain before scanning. Go to /verify-domain to get started.'] }
        }
      }
    } catch {
      // If domain check fails, let the backend enforce it
    }
  }

  // Build headers with optional auth token
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  try {
    const response = await fetch(`${backendUrl}/api/v1/scans`, {
      method: 'POST',
      headers,
      body: JSON.stringify(validatedFields.data),
    })

    if (!response.ok) {
      const errorBody = await response.json().catch(() => ({}))

      if (response.status === 429) {
        const resetsAt = errorBody.resets_at
        let message = errorBody.detail || 'You have reached your scan limit. Please try again later.'
        if (resetsAt) {
          const diff = new Date(resetsAt).getTime() - Date.now()
          if (diff > 0) {
            const hours = Math.floor(diff / (1000 * 60 * 60))
            const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
            const countdown = hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`
            message = `${message} Resets in ${countdown}.`
          }
        }
        return {
          errors: { _form: [message] }
        }
      }

      const detail = errorBody.detail || 'Failed to start scan. Please try again.'
      return {
        errors: { _form: [detail] }
      }
    }

    const data = await response.json()
    return { scanId: data.id }

  } catch (error) {
    return {
      errors: { _form: ['Unable to connect to the scanning service. Please try again later.'] }
    }
  }
}
