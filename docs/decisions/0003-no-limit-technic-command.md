# 0003 — No Limit; physical skills are Technics; Defend leaves the menu

- **Status:** accepted
- **Date:** 2026-11-18

## Context

The battle had a Limit gauge (filled by damage taken) that turned Attack into a
special move, a generic "Ability" command and Defend on the root menu. Playing
the sandbox showed that:

- Limit reads as a copy of another game's signature mechanic; we want our own
  special-move idea instead.
- "Ability" said nothing about what the command holds. The class skills that
  are not magic are physical techniques.
- Defend (half damage until the next action, spending the turn) is rarely worth
  a turn in an ATB battle: it only pays off for a tank holding out until healed
  or when the defender acts much more often than the enemy. A root-menu entry
  for it is noise.

## Decision

- Remove the Limit gauge, the `limit` action category, the Limit command and
  the `LimitReady` event. A replacement special move is an open design item in
  `PLAN.md`.
- The `ability` action category and the menu command become `technic`
  ("Technic" / "Técnica"). `twin-cleave` is now a Warrior technic that costs MP.
- Defend keeps its rule but has no menu entry: ← on the root menu defends.

## Consequences

- Fewer gauges on screen and in the protocol (`limit`, `limitAction` gone).
- Mouse-only players cannot defend; acceptable while it stays a niche command.
- Disconnected online players still defend automatically (`PLAN.md` §7).
