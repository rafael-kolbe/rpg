// Every exported function in src/utils must have a JSDoc block with an
// `@example`. Run by `make check`. See CLAUDE.md, "Utils".

import { readFileSync, readdirSync } from 'node:fs'
import { join } from 'node:path'

const UTILS_DIR = join(import.meta.dirname, '..', 'src', 'utils')
const EXPORTED_FUNCTION = /^export (async )?function (\w+)/

const problems: string[] = []
const files = readdirSync(UTILS_DIR).filter((f) => f.endsWith('.ts') && !f.endsWith('.test.ts'))

for (const file of files) {
  const lines = readFileSync(join(UTILS_DIR, file), 'utf8').split('\n')
  lines.forEach((line, index) => {
    const match = EXPORTED_FUNCTION.exec(line)
    if (!match) return
    // Walk up through the JSDoc block that ends right above the function.
    let doc = ''
    for (let i = index - 1; i >= 0 && lines[i]?.trim().match(/^(\/\*\*|\*|\*\/)/); i--) {
      doc = `${lines[i]}\n${doc}`
    }
    if (!doc.includes('@example')) problems.push(`src/utils/${file}:${index + 1} ${match[2]}`)
  })
}

if (problems.length > 0) {
  console.error('Utils functions without a JSDoc @example:')
  for (const problem of problems) console.error(`  - ${problem}`)
  process.exit(1)
}
console.warn(`TypeScript utils documented (${files.length} files).`)
