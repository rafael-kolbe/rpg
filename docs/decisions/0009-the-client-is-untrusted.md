# 0009 — The client is untrusted, including its own player

- **Status:** accepted
- **Date:** 2026-11-18

## Context

The game runs in a browser. Anyone can open the developer tools, read what
the page received, send any WebSocket message from the console or a script,
and replay requests. [0002](0002-server-authoritative.md) already keeps every
rule on the server, so a message can only ask for something (move, attack
this target, buy this item), never state a result (HP, damage, money). A
review of what the client can send found gaps around that core:

- Sandbox debug tools (knock out, apply status, fill gauge) worked on game
  battles in sandbox builds, and `.env.example` builds the sandbox.
- Two tabs of one account each held a copy of the game and overwrote each
  other's save: a tab opened before a losing fight could store the world as
  it was before it, undoing the fight.
- The WebSocket handshake did not check `Origin`. The session cookie is
  `SameSite=Lax`, which a page on a sibling subdomain (same site) still gets
  sent with a socket; that page could play the visitor's game.
- Messages had axum's default size limit (64 MiB) and no rate limit; every
  game command writes the save.

## Decision

- Debug tools only touch sandbox battles.
- One live game per account: starting a game claims it
  (`crates/server/src/open_games.rs`); an older connection is closed with
  `opened_elsewhere` at its next message or tick and never stores again.
- The handshake's `Origin`, when present, must match `Host`; otherwise 403.
- Messages are at most 16 KiB; a connection may send 60 at once and 30 per
  second after that (`too_many_messages` beyond).
- What the client sees is limited to what the player may know: the server
  hides fog-covered nodes and the content of glimpsed ones.

## Consequences

- Reading or editing the page gains nothing the player could not do by
  playing: the console is just another client.
- What stays possible: automating play (a bot sends the same messages a
  player would; in single-player this only hurts its own fun), and anyone
  holding a player's session cookie is that player (it is `httpOnly`, so
  page scripts cannot read it; the player must not hand it out).
- A tab closed by a newer one loses at most the last second of a battle's
  record (stored every second); a reload never does, since the connection
  stores on close.
- Spoilers in received data (monster names in a battle) are not secrets.
