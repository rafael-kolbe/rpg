# 0005 — Enemies decide when they start charging and announce the target

- **Status:** accepted; the hidden action is superseded by 0006
- **Date:** 2026-11-18

## Context

The AI chose its action at the moment its ATB gauge filled, so the player only
learned who was hit after the hit. The player wants to see an enemy's target
while its gauge charges, to have something to react to (heal, defend, kill the
enemy first).

## Decision

- An engine-controlled combatant decides its next command (its *intent*) on
  the first tick its gauge charges, using the same rules as before.
- Every tick while charging, the intent is checked with the same validation a
  command gets (action known, MP, Silence, item in stock, eligible target); if
  it fails, the AI decides again. When the gauge fills, it performs the intent.
- The protocol sends who the intent will hit (`intent`: one combatant, a whole
  side, or itself) for engine-controlled combatants; the screen shows it for
  enemies. The action itself is not revealed.

## Consequences

- Conditions are read at the start of the charge, not at its end: an enemy may
  heal an ally who was healed meanwhile. Accepted for the telegraph; a real
  problem would show up as a rule to re-check.
- The order of random draws changed, so a seed replays a different battle than
  before this change.
- An auto-battle party also has intents (not shown).
