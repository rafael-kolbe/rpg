# CLAUDE.md

Browser RPG with ATB battles (Final Fantasy 7 style). Story Mode (single-player)
and Online Mode (co-op); every rule runs on the server.

- **Game design (current, in Portuguese):** `PLAN.md`. Read the relevant section
  before implementing a feature; it is the source of truth for rules.
- **Why things are the way they are:** `docs/decisions/` (one ADR per decision).
- **This file:** where things live and how to work here. Keep it current.

## Where things live

One place per concept. Before creating a file, type or function, search for an
existing one. When a concept is added or moved, update this table in the same
change.

| Concept | Location |
| --- | --- |
| Battle loop: ticks, commands, action queue, end of battle | `crates/engine/src/battle.rs` |
| Applying action effects (damage rolls, healing, statuses) | `crates/engine/src/effects.rs` |
| Damage and healing formulas (pure math) | `crates/engine/src/damage.rs` |
| ATB gauge speed | `crates/engine/src/atb.rs` |
| Status kinds and timers | `crates/engine/src/status.rs` |
| Enemy / auto-battle AI rules and intents (announced targets) | `crates/engine/src/ai.rs` |
| Attributes and derived stats | `crates/engine/src/stats.rs` |
| Action, effect, element, item definitions | `crates/engine/src/action.rs` |
| Battle setup and its validation | `crates/engine/src/setup.rs` |
| Battle events | `crates/engine/src/event.rs` |
| Deterministic RNG | `crates/engine/src/rng.rs` |
| Engine unit-test fixtures | `crates/engine/src/test_support.rs` |
| Loading content and scenarios | `crates/content/src/lib.rs` |
| Content validation (`content-check`) | `crates/content/src/validation.rs` |
| Battle simulator CLI | `crates/sim/src/main.rs` |
| Actions and items (data) | `content/actions.json`, `content/items.json` |
| Sandbox scenarios (data) | `sandbox/*.json` |
| Sandbox-only engine tools (KO, apply status, fill gauge) | `crates/engine/src/debug.rs` |
| Generic Rust helpers (numbers, text…) | `crates/utils/src/` |
| Wire protocol: WebSocket messages | `crates/protocol/src/messages.rs` |
| Wire protocol: battle state as the client sees it | `crates/protocol/src/state.rs` |
| Wire protocol: `/api/health` | `crates/protocol/src/health.rs` |
| Battle session per WebSocket connection (`/ws`) | `crates/server/src/ws.rs` |
| Sandbox scenarios loaded by the server | `crates/server/src/sandbox.rs` |
| Generated TS types of the protocol (never edit) | `web/src/protocol/generated/` |
| Server entry point, CLI commands | `crates/server/src/main.rs` |
| Server configuration (env vars) | `crates/server/src/config.rs` |
| HTTP routes (`/api/*`, static client) | `crates/server/src/http.rs` |
| Docker healthcheck command | `crates/server/src/healthcheck.rs` |
| UI text and game text, per language | `locales/<locale>/*.ftl` |
| i18n runtime in the client | `web/src/i18n/` |
| API calls from the client | `web/src/api/` |
| Generic TypeScript helpers (classNames, percentOf, fetchJson…) | `web/src/utils/` |
| Shared UI components (Window, MenuList, Gauge, Button, Select…) and menu keyboard keys (`useMenuKeys`) | `web/src/ui/` |
| App-level components (language picker) | `web/src/components/` |
| Battle screen, command menu, party panel, HP/MP bars, enemy intents and target lines, which sound each event plays, WebSocket hook | `web/src/battle/` |
| Sound synthesizer and the sound definitions | `web/src/audio/` |
| Sandbox page and debug panel | `web/src/sandbox/` |
| Pages and routing | `web/src/pages/`, `web/src/App.tsx` |
| Browser tests (Playwright) | `web/e2e/` |
| Quality scripts used by `make check` | `scripts/`, `web/scripts/` |
| Database backup job | `scripts/backup.sh` (run by the `backup` service) |

