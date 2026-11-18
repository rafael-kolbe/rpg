import { useEffect, useState } from 'react'

import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { BattleSocket } from '../battle/use-battle.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { describeEvent } from './event-log.ts'

const LOG_LINES = 80

interface Props {
  subscribe: BattleSocket['subscribe']
  name: (id: CombatantId) => string
}

/**
 * Latest events, newest first. Raw events are stored and formatted at render,
 * so the log follows a language change.
 */
export function EventLog({ subscribe, name }: Props) {
  const { t } = useTranslation()
  const [log, setLog] = useState<{ key: string; tick: number; event: BattleEvent }[]>([])

  useEffect(
    () =>
      subscribe((batch) => {
        const lines = batch.events
          .map((event, index) => ({ key: `${batch.id}-${index}`, tick: batch.tick, event }))
          .toReversed()
        setLog((existing) => [...lines, ...existing].slice(0, LOG_LINES))
      }),
    [subscribe],
  )

  return (
    <>
      <h3>{t('ui-debug-log')}</h3>
      <ol className="debug-panel__log" data-testid="event-log">
        {log.map((line) => {
          const text = describeEvent(line.event, name, t)
          return (
            text !== null && (
              <li key={line.key}>
                <span>{(line.tick / 10).toFixed(1)}s</span> {text}
              </li>
            )
          )
        })}
      </ol>
    </>
  )
}
