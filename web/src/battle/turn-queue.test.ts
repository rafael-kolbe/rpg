import { describe, expect, test } from 'vitest'

import { enqueueTurns, menuActor } from './turn-queue.ts'

describe('menuActor', () => {
  test('keeps the first hero that became ready', () => {
    expect(menuActor([0, 2], null, [2, 0])).toBe(2)
  })

  test('serves the hero the player picked while it is ready', () => {
    expect(menuActor([0, 2], 0, [2, 0])).toBe(0)
    expect(menuActor([2], 0, [2, 0])).toBe(2)
  })

  test('falls back to party order for turns that never filled', () => {
    expect(menuActor([1, 2], null, [])).toBe(1)
    expect(menuActor([], null, [1])).toBeNull()
  })
})

describe('enqueueTurns', () => {
  test('moves refilled heroes to the back of the line', () => {
    expect(enqueueTurns([0, 1, 2], [0])).toEqual([1, 2, 0])
  })
})
