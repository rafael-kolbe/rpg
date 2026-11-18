import { createContext } from 'react'

import type { Locale, TranslationArgs } from './locales.ts'

export interface I18nContextValue {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (id: string, args?: TranslationArgs) => string
}

export const I18nContext = createContext<I18nContextValue | null>(null)
