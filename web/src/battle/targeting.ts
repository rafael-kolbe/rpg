import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'

/**
 * Whether `target` can receive `action` from `actor`. Only a UI hint for
 * highlighting targets: the server validates every command again.
 */
export function canTarget(
  action: ActionInfo,
  actor: CombatantState,
  target: CombatantState,
): boolean {
  const ownSide = action.target === 'single_ally' || action.target === 'all_allies'
  const side = ownSide ? actor.side : actor.side === 'party' ? 'enemies' : 'party'
  const knockedOut = target.hp === 0
  return target.side === side && !target.fled && knockedOut === action.targetsKnockedOut
}

/** Whether the command must name a target before it is sent. */
export function needsTarget(action: ActionInfo): boolean {
  return action.target === 'single_enemy' || action.target === 'single_ally'
}
