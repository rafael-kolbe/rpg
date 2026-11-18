import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle'

/** Languages the player can pick in the settings. The first one is the default. */
export const LOCALES = ['en-US', 'pt-BR'] as const

export type Locale = (typeof LOCALES)[number]

export const DEFAULT_LOCALE: Locale = 'en-US'

export type TranslationArgs = Record<string, FluentVariable>

export function isLocale(value: string): value is Locale {
  return (LOCALES as readonly string[]).includes(value)
}

// Every .ftl file under the repository's locales/<locale>/ directory, bundled
// as raw text. `make check` guarantees every locale defines the same messages.
const sources = import.meta.glob<string>('../../../locales/*/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
})

function buildBundle(locale: Locale): FluentBundle {
  const bundle = new FluentBundle(locale)
  for (const [path, source] of Object.entries(sources)) {
    if (!path.includes(`/locales/${locale}/`)) continue
    const errors = bundle.addResource(new FluentResource(source))
    for (const error of errors) console.error(`[i18n] ${path}: ${error.message}`)
  }
  return bundle
}

const bundles: Record<Locale, FluentBundle> = {
  'en-US': buildBundle('en-US'),
  'pt-BR': buildBundle('pt-BR'),
}

/**
 * Formats message `id` in `locale`. A missing message renders as its id, so it
 * is visible on screen instead of silently blank.
 */
export function translate(locale: Locale, id: string, args?: TranslationArgs): string {
  const bundle = bundles[locale]
  const message = bundle.getMessage(id)
  if (message?.value == null) {
    console.error(`[i18n] missing message "${id}" for ${locale}`)
    return id
  }
  const errors: Error[] = []
  const text = bundle.formatPattern(message.value, args ?? null, errors)
  for (const error of errors) console.error(`[i18n] "${id}" (${locale}): ${error.message}`)
  return text
}
