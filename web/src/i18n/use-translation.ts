import { useContext } from 'react'

import { I18nContext, type I18nContextValue } from './i18n-context.ts'

/** Current language, a setter, and `t(id, args)` to format a message. */
export function useTranslation(): I18nContextValue {
  const context = useContext(I18nContext)
  if (context === null) throw new Error('useTranslation must be used inside <I18nProvider>')
  return context
}
