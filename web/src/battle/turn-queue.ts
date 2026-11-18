import type { CombatantId } from '../protocol/generated/CombatantId.ts'

/**
 * Who the command menu serves: the hero the player picked (Tab or click) if
 * still ready, otherwise the ready hero whose gauge filled first. A turn that
 * comes later waits in line instead of taking the menu over.
 *
 * `order` lists ids in the order their gauges filled (most recent last); a
 * ready id missing from it (a battle that opens with full gauges) comes after
 * the listed ones, in party order.
 */
export function menuActor(
  ready: readonly CombatantId[],
  picked: CombatantId | null,
  order: readonly CombatantId[],
): CombatantId | null {
  if (picked !== null && ready.includes(picked)) return picked
  return order.find((id) => ready.includes(id)) ?? ready[0] ?? null
}

/** `order` with `filled` moved to the back, in the order they filled. */
export function enqueueTurns(
  order: readonly CombatantId[],
  filled: readonly CombatantId[],
): CombatantId[] {
  return [...order.filter((id) => !filled.includes(id)), ...filled]
}
