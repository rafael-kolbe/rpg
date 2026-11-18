import { describe, expect, test } from 'vitest'

import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { IntentTarget } from '../protocol/generated/IntentTarget.ts'
import { intentMessage, targetLines } from './intents.ts'

function combatant(
  id: number,
  side: CombatantState['side'],
  target: IntentTarget | null = null,
): CombatantState {
  return {
    id,
    side,
    name: `c${id}`,
    level: 1,
    hp: 10,
    maxHp: 10,
    mp: 0,
    maxMp: 0,
    atb: 0,
    atbFull: 10_000,
    turn: 'charging',
    statuses: [],
    defending: false,
    playerControlled: side === 'party',
    intent: target && { command: { type: 'action', id: 'bite' }, target },
    attack: 'attack',
    actions: [],
  }
}

describe('targetLines', () => {
  test('draws a line to each combatant an enemy will hit, none for itself', () => {
    const fallen = { ...combatant(2, 'party'), hp: 0 }
    const all = [
      combatant(0, 'party'),
      combatant(1, 'party'),
      fallen,
      combatant(3, 'enemies', { type: 'one', target: 1 }),
      combatant(4, 'enemies', { type: 'side', side: 'party' }),
      combatant(5, 'enemies', { type: 'user' }),
      combatant(6, 'party', { type: 'one', target: 3 }),
    ]
    expect(targetLines(all)).toEqual([
      { from: 3, to: 1 },
      { from: 4, to: 0 },
      { from: 4, to: 1 },
      { from: 4, to: 6 },
    ])
  })
})

describe('intentMessage', () => {
  test('names actions, items and Defend by their messages', () => {
    const target: IntentTarget = { type: 'user' }
    expect(intentMessage({ command: { type: 'action', id: 'bite' }, target })).toBe('action-bite')
    expect(intentMessage({ command: { type: 'item', id: 'potion' }, target })).toBe('item-potion')
    expect(intentMessage({ command: { type: 'defend' }, target })).toBe('ui-intent-defend')
  })
})
