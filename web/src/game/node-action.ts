import type { NodeView } from '../protocol/generated/NodeView.ts'

/** What using a node does, for its button: a Fluent id with arguments. */
export interface NodeAction {
  message: string
  /** Fluent ids, translated before use. */
  args: Record<string, string>
  /** Numbers, used as they are. */
  numbers?: Record<string, number>
  /** A fixed fight that can be ambushed. */
  ambush: boolean
}

/**
 * The action the current node offers, or `null` when there is nothing to do
 * (a path, a won fight, an empty chest). Arguments hold Fluent ids to
 * translate (`place`, `name`).
 */
export function nodeAction(node: NodeView | null): NodeAction | null {
  const kind = node?.kind ?? null
  switch (kind?.type) {
    case 'exit':
      return { message: 'ui-node-exit', args: { place: `instance-${kind.to}` }, ambush: false }
    case 'battle':
      if (kind.done) return null
      return kind.boss
        ? { message: 'ui-node-boss', args: {}, ambush: false }
        : { message: 'ui-node-battle', args: {}, ambush: true }
    case 'chest':
      return kind.done ? null : { message: 'ui-node-chest', args: {}, ambush: false }
    case 'npc':
      return { message: 'ui-node-npc', args: { name: kind.name }, ambush: false }
    case 'shop':
      return { message: 'ui-node-shop', args: {}, ambush: false }
    case 'temple':
      return { message: 'ui-node-temple', args: {}, ambush: false }
    case 'inn':
      return { message: 'ui-node-inn', args: {}, numbers: { price: kind.price }, ambush: false }
    case 'board':
      return { message: 'ui-node-board', args: {}, ambush: false }
    default:
      return null
  }
}
