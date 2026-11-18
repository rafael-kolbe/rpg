# 0006 — Enemies reveal their action; target lines replace the crosshair

- **Status:** accepted
- **Date:** 2026-11-18
- **Supersedes:** the "action itself is not revealed" part of 0005

## Context

0005 announced only the target of an enemy's next action: a "→ name" label
under the enemy and a red crosshair (with a count) on the hero. Playing the
sandbox showed that:

- Reading a label and matching it to a crosshair is slower than following a
  line from the enemy to its target.
- Knowing the target but not the action leaves the player guessing whether to
  heal, defend or ignore it; the guess adds nothing to the decision.

## Decision

- The protocol's `intent` carries the command as well (`AnnouncedIntent`:
  `command` = action, item or Defend; `target` as before). Attack is sent as
  the action it performs, so the screen names it like any other action.
- Under the enemy, the label shows the action's (or item's) name.
- A red dashed line, running toward the target, joins the enemy to each
  combatant it will hit; a whole-side action draws one per standing target, a
  self-targeted one draws none. The crosshair and its count are gone.

## Consequences

- Fights are fully telegraphed; surprise has to come from elsewhere (bosses
  changing phase, reactions) if a design ever wants it.
- Lines are placed from the layout (avatar centers) after every render and on
  resize, outside React state.
