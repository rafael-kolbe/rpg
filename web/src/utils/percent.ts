/**
 * `value` as a percentage of `max`, clamped to 0–100. A `max` of 0 or less
 * gives 0 (nothing to fill), so callers need no division guard.
 *
 * @example
 * percentOf(30, 120) // → 25
 * percentOf(150, 100) // → 100
 * percentOf(5, 0) // → 0
 */
export function percentOf(value: number, max: number): number {
  if (max <= 0) return 0
  return Math.min(100, Math.max(0, (value * 100) / max))
}
