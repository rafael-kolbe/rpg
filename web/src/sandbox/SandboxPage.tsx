import { useEffect, useMemo, useRef, useState } from 'react'

import { BattleScreen } from '../battle/BattleScreen.tsx'
import { useBattle } from '../battle/use-battle.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import { LanguageSelect } from '../components/LanguageSelect.tsx'
import { useTranslation } from '../i18n/use-translation.ts'
import { DebugPanel } from './DebugPanel.tsx'

interface Props {
  scenario: string
}

/** URL options: `?seed=42&auto=1&speed=400`. */
function readOptions() {
  const params = new URLSearchParams(window.location.search)
  const seed = params.get('seed')
  return {
    seed: seed === null || seed === '' ? null : Number(seed),
    auto: params.get('auto') === '1',
    speed: params.has('speed') ? Number(params.get('speed')) : null,
  }
}

/** `/sandbox/<scenario>`: a test battle with the debug panel beside it. */
export function SandboxPage({ scenario }: Props) {
  const { t } = useTranslation()
  const [options] = useState(readOptions)
  const start = useMemo<ClientMessage>(
    () => ({ type: 'start_sandbox', scenario, seed: options.seed, auto: options.auto }),
    [scenario, options],
  )
  const { connection, battle, rejection, error, send, subscribe } = useBattle(start)

  // Apply the URL speed once, when the first battle arrives.
  const speedApplied = useRef(false)
  useEffect(() => {
    if (battle === null || speedApplied.current || options.speed === null) return
    speedApplied.current = true
    send({ type: 'set_speed', percent: options.speed })
  }, [battle, options.speed, send])

  const restart = (seed: number | null, auto: boolean) =>
    send({ type: 'start_sandbox', scenario, seed, auto })

  return (
    <main className="sandbox">
      <header className="sandbox__header">
        <h1>
          {t('ui-sandbox-title')} · {scenario}
        </h1>
        <LanguageSelect />
      </header>
      {error !== null && <p className="sandbox__error">{t(`ui-error-${error}`)}</p>}
      {connection === 'closed' && <p className="sandbox__error">{t('ui-battle-disconnected')}</p>}
      {battle === null ? (
        connection !== 'closed' && <p>{t('ui-battle-connecting')}</p>
      ) : (
        <div className="sandbox__layout">
          <BattleScreen
            battle={battle}
            rejection={rejection}
            send={send}
            subscribe={subscribe}
            outcomeLabel={t('ui-outcome-restart')}
            onOutcome={() => restart(null, false)}
          />
          <DebugPanel
            battle={battle}
            auto={options.auto}
            send={send}
            subscribe={subscribe}
            onRestart={restart}
          />
        </div>
      )}
    </main>
  )
}
