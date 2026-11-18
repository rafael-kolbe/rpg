# 0002 — The server is authoritative in both modes

- **Status:** accepted
- **Date:** 2026-11-17

## Context

Story Mode is single-player, but its progress unlocks regions in Online Mode
and its items (tagged `Untradeable`) can be used there. If Story Mode ran in
the browser, editing the save would unlock the whole map and create items.

## Decision

All simulation (battles, map, travel, encounters, inventory, saves) runs on the
server for both modes. Story Mode is a one-player room on the server. The client
sends intentions and renders server events; it keeps only per-browser
preferences (language, volume).

## Consequences

- The game always requires a connection, even in Story Mode.
- Accounts are needed from Story Mode on.
- The engine can still run in the browser (WebAssembly) to predict and smooth
  the display, never to decide outcomes.
