import { defineConfig } from 'vitest/config'

// Unit tests only (src/**/*.test.ts); browser tests live in e2e/ (Playwright).
export default defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
  },
})
