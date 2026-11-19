# 0008 — The campaign runs on the server from data; saves are JSON documents

- **Status:** accepted; defeat, party size and encounters amended by [0010](0010-game-over-auto-saves-and-missions.md)
- **Date:** 2026-11-18

## Context

The first campaign arc (`PLAN.md` §4, "Campanha do MVP") needs a world map,
instances with fog of war, story flags, dialogues, shops, experience and
saves per mode. Rules must run on the server (ADR 0002), and the story is
content that will keep changing.

## Decision

- A new pure crate, `world` (no I/O, like `engine`), holds the campaign
  rules: the content types (`def`), a `Game` that takes player commands and
  orders battles, progression, the fog-of-war view sent to the client, and
  content validation. The engine stays about battles only.
- The whole story is data in `content/world/`: regions, instances (nodes and
  trails), heroes and classes, monsters, dialogues, shops and equipment.
  Story flags gate trails, nodes, arrival scenes and talks; story effects
  (flags, a hero joining, items, money, healing, a battle) run after
  dialogues and won fights. No story logic lives in code.
- The WebSocket connection owns the player's `Game` and the battle it orders;
  the battle's end goes back to the game, and the game is stored after every
  change.
- Storage: two JSONB documents per account, owned by the `world` crate:
  `progress` (story flags and explored nodes, shared by both modes; only the
  Campaign writes it) and one `save` per mode (position, party, bags, money,
  return temple, RNG state). The Multiplayer sees unexplored places as
  barriers and stays locked until a story flag opens it.
- A bot in the simulator plays the whole arc with the engine fighting for the
  party; a test runs it, so content changes that make the arc unbeatable fail
  `make check`. `make sim-campaign` prints levels, defeats and battle time
  for balancing.

## Consequences

- Writing story is editing JSON and Fluent files; `content-check` validates
  every reference, every hero and monster in a real battle setup, and every
  message in both locales.
- JSON documents keep the schema simple while the rules move fast; saves
  written by an older build must keep deserializing (add fields with
  defaults, never rename without a migration of the documents).
- Scenes (an open dialogue, unclosed rewards) are not saved: a dropped
  connection resumes on the map, and an arrival scene still due plays again.
  Effects of a won fight are applied at once, so they are never lost.
- Battles are saved (`Save::battle`) with their record (`engine::BattleRecord`:
  accepted commands with their tick, and the tick reached). The engine is
  deterministic, so replaying the record over the same setup and seed
  rebuilds the battle exactly: a reload resumes it where it was. Restarting
  it from the beginning was rejected: reloading a losing fight would undo
  it, a save point on demand. The server stores the record at each command,
  every second while time flows, and when the connection closes. If the
  content changed under a saved battle and the record no longer fits, the
  battle starts over.
- Placeholder art: nodes, maps and avatars are simple SVG shapes until the
  art pass.
