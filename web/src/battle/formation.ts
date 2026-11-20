/**
 * Where a combatant stands on its side of the field, in percent of that
 * side: `depth` from the side's outer edge (0) toward the middle of the field
 * (100), and `height` from the top. Fixed by how many stand on the side, so
 * the same fight always looks the same: some at the back, some in front.
 */
export interface Slot {
  depth: number
  height: number
}

const FORMATIONS: Slot[][] = [
  [{ depth: 50, height: 50 }],
  [
    { depth: 28, height: 30 },
    { depth: 70, height: 70 },
  ],
  [
    { depth: 24, height: 25 },
    { depth: 72, height: 50 },
    { depth: 24, height: 75 },
  ],
  [
    { depth: 22, height: 25 },
    { depth: 70, height: 29 },
    { depth: 22, height: 71 },
    { depth: 70, height: 75 },
  ],
]

/**
 * The slot of the `index`-th of `count` combatants on a side. Past four (a
 * raid), they fill columns of two from the back to the front.
 */
export function slotOf(count: number, index: number): Slot {
  const fixed = FORMATIONS[count - 1]?.[index]
  if (fixed !== undefined) return fixed
  const columns = Math.ceil(count / 2)
  const column = Math.floor(index / 2)
  return {
    depth: 15 + (column * 70) / Math.max(columns - 1, 1),
    height: index % 2 === 0 ? 25 : 75,
  }
}
