import type { SandboxList } from '../protocol/generated/SandboxList.ts'
import { fetchJson } from '../utils/fetch-json.ts'

/**
 * Calls `GET /api/sandbox`. Builds without the sandbox answer 404, which is
 * the same as having no scenarios.
 */
export async function fetchSandboxScenarios(signal?: AbortSignal): Promise<string[]> {
  const response = await fetchJson<SandboxList>('/api/sandbox', { signal })
  return response?.status === 200 && response.body ? response.body.scenarios : []
}
