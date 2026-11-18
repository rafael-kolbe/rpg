import { defineConfig } from '@playwright/test'

// End-to-end tests drive the real stack: the Rust server built with the
// sandbox feature serves the built client (`npm run build` first; `make e2e`
// does both). The database is not needed: battles live in memory, and the
// server starts with an unreachable one.
const PORT = 3100

export default defineConfig({
  testDir: 'e2e',
  timeout: 60_000,
  fullyParallel: true,
  use: { baseURL: `http://127.0.0.1:${PORT}` },
  webServer: {
    command: 'cargo run --quiet --features sandbox --bin rpg-server',
    cwd: '..',
    url: `http://127.0.0.1:${PORT}/`,
    timeout: 300_000,
    reuseExistingServer: false,
    env: {
      PORT: String(PORT),
      DATABASE_URL: 'postgres://rpg@127.0.0.1:1/rpg',
      WEB_DIR: 'web/dist',
      RUST_LOG: 'warn',
    },
  },
})
