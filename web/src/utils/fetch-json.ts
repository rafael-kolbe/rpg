/** Status and parsed body of a response. `body` is `null` when it is not JSON. */
interface JsonResponse<T> {
  status: number
  body: T | null
}

/**
 * GETs `url` and parses the body as JSON, whatever the status code (some APIs
 * send a useful JSON body with errors, e.g. 503 from a health check).
 * Resolves `null` when the server cannot be reached. Rethrows when `signal`
 * aborted the request, so callers can ignore aborted results.
 *
 * @example
 * const response = await fetchJson<HealthResponse>('/api/health', controller.signal)
 * if (response?.body) showStatus(response.body.status)
 */
export async function fetchJson<T>(
  url: string,
  signal?: AbortSignal,
): Promise<JsonResponse<T> | null> {
  let response: Response
  try {
    response = await fetch(url, signal ? { signal } : {})
  } catch (error) {
    if (signal?.aborted) throw error
    return null
  }
  try {
    return { status: response.status, body: (await response.json()) as T }
  } catch {
    return { status: response.status, body: null }
  }
}
