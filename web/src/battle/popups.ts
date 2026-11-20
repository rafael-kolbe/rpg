import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { Element } from '../protocol/generated/Element.ts'

/** Something drawn over a combatant for a moment. */
export interface Popup {
  key: string
  target: CombatantId
  kind:
    | 'damage'
    | 'critical'
    | 'poison'
    | 'heal'
    | 'mp'
    | 'miss'
    | 'resist'
    | 'immune'
    | 'cover'
    | 'stolen'
    | 'fled'
    | 'recovered'
  amount?: number
  /** Element of the damage: picks the number's color. */
  element?: Element
}

/** Animation played over each target when an action starts. */
export interface Strike {
  key: string
  target: CombatantId
  kind: 'slash' | 'spell' | 'item'
}

/**
 * Turns one update's events into popups and strike animations. Damage takes
 * the element of the action that caused it (the last one started before it).
 */
export function effectsOf(
  batchId: number,
  events: BattleEvent[],
  catalog: Catalog,
): { popups: Popup[]; strikes: Strike[] } {
  const popups: Popup[] = []
  const strikes: Strike[] = []
  let element: Element | null = null
  events.forEach((event, index) => {
    const key = `${batchId}-${index}`
    switch (event.type) {
      case 'action_started': {
        const info = catalog.actions[event.action]
        const category = info?.category
        element = info?.element ?? null
        const kind =
          category === 'magic' ? 'spell' : category === 'item' ? 'item' : ('slash' as const)
        for (const target of event.targets) strikes.push({ key: `${key}-${target}`, target, kind })
        break
      }
      case 'damaged':
        popups.push({
          key,
          target: event.target,
          kind: event.critical ? 'critical' : 'damage',
          amount: event.amount,
          ...(element !== null && { element }),
        })
        break
      case 'poison_damaged':
        popups.push({ key, target: event.target, kind: 'poison', amount: event.amount })
        break
      case 'healed':
      case 'regenerated':
        popups.push({ key, target: event.target, kind: 'heal', amount: event.amount })
        break
      case 'revived':
        popups.push({ key, target: event.target, kind: 'heal', amount: event.hp })
        break
      case 'mp_restored':
        popups.push({ key, target: event.target, kind: 'mp', amount: event.amount })
        break
      case 'missed':
        popups.push({ key, target: event.target, kind: 'miss' })
        break
      case 'covered':
        popups.push({ key, target: event.protector, kind: 'cover' })
        break
      case 'stole':
        popups.push({ key, target: event.target, kind: 'stolen' })
        break
      case 'fled':
        popups.push({ key, target: event.combatant, kind: 'fled' })
        break
      case 'recovered':
        popups.push({ key, target: event.combatant, kind: 'recovered' })
        break
      case 'status_resisted':
        popups.push({ key, target: event.target, kind: 'resist' })
        break
      case 'status_immune':
        popups.push({ key, target: event.target, kind: 'immune' })
        break
      default:
        break
    }
  })
  return { popups, strikes }
}
