import type { AnnouncedIntent } from '../protocol/generated/AnnouncedIntent.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'

/** A target line: an enemy about to act on someone else. */
export interface TargetLine {
  from: CombatantId
  to: CombatantId
}

/**
 * One line from each charging enemy to each combatant its announced action
 * will hit; a whole-side action draws one line per standing combatant there.
 * Self-targeted actions draw none (the enemy's label says it).
 */
export function targetLines(combatants: CombatantState[]): TargetLine[] {
  return combatants.flatMap(({ id, side, intent }) => {
    if (side !== 'enemies' || intent === null) return []
    const { target } = intent
    switch (target.type) {
      case 'one':
        return [{ from: id, to: target.target }]
      case 'side':
        return combatants
          .filter((c) => c.side === target.side && c.hp > 0 && !c.fled)
          .map((c) => ({ from: id, to: c.id }))
      case 'user':
        return []
    }
  })
}

/** Fluent message id naming the announced command. */
export function intentMessage({ command }: AnnouncedIntent): string {
  switch (command.type) {
    case 'action':
      return `action-${command.id}`
    case 'item':
      return `item-${command.id}`
    case 'defend':
      return 'ui-intent-defend'
  }
}
