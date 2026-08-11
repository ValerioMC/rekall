import { ofetch, FetchError } from 'ofetch'
import { env } from '@/common/config/env'

/**
 * A failure the user can act on.
 *
 * The server writes its messages to be read, so they are carried through verbatim instead of
 * being replaced with a generic string. `detail` on a ProblemDetail body is the message.
 */
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

/**
 * Retries transient failures only.
 *
 * A 4xx means the request itself is wrong, so repeating it just repeats the error. A 5xx or a
 * timeout may be the server being briefly unavailable, which is worth one or two more attempts.
 */
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

/** Wraps a call so every caller sees an {@link ApiError} rather than a raw fetch failure. */
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
