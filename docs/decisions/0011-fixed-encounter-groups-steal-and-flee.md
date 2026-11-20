# 0011 — Fixed encounter groups; steal and flee

- **Status:** accepted
- **Date:** 2026-11-19

## Context

[0010](0010-game-over-auto-saves-and-missions.md) drew each enemy of an
encounter from a weighted list. Bad luck could put three Goblin Shamans (Fire)
in front of a level 1 hero alone: a death without a chance. The Bandit also
needed a technique that makes it a different kind of threat.

## Decision

- A trail lists **fixed groups** chosen by hand for the area's average level,
  each with a weight; strong monsters come in limited numbers (at most one
  Shaman, Bandit or Sewer Crawler per battle). Only groups of up to 3 can be
  drawn, or up to the party's size when it is larger. This replaces the
  per-enemy draw of 0010.
- **Steal** takes one random item from the party's bag. Chance: `40 +
  (thief Dexterity − target Dexterity) + 3 × (thief level − target level)`,
  within 5–80%. A thief **flees** on its next turn with what it took;
  knocked out first, it drops it. An enemy that fled is out of the battle
  (no target, no turn) and gives no experience, money, drops or mission
  progress.

## Consequences

- Encounters are part of the content's balance: a new area writes its
  groups, and `make sim-campaign` plus the `ch1-*` sandbox scenarios check
  them.
- The engine has a "fled" state besides "knocked out"; the client hides a
  fled combatant.
