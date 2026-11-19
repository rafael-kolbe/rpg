import type { InstanceView } from '../protocol/generated/InstanceView.ts'
import type { NodeId } from '../protocol/generated/NodeId.ts'
import type { NodeView } from '../protocol/generated/NodeView.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { classNames } from '../utils/class-names.ts'
import { neighborsOf } from './map-nav.ts'
import { NodeIcon } from './NodeIcon.tsx'

interface Props {
  instance: InstanceView
  position: NodeId
  onMove: (node: NodeId) => void
  onInteract: () => void
}

/** Fog cleared around a visited node, and around a glimpsed one. */
const CLEAR_EXPLORED = 190
const CLEAR_GLIMPSED = 95

/** The name shown under a node, if any. */
function nodeName(node: NodeView, t: (id: string) => string): string | null {
  if (node.label) return t(node.label)
  if (node.kind?.type === 'npc') return t(node.kind.name)
  if (node.kind?.type === 'exit') return `→ ${t(`instance-${node.kind.to}`)}`
  return null
}

/**
 * The close-up map: trails, nodes and the fog of war, seen from above.
 * Clicking a neighbor walks there; clicking the current node uses it.
 */
export function InstanceMap({ instance, position, onMove, onInteract }: Props) {
  const { t } = useTranslation()
  const byId = new Map(instance.nodes.map((node) => [node.id, node]))
  const reachable = new Set(neighborsOf(instance, position).map((node) => node.id))
  const fogId = `fog-${instance.id}`

  return (
    <svg
      className={`instance-map instance-map--${instance.terrain}`}
      viewBox="0 0 1000 1000"
      data-testid="instance-map"
    >
      <defs>
        <filter id="fog-soften" x="-20%" y="-20%" width="140%" height="140%">
          <feGaussianBlur stdDeviation="45" />
        </filter>
        <mask id={fogId}>
          <rect width="1000" height="1000" fill="white" />
          <g filter="url(#fog-soften)">
            {instance.nodes.map((node) => (
              <circle
                key={node.id}
                cx={node.x}
                cy={node.y}
                r={node.fog === 'explored' ? CLEAR_EXPLORED : CLEAR_GLIMPSED}
                fill="black"
              />
            ))}
          </g>
        </mask>
      </defs>
      <rect className="instance-map__ground" width="1000" height="1000" />
      {instance.trails.map((trail) => {
        const a = byId.get(trail.a)
        const b = byId.get(trail.b)
        if (!a || !b) return null
        return (
          <line
            key={`${trail.a}-${trail.b}`}
            className={classNames('trail', !trail.open && 'trail--closed')}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
          />
        )
      })}
      <rect className="instance-map__fog" width="1000" height="1000" mask={`url(#${fogId})`} />
      {instance.nodes.map((node) => {
        const here = node.id === position
        const canWalk = reachable.has(node.id) && node.fog !== 'barrier'
        const name = nodeName(node, t)
        return (
          <g
            key={node.id}
            className={classNames(
              'map-node',
              here && 'map-node--here',
              canWalk && 'map-node--reachable',
            )}
            transform={`translate(${node.x} ${node.y})`}
            onClick={() => {
              if (here) onInteract()
              else if (canWalk) onMove(node.id)
            }}
            data-testid={`node-${node.id}`}
            role={here || canWalk ? 'button' : undefined}
            aria-label={name ?? node.id}
          >
            {here && <circle className="map-node__marker" r="32" />}
            <NodeIcon node={node} />
            {name !== null && (
              <text className="map-node__name" y="44" textAnchor="middle">
                {name}
              </text>
            )}
          </g>
        )
      })}
    </svg>
  )
}
