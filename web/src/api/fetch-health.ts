import type { HealthResponse } from '../protocol/generated/HealthResponse.ts'

/**
 * Calls `GET /api/health`. The server answers 503 with a valid body when a
 * dependency is down, so the body is read regardless of the status code.
 * Returns `null` when the server cannot be reached at all.
 */
export async function fetchHealth(signal?: AbortSignal): Promise<HealthResponse | null> {
  try {
    const response = await fetch('/api/health', signal ? { signal } : {})
    return (await response.json()) as HealthResponse
  } catch (error) {
    if (signal?.aborted) throw error
    return null
  }
}
