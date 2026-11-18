import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { Battle, BattleSocket } from '../battle/use-battle.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { CombatantTools } from './CombatantTools.tsx'
import { EventLog } from './EventLog.tsx'
import { RestartControls } from './RestartControls.tsx'
import { TimeControls } from './TimeControls.tsx'

interface Props {
  battle: Battle
  auto: boolean
  send: (message: ClientMessage) => void
  subscribe: BattleSocket['subscribe']
  onRestart: (seed: number | null, auto: boolean) => void
}

/** Sandbox tools beside the battle: time, restarts, state editing, event log. */
export function DebugPanel({ battle, auto, send, subscribe, onRestart }: Props) {
  const { t } = useTranslation()
  const { state } = battle
  const name = (id: CombatantId) => {
    const combatant = state.combatants.find((c) => c.id === id)
    return combatant ? `${t(combatant.name)} #${id}` : `#${id}`
  }

  return (
    <aside className="debug-panel" data-testid="debug-panel">
      <h2>{t('ui-debug-title')}</h2>
      <p>
        {t('ui-debug-tick', { tick: state.tick })} · {t('ui-debug-seed')} {battle.seed}
      </p>
      <TimeControls state={state} send={send} />
      <RestartControls seed={battle.seed} auto={auto} onRestart={onRestart} />
      <CombatantTools combatants={state.combatants} name={name} send={send} />
      <EventLog subscribe={subscribe} name={name} />
    </aside>
  )
}
