import { useEffect, useState } from 'react'

import { fetchHealth } from '../api/fetch-health.ts'
import { fetchSandboxScenarios } from '../api/fetch-sandbox.ts'
import { LanguageSelect } from '../components/LanguageSelect.tsx'
import { useTranslation } from '../i18n/use-translation.ts'

type ServerState = 'checking' | 'ok' | 'unavailable'

/** `/`: server status and, on sandbox builds, the list of test scenarios. */
export function HomePage() {
  const { t } = useTranslation()
  const [server, setServer] = useState<ServerState>('checking')
  const [scenarios, setScenarios] = useState<string[]>([])

  useEffect(() => {
    const controller = new AbortController()
    fetchHealth(controller.signal)
      .then((health) => setServer(health?.status === 'ok' ? 'ok' : 'unavailable'))
      .catch(() => {
        // Aborted on unmount: nothing to update.
      })
    fetchSandboxScenarios(controller.signal)
      .then(setScenarios)
      .catch(() => {
        // Aborted on unmount: nothing to update.
      })
    return () => controller.abort()
  }, [])

  return (
    <main className="app">
      <h1>{t('ui-game-title')}</h1>
      <p className={`server-status server-status--${server}`}>
        {t('ui-server-status', { status: server })}
      </p>
      {scenarios.length > 0 && (
        <section>
          <h2>{t('ui-sandbox-scenarios')}</h2>
          <ul>
            {scenarios.map((scenario) => (
              <li key={scenario}>
                <a href={`/sandbox/${scenario}`}>{scenario}</a>
              </li>
            ))}
          </ul>
        </section>
      )}
      <LanguageSelect />
    </main>
  )
}
