import type { SoundName } from '../audio/sounds.ts'
import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'

/**
 * The sounds of one update's events, each at most once so a hit on three
 * targets does not play three times louder. Damage sounds follow the action
 * that caused them: elemental spells by element, everything else a blow.
 */
export function soundsOf(events: BattleEvent[], catalog: Catalog): SoundName[] {
  const sounds = new Set<SoundName>()
  let element: SoundName | null = null
  // The AI queues in the same update its gauge fills; a turn_ready without
  // that is a hero waiting for the player.
  const decided = new Set(
    events.flatMap((e) =>
      e.type === 'action_queued' ? [e.actor] : e.type === 'defending' ? [e.combatant] : [],
    ),
  )

  for (const event of events) {
    switch (event.type) {
      case 'action_started': {
        const info = catalog.actions[event.action]
        element = info?.element ?? null
        if (info?.category === 'item') sounds.add('item')
        break
      }
      case 'damaged':
        sounds.add(element ?? (event.critical ? 'critical' : 'hit'))
        break
      case 'poison_damaged':
        sounds.add('poison')
        break
      case 'missed':
      case 'status_resisted':
      case 'status_immune':
        sounds.add('miss')
        break
      case 'healed':
      case 'mp_restored':
      case 'regenerated':
        sounds.add('heal')
        break
      case 'revived':
        sounds.add('revive')
        break
      case 'status_applied':
        sounds.add('status')
        break
      case 'status_removed':
        if (event.reason === 'cured') sounds.add('cure')
        break
      case 'defending':
        sounds.add('defend')
        break
      case 'knocked_out':
        sounds.add('knockout')
        break
      case 'turn_ready':
        if (!decided.has(event.combatant)) sounds.add('turn')
        break
      case 'battle_ended':
        sounds.add(event.outcome === 'victory' ? 'victory' : 'defeat')
        break
      case 'action_queued':
      case 'action_cancelled':
      case 'action_failed':
        break
    }
  }
  return [...sounds]
}
