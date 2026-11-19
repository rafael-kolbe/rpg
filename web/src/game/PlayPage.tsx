import { useEffect, useState } from 'react'

import { fetchGameSummary } from '../api/fetch-game.ts'
import type { GameMode } from '../protocol/generated/GameMode.ts'
import type { GameSummary } from '../protocol/generated/GameSummary.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { GameScreen } from './GameScreen.tsx'
import { ModeSelect } from './ModeSelect.tsx'

type Loading = { state: 'loading' } | { state: 'ready'; summary: GameSummary }

/**
 * `/play`: after login, the player picks Campaign or Multiplayer, then
 * plays. Without a session it sends the player to the login page.
 */
export function PlayPage() {
  const { t } = useTranslation()
  const [loading, setLoading] = useState<Loading>({ state: 'loading' })
  const [mode, setMode] = useState<GameMode | null>(null)

  useEffect(() => {
    const controller = new AbortController()
    fetchGameSummary(controller.signal)
      .then((summary) => {
        if (summary === null) window.location.assign('/login')
        else setLoading({ state: 'ready', summary })
      })
      .catch(() => {
        // Aborted on unmount: nothing to update.
      })
    return () => controller.abort()
  }, [])

  if (mode !== null) return <GameScreen mode={mode} />
  return (
    <main className="app">
      {loading.state === 'loading' ? (
        <p>{t('ui-battle-connecting')}</p>
      ) : (
        <ModeSelect summary={loading.summary} onChoose={setMode} />
      )}
    </main>
  )
}
