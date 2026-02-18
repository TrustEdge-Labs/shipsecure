import { http, HttpResponse } from 'msw'
import { scanFixtures } from '../fixtures/scan'
import { resultsFixtures } from '../fixtures/results'

const BASE_URL = 'http://localhost:3000'

export const handlers = [
  // POST /api/v1/scans - Create scan (happy path)
  http.post(`${BASE_URL}/api/v1/scans`, () => {
    return HttpResponse.json(scanFixtures.created, { status: 201 })
  }),

  // GET /api/v1/scans/:id - Get scan status (happy path: in_progress)
  http.get(`${BASE_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json(scanFixtures.inProgress)
  }),

  // GET /api/v1/results/:token - Get results by token (happy path)
  http.get(`${BASE_URL}/api/v1/results/:token`, () => {
    return HttpResponse.json(resultsFixtures.gradeA)
  }),

  // GET /api/v1/stats/scan-count - Scan counter (happy path)
  http.get(`${BASE_URL}/api/v1/stats/scan-count`, () => {
    return HttpResponse.json({ count: 1247 })
  }),
]

// Error handler factories for use with server.use() in individual tests
export const errorHandlers = {
  scanNotFound: http.get(`${BASE_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/not-found', title: 'Not Found', status: 404, detail: 'Scan not found' },
      { status: 404 }
    )
  }),

  scanServerError: http.get(`${BASE_URL}/api/v1/scans/:id`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/internal', title: 'Internal Server Error', status: 500, detail: 'An unexpected error occurred' },
      { status: 500 }
    )
  }),

  resultsNotFound: http.get(`${BASE_URL}/api/v1/results/:token`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/not-found', title: 'Not Found', status: 404, detail: 'Results not found or expired' },
      { status: 404 }
    )
  }),

  resultsServerError: http.get(`${BASE_URL}/api/v1/results/:token`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/internal', title: 'Internal Server Error', status: 500, detail: 'An unexpected error occurred' },
      { status: 500 }
    )
  }),

  createScanRateLimited: http.post(`${BASE_URL}/api/v1/scans`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/rate-limit', title: 'Rate Limit Exceeded', status: 429, detail: 'You have reached the limit of 3 scans per day. Please try again tomorrow.' },
      { status: 429 }
    )
  }),

  createScanServerError: http.post(`${BASE_URL}/api/v1/scans`, () => {
    return HttpResponse.json(
      { type: 'https://shipsecure.io/errors/internal', title: 'Internal Server Error', status: 500, detail: 'Failed to create scan' },
      { status: 500 }
    )
  }),
}
