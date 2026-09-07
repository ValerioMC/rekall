import { ofetch, FetchError } from 'ofetch'
import { env } from '@/common/config/env'

export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly correlationId: string
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

const CONNECT_TIMEOUT_MS = 15_000
const MAX_RETRIES = 2

function correlationId(): string {
  return crypto.randomUUID()
}

function shouldRetry(status: number | undefined): boolean {
  return status === undefined || status >= 500
}

export const apiClient = ofetch.create({
  baseURL: env.VITE_API_BASE_URL,
  timeout: CONNECT_TIMEOUT_MS,
  retry: MAX_RETRIES,
  retryDelay: 200,
  retryStatusCodes: [408, 500, 502, 503, 504],
  headers: { 'Content-Type': 'application/json' },
  onRequest({ options }) {
    const headers = new Headers(options.headers)
    headers.set('X-Correlation-Id', correlationId())
    options.headers = headers
  }
})

export async function request<T>(operation: () => Promise<T>): Promise<T> {
  try {
    return await operation()
  } catch (error) {
    if (error instanceof FetchError) {
      const status = error.response?.status ?? 0
      const detail =
        (error.data as { detail?: string } | undefined)?.detail ??
        error.message ??
        'The request failed'
      const id = String(error.request instanceof Request ? error.request.url : '')
      if (!shouldRetry(status)) {
        throw new ApiError(status, detail, id)
      }
      throw new ApiError(status, detail, id)
    }
    throw error
  }
}
