import { describe, expect, test } from 'vitest'

import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import { soundsOf } from './battle-sounds.ts'

const info = { target: 'single_enemy', mpCost: 0, targetsKnockedOut: false } as const
const catalog: Catalog = {
  actions: {
    attack: { ...info, category: 'attack', element: null },
    fire: { ...info, category: 'magic', element: 'fire' },
  },
  items: {},
}

const started = (action: string): BattleEvent => ({
  type: 'action_started',
  actor: 0,
  action,
  item: null,
  targets: [1],
  mpSpent: 0,
})
const damaged = (critical = false): BattleEvent => ({
  type: 'damaged',
  target: 1,
  amount: 10,
  critical,
  distracted: false,
  affinity: null,
})

describe('soundsOf', () => {
  test('a blow hits, a critical hits harder, a spell sounds like its element', () => {
    expect(soundsOf([started('attack'), damaged()], catalog)).toEqual(['hit'])
    expect(soundsOf([started('attack'), damaged(true)], catalog)).toEqual(['critical'])
    expect(soundsOf([started('fire'), damaged(), damaged()], catalog)).toEqual(['fire'])
  })

  test('chimes for a hero turn but not for an enemy that acts at once', () => {
    const enemy: BattleEvent[] = [
      { type: 'turn_ready', combatant: 3 },
      { type: 'action_queued', actor: 3, action: 'attack', item: null },
    ]
    expect(soundsOf(enemy, catalog)).toEqual([])
    expect(soundsOf([{ type: 'turn_ready', combatant: 0 }], catalog)).toEqual(['turn'])
  })
})
