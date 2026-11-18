import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { TranslationArgs } from '../i18n/locales.ts'

type Translate = (id: string, args?: TranslationArgs) => string

/** One debug-log line per event worth reading; gauge bookkeeping is skipped. */
export function describeEvent(
  event: BattleEvent,
  name: (id: CombatantId) => string,
  t: Translate,
): string | null {
  switch (event.type) {
    case 'action_started':
      return t('ui-log-action', {
        actor: name(event.actor),
        action: t(`action-${event.action}`),
        targets: event.targets.map(name).join(', '),
      })
    case 'damaged': {
      const tags = [
        event.critical ? t('ui-log-tag-critical') : null,
        event.distracted ? t('ui-log-tag-distracted') : null,
        event.affinity ? t(`ui-log-tag-${event.affinity}`) : null,
      ].filter((tag) => tag !== null)
      return t('ui-log-damage', {
        target: name(event.target),
        amount: event.amount,
        tags: tags.length > 0 ? ` (${tags.join(', ')})` : '',
      })
    }
    case 'healed':
      return t('ui-log-heal', { target: name(event.target), amount: event.amount })
    case 'mp_restored':
      return t('ui-log-mp', { target: name(event.target), amount: event.amount })
    case 'revived':
      return t('ui-log-revive', { target: name(event.target), hp: event.hp })
    case 'poison_damaged':
      return t('ui-log-poison', { target: name(event.target), amount: event.amount })
    case 'regenerated':
      return t('ui-log-regen', { target: name(event.target), amount: event.amount })
    case 'missed':
      return t('ui-log-miss', { target: name(event.target) })
    case 'status_applied':
      return t('ui-log-status-gain', {
        target: name(event.target),
        status: t(`ui-status-${event.status}`),
      })
    case 'status_removed':
      return t('ui-log-status-lose', {
        target: name(event.target),
        status: t(`ui-status-${event.status}`),
      })
    case 'status_resisted':
      return t('ui-log-resist', {
        target: name(event.target),
        status: t(`ui-status-${event.status}`),
      })
    case 'status_immune':
      return t('ui-log-immune', {
        target: name(event.target),
        status: t(`ui-status-${event.status}`),
      })
    case 'defending':
      return t('ui-log-defend', { actor: name(event.combatant) })
    case 'knocked_out':
      return t('ui-log-knocked-out', { target: name(event.combatant) })
    case 'action_failed':
      return t('ui-log-failed', { actor: name(event.actor), action: t(`action-${event.action}`) })
    case 'action_cancelled':
      return t('ui-log-cancelled', {
        actor: name(event.actor),
        action: t(`action-${event.action}`),
      })
    case 'battle_ended':
      return t(`ui-outcome-${event.outcome}`)
    case 'turn_ready':
    case 'action_queued':
      return null
  }
}
