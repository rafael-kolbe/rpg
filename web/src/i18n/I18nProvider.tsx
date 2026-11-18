import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react'

import { I18nContext, type I18nContextValue } from './i18n-context.ts'
import { DEFAULT_LOCALE, isLocale, translate, type Locale } from './locales.ts'

const STORAGE_KEY = 'rpg.locale'

// The chosen language is a per-browser preference, not game state, so it lives
// in localStorage. Storage can be unavailable (private mode, blocked), hence
// the try/catch: the game must still work, just in the default language.
function readStoredLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    return stored !== null && isLocale(stored) ? stored : DEFAULT_LOCALE
  } catch {
    return DEFAULT_LOCALE
  }
}

function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale)
  } catch {
    // Preference not persisted; the current session still uses it.
  }
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(readStoredLocale)

  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  const setLocale = useCallback((next: Locale) => {
    storeLocale(next)
    setLocaleState(next)
  }, [])

  const value = useMemo<I18nContextValue>(
    () => ({ locale, setLocale, t: (id, args) => translate(locale, id, args) }),
    [locale, setLocale],
  )

  return <I18nContext value={value}>{children}</I18nContext>
}
