import { useEffect, useState } from 'react'

import { fetchHealth } from './api/fetch-health.ts'
import { useTranslation } from './i18n/use-translation.ts'
import { LOCALES, isLocale } from './i18n/locales.ts'

type ServerState = 'checking' | 'ok' | 'unavailable'

export function App() {
  const { t, locale, setLocale } = useTranslation()
  const [server, setServer] = useState<ServerState>('checking')

  useEffect(() => {
    const controller = new AbortController()
    fetchHealth(controller.signal)
      .then((health) => setServer(health?.status === 'ok' ? 'ok' : 'unavailable'))
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
      <label className="language">
        {t('ui-language')}
        <select
          value={locale}
          onChange={(event) => {
            if (isLocale(event.target.value)) setLocale(event.target.value)
          }}
        >
          {LOCALES.map((option) => (
            <option key={option} value={option}>
              {t(`ui-language-${option}`)}
            </option>
          ))}
        </select>
      </label>
    </main>
  )
}
