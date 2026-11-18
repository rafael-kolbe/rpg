// Fails when the locales disagree: a message (or message attribute) defined in
// one language and missing in another, or an .ftl file with syntax errors.
// Run by `make check`.

import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

import { Junk, Message, Term, parse } from '@fluent/syntax'

const LOCALES_DIR = join(import.meta.dirname, '..', '..', 'locales')

/** Keys defined by one locale: `message-id` and `message-id.attribute`. */
function collectKeys(locale: string, problems: string[]): Set<string> {
  const keys = new Set<string>()
  const dir = join(LOCALES_DIR, locale)
  for (const file of readdirSync(dir).filter((name) => name.endsWith('.ftl'))) {
    const resource = parse(readFileSync(join(dir, file), 'utf8'), {})
    for (const entry of resource.body) {
      if (entry instanceof Junk) {
        problems.push(`${locale}/${file}: syntax error near "${entry.content.trim().slice(0, 40)}"`)
      } else if (entry instanceof Message || entry instanceof Term) {
        const id = entry instanceof Term ? `-${entry.id.name}` : entry.id.name
        if (keys.has(id)) problems.push(`${locale}/${file}: "${id}" defined twice`)
        keys.add(id)
        for (const attribute of entry.attributes) keys.add(`${id}.${attribute.id.name}`)
      }
    }
  }
  return keys
}

const locales = readdirSync(LOCALES_DIR, { withFileTypes: true })
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)

const problems: string[] = []
const keysByLocale = new Map(locales.map((locale) => [locale, collectKeys(locale, problems)]))
const allKeys = new Set([...keysByLocale.values()].flatMap((keys) => [...keys]))

for (const [locale, keys] of keysByLocale) {
  for (const key of allKeys) {
    if (!keys.has(key)) problems.push(`${locale}: missing "${key}"`)
  }
}

if (problems.length > 0) {
  console.error(`Locale check failed (${problems.length}):`)
  for (const problem of problems) console.error(`  - ${problem}`)
  process.exit(1)
}

console.warn(`Locales ok: ${locales.join(', ')} (${allKeys.size} keys each)`)
