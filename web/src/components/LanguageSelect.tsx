import { LOCALES } from '../i18n/locales.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Select } from '../ui/Select.tsx'

/** The language picker (en-US / pt-BR). */
export function LanguageSelect() {
  const { t, locale, setLocale } = useTranslation()
  return (
    <Select
      label={t('ui-language')}
      value={locale}
      options={LOCALES.map((option) => ({ value: option, label: t(`ui-language-${option}`) }))}
      onChange={setLocale}
      testId="language"
    />
  )
}
