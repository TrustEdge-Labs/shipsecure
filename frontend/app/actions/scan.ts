'use server'

import { z } from 'zod'

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

  try {
    const response = await fetch(`${backendUrl}/api/v1/scans`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(validatedFields.data),
    })

    if (!response.ok) {
      const errorBody = await response.json().catch(() => ({}))
      const detail = errorBody.detail || 'Failed to start scan. Please try again.'

      if (response.status === 429) {
        return {
          errors: { _form: ['You have reached the maximum number of scans for today. Please try again tomorrow.'] }
        }
      }

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
