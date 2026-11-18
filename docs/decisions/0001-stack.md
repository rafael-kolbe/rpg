# 0001 — Stack: Rust server, Postgres, Vite + React client

- **Status:** accepted
- **Date:** 2026-11-17

## Context

An ATB battle RPG with a single-player Story Mode and a co-op Online Mode. Every
rule must run on the server (Story Mode progress unlocks Online Mode content, so
a client-side save could be forged). The design is a large set of state machines
(commands, statuses, ambushes, node types, travel modes, item tags), the code is
written mostly by an AI assistant across many sessions, and drift (duplicated
code, stale decisions, stale comments) is the main risk.

## Decision

- **Rust** for the server and the rules engine: enums with data and exhaustive
  `match` model the rules; `dead_code` as an error and crate boundaries catch
  drift at compile time; the engine can compile to WebAssembly for the client.
  `tokio` + `axum`, one actor (task) per room.
- **Postgres 17** with **SQLx** (compile-time checked SQL, no ORM). Game state
  lives in memory in the room actor; the database receives checkpoints.
- **Vite + React + TypeScript SPA**, no SSR: the game lives behind a login on a
  WebSocket, so server rendering brings nothing.
- **WebSocket + JSON**, with TypeScript types generated from Rust (`ts-rs`).
- **Project Fluent** for translations (`en-US` default, `pt-BR`).
- **oxlint** instead of ESLint: it is the current Vite template default, fast,
  and covers the rules we need.

## Alternatives considered

- **Go:** simpler and fast to write, but weaker type system (`nil`, no sum
  types) for a rules-heavy domain, and a heavier WebAssembly story.
- **C:** no safety for no real gain at 10 ticks per second.
- **Node + TypeScript** (as the other projects on this server): fastest to start,
  but dead code and unknown fields pass silently.
- **Next.js with SSR:** adds a Node server and solves SEO/first paint, which a
  logged-in real-time game does not need.
- **ORMs (Diesel, SeaORM):** fight complex queries; SQLx gives checked SQL.

## Consequences

- Slower to write than Go/Node, especially early on.
- The project needs Rust and Node toolchains locally; production builds in
  Docker with the versions pinned in `rust-toolchain.toml` and the Dockerfile.
