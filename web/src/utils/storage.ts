/**
 * Reads a `localStorage` key, returning `null` when the key is missing or
 * storage is unavailable (private mode, blocked site data). Never throws.
 *
 * @example
 * const locale = readStorage('rpg.locale') ?? 'en-US'
 */
export function readStorage(key: string): string | null {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

/**
 * Writes a `localStorage` key, silently doing nothing when storage is
 * unavailable. For per-browser preferences only: the write may not persist.
 *
 * @example
 * writeStorage('rpg.locale', 'pt-BR')
 */
export function writeStorage(key: string, value: string): void {
  try {
    localStorage.setItem(key, value)
  } catch {
    // Not persisted; the current session still has the value in memory.
  }
}
