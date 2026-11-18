import { describe, expect, test } from 'vitest'

import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import { rememberedEntry, repeatCommand } from './action-memory.ts'

const catalog: Catalog = {
  actions: {
    attack: {
      category: 'attack',
      target: 'single_enemy',
      mpCost: 0,
      targetsKnockedOut: false,
      element: null,
    },
    fire: {
      category: 'magic',
      target: 'single_enemy',
      mpCost: 8,
      targetsKnockedOut: false,
      element: null,
    },
    'twin-cleave': {
      category: 'technic',
      target: 'single_enemy',
      mpCost: 6,
      targetsKnockedOut: false,
      element: null,
    },
    potion: {
      category: 'item',
      target: 'single_ally',
      mpCost: 0,
      targetsKnockedOut: false,
      element: null,
    },
  },
  items: { potion: { action: 'potion' } },
}

function combatant(id: number, side: 'party' | 'enemies', hp = 100): CombatantState {
  return {
    id,
    side,
    name: `c${id}`,
    level: 1,
    hp,
    maxHp: 100,
    mp: 10,
    maxMp: 10,
    atb: 0,
    atbFull: 10_000,
    turn: 'charging',
    statuses: [],
    defending: false,
    playerControlled: side === 'party',
    intent: null,
    attack: 'attack',
    actions: ['fire', 'twin-cleave'],
  }
}

const hero = combatant(0, 'party')

function battle(enemies: CombatantState[], potions = 1): BattleState {
  return {
    tick: 0,
    outcome: null,
    combatants: [hero, ...enemies],
    inventory: [{ item: 'potion', count: potions }],
    timeMode: 'active',
    speedPercent: 100,
    paused: false,
  }
}

describe('rememberedEntry', () => {
  test('starts on the command, then on the spell inside its submenu', () => {
    const fire = { kind: { type: 'action', id: 'fire' }, target: 1 } as const
    expect(rememberedEntry(fire, 'root', catalog)).toBe('magic')
    expect(rememberedEntry(fire, 'magic', catalog)).toBe('fire')
    expect(rememberedEntry(fire, 'technic', catalog)).toBeUndefined()
  })

  test('starts on the item inside the item menu', () => {
    const potion = { kind: { type: 'item', id: 'potion' }, target: 0 } as const
    expect(rememberedEntry(potion, 'root', catalog)).toBe('item')
    expect(rememberedEntry(potion, 'item', catalog)).toBe('potion')
  })

  test('keeps the default cursor without memory or after Defend', () => {
    expect(rememberedEntry(null, 'root', catalog)).toBeUndefined()
    expect(rememberedEntry({ kind: { type: 'defend' }, target: null }, 'root', catalog)).toBe(
      undefined,
    )
  })
})

describe('repeatCommand', () => {
  test('repeats the command on the same target', () => {
    const state = battle([combatant(1, 'enemies'), combatant(2, 'enemies')])
    const again = repeatCommand({ kind: { type: 'attack' }, target: 2 }, hero, state, catalog)
    expect(again).toEqual({ kind: { type: 'attack' }, target: 2 })
  })

  test('moves to the first valid target when the old one is knocked out', () => {
    const state = battle([combatant(1, 'enemies'), combatant(2, 'enemies', 0)])
    const again = repeatCommand({ kind: { type: 'attack' }, target: 2 }, hero, state, catalog)
    expect(again?.target).toBe(1)
  })

  test('gives up when the choice cannot be used any more', () => {
    const state = battle([combatant(1, 'enemies')], 0)
    const potion = { kind: { type: 'item', id: 'potion' }, target: 0 } as const
    expect(repeatCommand(potion, hero, state, catalog)).toBeNull()
    const poor = { ...hero, mp: 2 }
    const fire = { kind: { type: 'action', id: 'fire' }, target: 1 } as const
    expect(repeatCommand(fire, poor, state, catalog)).toBeNull()
  })
})