Planned, created by the phase that needs them (see `PLAN.md` §5 and §9):
`migrations/` (SQLx).

Dependency direction (enforced by Cargo): `protocol`, `content` → `engine`;
`server` → all; `engine` never depends on I/O, network, clock or OS randomness;
`utils` depends on nothing and anyone may use it.

## Commands

| Command | What |
| --- | --- |
| `make check` | Everything that must pass before a task is done |
| `make fmt` | Format Rust and web code |
| `make dev-db` / `make dev-server` / `make dev-web` | Local development with hot reload (db on 5433, server on 3000, Vite on 5173) |
| `make up` / `make up-d` / `make down` | Production-like stack on http://127.0.0.1:8083 (needs `.env`) |
| `make protocol-types` | Regenerate TS types after changing `crates/protocol` |
| `make docs` | Build and open rustdoc for the whole workspace |
| `make sim` / `make sim RUNS=1000` | AI-vs-AI battle log / balancing summary (`SEED`, `SCENARIO`) |
| `make e2e` | Build the client and run the Playwright browser tests (starts its own sandbox server) |
| Sandbox in the browser | `make dev-server` + `make dev-web`, open `/sandbox/battle-basic` (`?seed=42&auto=1&speed=400`) |

`cargo` lives in `~/.cargo/bin`; the Makefile adds it to `PATH`. In a bare
shell, `source ~/.cargo/env` first.

## Conventions

Everything in the repository is in **English**: code, identifiers, content ids,
comments, commit messages, docs. Exception: `PLAN.md` (design conversation, in
Portuguese).

| Where | Convention | Example |
| --- | --- | --- |
| Rust types, traits, enum variants | `UpperCamelCase` | `Combatant`, `Status::Distracted` |
| Rust functions, variables, modules, files | `snake_case` | `apply_damage`, `atb_gauge.rs` |
| Rust constants | `SCREAMING_SNAKE_CASE` | `TICK_MS` |
| TS variables, functions | `camelCase` | `selectTarget` |
| React components and TS types | `PascalCase`, component file `PascalCase.tsx` | `BattleScreen.tsx` |
| Other TS files | `kebab-case` | `fetch-health.ts` |
| Content ids | `kebab-case`, unique per kind | `iron-sword`, `forest-wolf` |
| Fluent message ids | `<kind>-<id>` (+ `.attribute`) | `item-iron-sword`, `ui-server-status` |
| Postgres tables / columns | `snake_case`, tables plural | `recruits`, `created_at` |
| Protocol fields / enum values | `camelCase` / `snake_case` on the wire | `{"status": "ok"}` |
| Commits | Conventional Commits | `feat(engine): add distracted status` |

### Rust

- Idiomatic Rust, not simplified Rust: newtypes for ids, enums with data instead
  of boolean/`Option` combinations, errors as values (`thiserror` in libraries,
  `anyhow` in binaries). No `unwrap()`/`expect()` outside tests.
- **No speculative abstraction.** A trait, generic or layer needs a second real
  use. One use = concrete code.
- **`unsafe` is denied workspace-wide.** If truly unavoidable: `#[allow(unsafe_code)]`
  on the smallest block, a `// SAFETY:` comment with the invariants, an ADR in
  `docs/decisions/` explaining why no safe alternative works, and a safe wrapper
  with tests.
- Lints live in `[workspace.lints]` (`Cargo.toml`): clippy `all` + `pedantic`,
  `missing_docs`. Fix the code; do not add `#[allow]` without a comment saying
  why the lint is wrong in that spot.
- Every module starts with `//!` (what it is, why it exists); every public item
  has `///`. Link items with `` [`Name`] `` — broken links fail `make check`.

### Web

- TypeScript strict + `noUncheckedIndexedAccess`; oxlint, Prettier, knip;
  unit tests with Vitest (`*.test.ts` next to the code).
- **No user-facing string in code.** Every text is a Fluent message in
  `locales/en-US` (default) and `locales/pt-BR`, used through `t(id, args)`.
  The server sends ids + arguments, never translated text.
