import { describe, expect, test } from 'vitest'

import { slotOf } from './formation.ts'

describe('slotOf', () => {
  test('one stands in the middle; three put one in front of two', () => {
    expect(slotOf(1, 0)).toEqual({ depth: 50, height: 50 })
    const three = [0, 1, 2].map((i) => slotOf(3, i))
    expect(three[1]?.depth).toBeGreaterThan(three[0]?.depth ?? 100)
    expect(three[0]?.depth).toBe(three[2]?.depth)
  })

  test('every slot stays inside its side, raids included', () => {
    for (const count of [1, 2, 3, 4, 6, 8]) {
      for (let i = 0; i < count; i++) {
        const { depth, height } = slotOf(count, i)
        expect(depth).toBeGreaterThanOrEqual(0)
        expect(depth).toBeLessThanOrEqual(100)
        expect(height).toBeGreaterThan(0)
        expect(height).toBeLessThan(100)
      }
    }
  })
})
