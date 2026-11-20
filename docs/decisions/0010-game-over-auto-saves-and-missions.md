# 0010 — Game Over with auto saves, a party of four, missions

- **Status:** accepted; encounters amended by [0011](0011-fixed-encounter-groups-steal-and-flee.md)
- **Date:** 2026-11-19

## Context

The real first chapter (`STORY.md`) replaced the pilot story and asked for
rules the pilot did not have: a Game Over in the Campaign instead of a free
trip back to the temple, guests who fight on their own, a party of four,
missions with a counter and a Guild rank, a single world currency, and
places that stay closed until later chapters. The pilot's saves point at
places that no longer exist.

## Decision

- **Defeat.** The Campaign shows a Game Over and goes back to its last
  **auto save** (save and story progress as they were; the encounter luck
  goes on so the same ambush does not wait after every Game Over). The
  Multiplayer keeps the resurrection at the last Temple.
- **Auto saves** (Campaign only): on entering an instance, when a scene says
  so, when resting (temple, inn) and before every boss. A boss on a node is
  saved before the fight, back on the map; a boss that ambushes on arrival
  is saved one node back. Saving with the battle already owed would trap a
  player who cannot win it.
- **Party** of up to four. Story effects make heroes join and leave; a hero
  who leaves keeps level and gear for a return. **Guests** act on their own
  AI and take no experience.
- **Encounters** draw 1 to 3 enemies (up to the party's size when larger)
  from a weighted list per trail, instead of fixed groups.
- **Missions** kill monsters of given kinds; a board offers those of the
  player's rank or below; the story can give one; a count can trigger a
  scene; completion pays Kalds. The screen shows the campaign objective and
  the missions under way.
- **Kald** is the money of every kingdom.
- **Experience** to the next level is `12 × level³`.
- **Content epoch** (`start.json`): a save or progress of another epoch
  starts over instead of failing to load.
- Battle engine: a **leader** whose fall ends the battle and takes its
  servants with it; **Cover** (70% to take a single-target hit meant for an
  ally); **Devour** (consume an ally's HP) and **Reanimate** (raise a fallen
  ally as another creature), aimed at allies by name.

## Consequences

- Losing costs what happened since the last auto save; resting is how a
  player keeps progress safe, so story-locked areas need rest points (the
  Burrowood spring, the sewer maintenance rooms).
- The campaign bot trains before bosses and rests when hurt, like a player;
  `make sim-campaign` and the arc test cover chapter 1.
- Old pilot saves start over once (epoch 1).
