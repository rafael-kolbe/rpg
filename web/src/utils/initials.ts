/**
 * Uppercase initials of the first words of `name`, up to `count` letters.
 * Splits on any whitespace and ignores empty pieces.
 *
 * @example
 * initialsOf('Forest Wolf') // → 'FW'
 * initialsOf('Lobo da Floresta', 3) // → 'LDF'
 * initialsOf('  slime ') // → 'S'
 */
export function initialsOf(name: string, count = 2): string {
  return name
    .split(/\s+/)
    .filter((word) => word.length > 0)
    .slice(0, count)
    .map((word) => word.charAt(0).toUpperCase())
    .join('')
}
