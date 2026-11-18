# 0004 — ATB: 8 s base, Dexterity speeds it up to a 4 s cap

- **Status:** accepted
- **Date:** 2026-11-18

## Context

The gauge gained `2.5% × (Dexterity + 50) / 100` per tick: 4 s at Dexterity
50, but 1.3 s at 255, more than three times as fast. Playing the sandbox showed
two problems:

- A fast combatant acts so often that the ATB timing stops mattering, and Haste
  makes it worse.
- Even 4 s at the base is fun but leaves little time to choose a command,
  target and react to what the enemy is about to do.

## Decision

`gain per tick = 1.25% × (min(Dexterity, 255) + 155) / 205 × status modifier`

- 8 s at Dexterity 50 with no status; 4 s at Dexterity 255; Dexterity above
  255 gives no extra speed. Haste still doubles the speed (2 s at best).
- The same rule for heroes and enemies.

The curve is linear in **gauge speed** (turns per minute), not in seconds:
every Dexterity point adds the same number of turns, so a stat point is worth
the same at any level and players can predict it. In seconds this already
gives diminishing returns (50→100 saves 1.5 s, 205→255 saves 0.6 s). A curve
bending further (square root, logarithm) would make high Dexterity feel
wasted; a steeper one would bring back the runaway speed this cap removes.

## Consequences

- At most twice the base speed from attributes (four times with Haste).
- The whole 1–255 range spans 10.6 s to 4 s; battles last about twice as long.
- Status durations, poison pulses and action animations keep their times, so
  they weigh more per turn. Balance numbers from the simulator need a new pass.
