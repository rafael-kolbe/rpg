import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'

/**
 * The action `kind` would run for `actor` right now, or `null` when it cannot
 * be used (unknown, not enough MP, silenced, item ran out). Defend runs no
 * action and is always `null`. Only a UI hint: the server checks again.
 */
export function usableAction(
  kind: CommandKind,
  actor: CombatantState,
  state: BattleState,
  catalog: Catalog,
): ActionInfo | null {
  switch (kind.type) {
    case 'attack':
      return catalog.actions[actor.attack] ?? null
    case 'defend':
      return null
    case 'action': {
      const action = catalog.actions[kind.id]
      if (!action || !actor.actions.includes(kind.id) || action.mpCost > actor.mp) return null
      const silenced = actor.statuses.some((s) => s.kind === 'silence')
      return action.category === 'magic' && silenced ? null : action
    }
    case 'item': {
      const count = state.inventory.find((entry) => entry.item === kind.id)?.count ?? 0
      const info = catalog.items[kind.id]
      return count > 0 && info ? (catalog.actions[info.action] ?? null) : null
    }
  }
}

/** Whether two commands are the same choice (same kind, same spell or item). */
export function sameCommand(a: CommandKind, b: CommandKind): boolean {
  return a.type === b.type && ('id' in a ? a.id : null) === ('id' in b ? b.id : null)
}
