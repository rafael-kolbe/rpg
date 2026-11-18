import type { AccountError } from '../protocol/generated/AccountError.ts'
import type { AccountErrorResponse } from '../protocol/generated/AccountErrorResponse.ts'
import type { Credentials } from '../protocol/generated/Credentials.ts'
import type { RecoverRequest } from '../protocol/generated/RecoverRequest.ts'
import type { RecoveryKeyResponse } from '../protocol/generated/RecoveryKeyResponse.ts'
import type { SessionResponse } from '../protocol/generated/SessionResponse.ts'
import { fetchJson } from '../utils/fetch-json.ts'

/** Outcome of an account call: its answer, or why it failed. */
export type AccountResult<T> = { ok: true; value: T } | { ok: false; error: AccountError }

/** Successful status codes carry `T`; anything else carries an error code. */
async function call<T>(
  url: string,
  method: 'GET' | 'POST' | 'DELETE',
  body?: unknown,
): Promise<AccountResult<T>> {
  const response = await fetchJson<T | AccountErrorResponse>(url, { method, body })
  if (response === null) return { ok: false, error: 'unavailable' }
  if (response.status < 300) return { ok: true, value: response.body as T }
  const error = (response.body as AccountErrorResponse | null)?.error
  return { ok: false, error: error ?? 'unavailable' }
}

/** `POST /api/accounts`: creates the account, logs in, answers the recovery key. */
export function signUp(credentials: Credentials) {
  return call<RecoveryKeyResponse>('/api/accounts', 'POST', credentials)
}

/** `POST /api/session`. */
export function logIn(credentials: Credentials) {
  return call<null>('/api/session', 'POST', credentials)
}

/** `DELETE /api/session`. */
export function logOut() {
  return call<null>('/api/session', 'DELETE')
}

/** `GET /api/session`: who is logged in. */
export function fetchSession() {
  return call<SessionResponse>('/api/session', 'GET')
}

/** `POST /api/accounts/recover`: new password; answers the new recovery key. */
export function recoverAccount(request: RecoverRequest) {
  return call<RecoveryKeyResponse>('/api/accounts/recover', 'POST', request)
}
