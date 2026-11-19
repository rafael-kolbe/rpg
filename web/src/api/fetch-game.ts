import type { GameSummary } from '../protocol/generated/GameSummary.ts'
import { fetchJson } from '../utils/fetch-json.ts'

/**
 * Calls `GET /api/game`: the logged-in player's games. `null` when not
 * logged in or the server is unavailable.
 */
export async function fetchGameSummary(signal?: AbortSignal): Promise<GameSummary | null> {
  const response = await fetchJson<GameSummary>('/api/game', { signal })
  return response?.status === 200 ? response.body : null
}
