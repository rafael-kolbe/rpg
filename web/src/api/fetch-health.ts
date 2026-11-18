import type { HealthResponse } from '../protocol/generated/HealthResponse.ts'
import { fetchJson } from '../utils/fetch-json.ts'

/**
 * Calls `GET /api/health`. The server answers 503 with a valid body when a
 * dependency is down, so the body is used regardless of the status code.
 * Returns `null` when the server cannot be reached at all.
 */
export async function fetchHealth(signal?: AbortSignal): Promise<HealthResponse | null> {
  return (await fetchJson<HealthResponse>('/api/health', { signal }))?.body ?? null
}
