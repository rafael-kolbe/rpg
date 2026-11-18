import { describe, expect, test } from 'vitest'

import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import { effectsOf } from './popups.ts'

const info = { target: 'single_enemy', mpCost: 0, targetsKnockedOut: false } as const
const catalog: Catalog = {
  actions: {
    attack: { ...info, category: 'attack', element: null },
    fire: { ...info, category: 'magic', element: 'fire' },
  },
  items: {},
}

function hit(action: string): BattleEvent[] {
  return [
    { type: 'action_started', actor: 0, action, item: null, targets: [1], mpSpent: 0 },
    { type: 'damaged', target: 1, amount: 9, critical: false, distracted: false, affinity: null },
  ]
}

describe('effectsOf', () => {
  test('damage numbers carry the element of the action that dealt them', () => {
    expect(effectsOf(1, hit('fire'), catalog).popups[0]?.element).toBe('fire')
    expect(effectsOf(1, hit('attack'), catalog).popups[0]?.element).toBeUndefined()
  })
})
