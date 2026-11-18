/**
 * `index` wrapped into `0..length`, so stepping past either end of a list
 * comes back from the other end. Negative indices count from the end. A
 * `length` of 0 or less gives 0.
 *
 * @example
 * wrapIndex(3, 3) // → 0
 * wrapIndex(-1, 3) // → 2
 * wrapIndex(7, 0) // → 0
 */
export function wrapIndex(index: number, length: number): number {
  if (length <= 0) return 0
  return ((index % length) + length) % length
}
