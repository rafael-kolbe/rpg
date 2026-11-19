import type { InstanceView } from '../protocol/generated/InstanceView.ts'
import type { NodeId } from '../protocol/generated/NodeId.ts'
import type { NodeView } from '../protocol/generated/NodeView.ts'

/** A direction pressed on the map. */
export type Direction = 'up' | 'down' | 'left' | 'right'

const VECTORS: Record<Direction, [number, number]> = {
  up: [0, -1],
  down: [0, 1],
  left: [-1, 0],
  right: [1, 0],
}

/** Least alignment (cosine) between a direction and a trail to follow it. */
const MIN_ALIGNMENT = 0.35

/** Nodes one open trail away from `from`. */
export function neighborsOf(instance: InstanceView, from: NodeId): NodeView[] {
  const ids = instance.trails.flatMap((trail) => {
    if (!trail.open) return []
    if (trail.a === from) return [trail.b]
    if (trail.b === from) return [trail.a]
    return []
  })
  return instance.nodes.filter((node) => ids.includes(node.id))
}

/**
 * The neighbor of `from` that lies most in `direction` (screen coordinates,
 * y down), or `null` when no trail goes roughly that way.
 */
export function neighborToward(
  instance: InstanceView,
  from: NodeId,
  direction: Direction,
): NodeView | null {
  const origin = instance.nodes.find((node) => node.id === from)
  if (!origin) return null
  const [vx, vy] = VECTORS[direction]
  let best: { node: NodeView; alignment: number } | null = null
  for (const node of neighborsOf(instance, from)) {
    const dx = node.x - origin.x
    const dy = node.y - origin.y
    const length = Math.hypot(dx, dy)
    if (length === 0) continue
    const alignment = (dx * vx + dy * vy) / length
    if (alignment >= MIN_ALIGNMENT && (best === null || alignment > best.alignment)) {
      best = { node, alignment }
    }
  }
  return best?.node ?? null
}
