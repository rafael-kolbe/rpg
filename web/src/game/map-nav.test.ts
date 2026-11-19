import { describe, expect, test } from 'vitest'

import type { InstanceView } from '../protocol/generated/InstanceView.ts'
import type { NodeView } from '../protocol/generated/NodeView.ts'
import { neighborToward, neighborsOf } from './map-nav.ts'

function node(id: string, x: number, y: number): NodeView {
  return { id, x, y, fog: 'explored', kind: { type: 'path' }, label: null }
}

const instance: InstanceView = {
  id: 'test',
  region: 'land',
  terrain: 'fields',
  nodes: [
    node('center', 500, 500),
    node('north', 520, 200),
    node('east', 800, 450),
    node('south-west', 300, 750),
    node('locked', 200, 500),
  ],
  trails: [
    { a: 'center', b: 'north', open: true },
    { a: 'east', b: 'center', open: true },
    { a: 'center', b: 'south-west', open: true },
    { a: 'center', b: 'locked', open: false },
  ],
}

describe('neighborsOf', () => {
  test('lists nodes across open trails, both ways', () => {
    expect(neighborsOf(instance, 'center').map((n) => n.id)).toEqual([
      'north',
      'east',
      'south-west',
    ])
  })
})

describe('neighborToward', () => {
  test('picks the neighbor most in the pressed direction', () => {
    expect(neighborToward(instance, 'center', 'up')?.id).toBe('north')
    expect(neighborToward(instance, 'center', 'right')?.id).toBe('east')
    expect(neighborToward(instance, 'center', 'down')?.id).toBe('south-west')
  })

  test('ignores closed trails and directions with no trail', () => {
    expect(neighborToward(instance, 'center', 'left')?.id).toBe('south-west')
    expect(neighborToward(instance, 'north', 'up')).toBeNull()
  })
})
