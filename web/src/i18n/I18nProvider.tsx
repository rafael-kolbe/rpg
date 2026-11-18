import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react'

import { I18nContext, type I18nContextValue } from './i18n-context.ts'
import { readStorage, writeStorage } from '../utils/storage.ts'
import { DEFAULT_LOCALE, isLocale, translate, type Locale } from './locales.ts'

const STORAGE_KEY = 'rpg.locale'

// The chosen language is a per-browser preference, not game state, so it lives
// in localStorage (missing or blocked storage falls back to the default).
function readStoredLocale(): Locale {
  const stored = readStorage(STORAGE_KEY)
  return stored !== null && isLocale(stored) ? stored : DEFAULT_LOCALE
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(readStoredLocale)

  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  const setLocale = useCallback((next: Locale) => {
    writeStorage(STORAGE_KEY, next)
    setLocaleState(next)
  }, [])

  const value = useMemo<I18nContextValue>(
    () => ({ locale, setLocale, t: (id, args) => translate(locale, id, args) }),
    [locale, setLocale],
  )

  return <I18nContext value={value}>{children}</I18nContext>
}
