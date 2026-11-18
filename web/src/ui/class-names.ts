/** Joins the truthy class names: `classNames('a', cond && 'b')`. */
export function classNames(...names: (string | false | null | undefined)[]): string {
  return names.filter(Boolean).join(' ')
}
