import type { SandboxList } from '../protocol/generated/SandboxList.ts'

/**
 * Calls `GET /api/sandbox`. Builds without the sandbox answer 404, which is
 * the same as having no scenarios.
 */
export async function fetchSandboxScenarios(signal?: AbortSignal): Promise<string[]> {
  try {
    const response = await fetch('/api/sandbox', signal ? { signal } : {})
    if (!response.ok) return []
    return ((await response.json()) as SandboxList).scenarios
  } catch (error) {
    if (signal?.aborted) throw error
    return []
  }
}
