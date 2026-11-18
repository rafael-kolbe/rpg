import { useState } from 'react'

import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import { useStoredToggle } from '../ui/use-stored-toggle.ts'
import type { RememberedCommand } from './action-memory.ts'

/**
 * Action memory: the last command of each hero, and whether the player wants
 * menus to start on it. The switch is a per-browser preference; commands are
 * remembered even while it is off, so turning it on uses the latest ones.
 */
export function useActionMemory() {
  const [enabled, toggle] = useStoredToggle('rpg.action-memory', false)
  const [commands, setCommands] = useState<ReadonlyMap<CombatantId, RememberedCommand>>(
    () => new Map(),
  )

  return {
    enabled,
    toggle,
    /** The hero's last command while memory is on. */
    recall: (actor: CombatantId): RememberedCommand | null =>
      enabled ? (commands.get(actor) ?? null) : null,
    remember: (actor: CombatantId, command: RememberedCommand) =>
      setCommands((existing) => new Map(existing).set(actor, command)),
  }
}
