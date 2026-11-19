import { describe, expect, test } from 'vitest'

import type { NodeView } from '../protocol/generated/NodeView.ts'
import { nodeAction } from './node-action.ts'

function node(kind: NodeView['kind']): NodeView {
  return { id: 'n', x: 0, y: 0, fog: 'explored', kind, label: null }
}

describe('nodeAction', () => {
  test('offers what a node does, and nothing on paths or finished nodes', () => {
    expect(nodeAction(node({ type: 'path' }))).toBeNull()
    expect(nodeAction(node({ type: 'chest', done: true }))).toBeNull()
    expect(nodeAction(node({ type: 'battle', boss: false, done: true }))).toBeNull()
    expect(nodeAction(node({ type: 'battle', boss: false, done: false }))?.ambush).toBe(true)
    expect(nodeAction(node({ type: 'battle', boss: true, done: false }))?.ambush).toBe(false)
    expect(nodeAction(node({ type: 'exit', to: 'cave' }))?.args).toEqual({ place: 'instance-cave' })
    expect(nodeAction(node({ type: 'inn', price: 30 }))?.numbers).toEqual({ price: 30 })
    expect(nodeAction(node({ type: 'sealed' }))).toBeNull()
  })

  test('glimpsed nodes and no node offer nothing', () => {
    expect(nodeAction(node(null))).toBeNull()
    expect(nodeAction(null)).toBeNull()
  })
})
