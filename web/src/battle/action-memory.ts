import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'
import type { MenuLevel } from './CommandMenu.tsx'
import { usableAction } from './commands.ts'
import { canTarget, needsTarget } from './targeting.ts'

/** The last command a hero was given. */
export interface RememberedCommand {
  kind: CommandKind
  target: CombatantId | null
}

/**
 * The menu entry the cursor starts on at `level` for a remembered command:
 * its command on the root menu (Magic for a spell), then the spell or item
 * itself inside its submenu. `undefined` keeps the default cursor.
 */
export function rememberedEntry(
  remembered: RememberedCommand | null,
  level: MenuLevel,
  catalog: Catalog,
): string | undefined {
  const kind = remembered?.kind
  if (kind === undefined || kind.type === 'defend') return undefined
  if (kind.type === 'attack') return level === 'root' ? 'attack' : undefined
  if (kind.type === 'item')
    return level === 'root' ? 'item' : level === 'item' ? kind.id : undefined
  const category = catalog.actions[kind.id]?.category
  if (level === 'root') return category
  return level === category ? kind.id : undefined
}

/**
 * The remembered command, ready to send again for `actor`: same choice, same
 * target, or the first valid target when that one is gone (knocked out,
 * revived). `null` when the choice cannot be used now.
 */
export function repeatCommand(
  remembered: RememberedCommand,
  actor: CombatantState,
  state: BattleState,
  catalog: Catalog,
): RememberedCommand | null {
  const { kind } = remembered
  if (kind.type === 'defend') return { kind, target: null }
  const action = usableAction(kind, actor, state, catalog)
  if (action === null) return null
  if (!needsTarget(action)) return { kind, target: null }
  const targets = state.combatants.filter((c) => canTarget(action, actor, c))
  const target = targets.find((c) => c.id === remembered.target) ?? targets[0]
  return target ? { kind, target: target.id } : null
}
