/**
 * Joins the truthy class names with spaces, skipping `false`, `null`,
 * `undefined` and empty strings. Lets conditional classes stay inline.
 *
 * @example
 * classNames('combatant', isKnockedOut && 'combatant--knocked-out', null)
 * // → 'combatant combatant--knocked-out' when isKnockedOut, else 'combatant'
 */
export function classNames(...names: (string | false | null | undefined)[]): string {
  return names.filter(Boolean).join(' ')
}