- State comes from the server; `localStorage` only for per-browser preferences.
- **Componentize; never repeat markup.** Generic pieces live in `web/src/ui/`
  (Window, MenuList, Gauge, Button, Select, Checkbox, TextInput) and are used
  everywhere; a screen composes them. When the same markup or styling shows up
  twice, it becomes a component. Keep files small: one component per file,
  split a component when it grows past one clear job.
- Shared component styles in `web/src/ui/ui.css`; screen styles beside the
  screen (`battle/battle.css`).
- React rules enforced by oxlint: no `setState` synchronously inside effects;
  react to server events through `subscribe` from `useBattle`.
- Every interactive element tests need gets a `data-testid`.

### Comments

Explain **why**, never what, never history ("was X", "changed to Y"). History
goes to `docs/decisions/`; the current design to `PLAN.md`.

### Tests

- Rust unit tests in the same file (`#[cfg(test)] mod tests`): deleting code
  deletes its tests.
- A test covers current behavior. Behavior removed → test removed.
  `#[ignore]`, `.skip`, `.only`, `.todo` fail `make check`.
- Name tests as the behavior: `rejects_a_port_that_is_not_a_number`.

## When a decision changes

The **same change** removes the old code, tests, content, translations and
comments; adds an ADR to `docs/decisions/`; and updates `PLAN.md` and this file.
`make check` (dead code, unused deps and exports, missing translations) must
prove nothing was left behind.

## Pending wiring (do it when the trigger happens)

- First `sqlx::query!` → add `cargo sqlx prepare --workspace --check` to
  `check-rust` and commit `.sqlx/`.
- First table → create `migrations/`, run `sqlx::migrate!` at server startup.
- People start editing content by hand → generate JSON Schemas from the engine
  types (`schemars`) for editor autocomplete, checked for staleness like the
  protocol types.

## Utils

Generic helpers live in **`crates/utils`** (Rust) and **`web/src/utils/`**
(TypeScript). Reuse before writing:

1. **Before writing any helper, read the utils folder.** If something close
   exists, use it or extend it; never write a second function that does the
   same job.
2. **What goes there** (the usual criteria): a small, pure (or thin
   platform-wrapper) function that knows nothing about the game — no
   combatants, items, ticks, rules — and is needed in two or more places, or is
   a well-known general operation. Game logic stays in its domain module even
   when short (damage formulas stay in `engine/src/damage.rs`).
3. **No dumping ground:** one file/module per topic (`numbers`, `text`,
   `storage`, `fetch-json`), never `misc` or `helpers`.
4. **Descriptive names** that say what comes out: `clamp_to_u8`,
   `saturating_u16`, `percentOf`, `initialsOf`.
5. **Documentation is mandatory:** what it does, how (edge cases), and an
   example — Rust `# Examples` (a doctest, run by `cargo test`), TypeScript
   JSDoc `@example`. `make check` fails without them
   (`scripts/check-utils-docs.sh`, `web/scripts/check-utils-docs.ts`).
6. **Tested:** Rust by its doctests (plus unit tests when needed); TypeScript
   in `web/src/utils/*.test.ts` (Vitest).

## Sandbox

- Built only with the `sandbox` Cargo feature (`server/sandbox` → `engine/sandbox`).
  `make dev-server` and `make e2e` enable it; Docker builds it when `.env` has
  `SANDBOX=true` (fine on localhost, **never** in a published image).
- A new scenario is a JSON file in `sandbox/`; the server and `content-check`
  validate it at startup / check time. Prefer a scenario over manual setup when
  testing a mechanic.

## Content rules

- Content is data: `content/` (game) and `sandbox/` (test scenarios). The engine
  never hard-codes an action, item or monster.
- Every action id needs `action-<id>`, every item `item-<id>`, every combatant
  name its message, in **every** locale (`content-check` enforces it).
- Names are our own. Never reuse names that belong to other games (no
  "Phoenix Down", no famous Limit names).

## Done means

1. `make check` is green.
2. `CLAUDE.md`, `PLAN.md` and ADRs reflect the change.
3. No leftovers: old code, stale comments, unused files, skipped tests.
