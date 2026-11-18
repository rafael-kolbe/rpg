/** Status and parsed body of a response. `body` is `null` when it is not JSON. */
interface JsonResponse<T> {
  status: number
  body: T | null
}

/** Request options of {@link fetchJson}. */
interface JsonRequest {
  /** Aborts the request. */
  signal?: AbortSignal
  /** HTTP method; `GET` by default. */
  method?: 'GET' | 'POST' | 'DELETE'
  /** Sent as JSON. */
  body?: unknown
}

/**
 * Sends a request to `url` (a GET unless `method` says otherwise, with `body`
 * as JSON) and parses the answer as JSON, whatever the status code (some APIs
 * send a useful JSON body with errors, e.g. 503 from a health check).
 * Resolves `null` when the server cannot be reached. Rethrows when `signal`
 * aborted the request, so callers can ignore aborted results.
 *
 * @example
 * const response = await fetchJson<HealthResponse>('/api/health', { signal })
 * if (response?.body) showStatus(response.body.status)
 * await fetchJson('/api/session', { method: 'POST', body: { username, password } })
 */
export async function fetchJson<T>(
  url: string,
  { signal, method = 'GET', body }: JsonRequest = {},
): Promise<JsonResponse<T> | null> {
  const init: RequestInit = { method }
  if (signal) init.signal = signal
  if (body !== undefined) {
    init.body = JSON.stringify(body)
    init.headers = { 'content-type': 'application/json' }
  }
  let response: Response
  try {
    response = await fetch(url, init)
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
