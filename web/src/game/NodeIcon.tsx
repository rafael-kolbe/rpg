import type { NodeView } from '../protocol/generated/NodeView.ts'

interface Props {
  node: NodeView
}

/** Our own glyph for each node kind, drawn around (0, 0), about 36 units wide. */
export function NodeIcon({ node }: Props) {
  if (node.fog === 'barrier') {
    return (
      <g className="node-icon node-icon--barrier">
        <circle r="18" />
        <path d="M-9 -9 L9 9 M9 -9 L-9 9" />
      </g>
    )
  }
  const kind = node.kind
  if (kind === null) {
    return (
      <g className="node-icon node-icon--glimpsed">
        <circle r="14" />
        <text y="6" textAnchor="middle">
          ?
        </text>
      </g>
    )
  }
  switch (kind.type) {
    case 'path':
      return <circle className="node-icon node-icon--path" r="9" />
    case 'exit':
      return (
        <g className="node-icon node-icon--exit">
          <circle r="18" />
          <path d="M-9 0 H8 M2 -7 L9 0 L2 7" />
        </g>
      )
    case 'battle':
      return kind.boss ? (
        <g className="node-icon node-icon--boss" data-done={kind.done}>
          <circle r="22" />
          <path d="M-9 -2 a9 9 0 1 1 18 0 v5 h-4 v4 h-10 v-4 h-4 Z" />
          <circle cx="-4" cy="-2" r="2.2" className="node-icon__hole" />
          <circle cx="4" cy="-2" r="2.2" className="node-icon__hole" />
        </g>
      ) : (
        <g className="node-icon node-icon--battle" data-done={kind.done}>
          <circle r="19" />
          <path d="M-10 -10 L10 10 M10 -10 L-10 10 M-10 5 L-5 10 M10 5 L5 10" />
        </g>
      )
    case 'chest':
      return (
        <g className="node-icon node-icon--chest" data-done={kind.done}>
          <circle r="18" />
          <rect x="-10" y="-6" width="20" height="13" rx="2" />
          <path d="M-10 -1 H10 M0 -1 V3" />
        </g>
      )
    case 'npc':
      return (
        <g className="node-icon node-icon--npc">
          <circle r="18" />
          <circle cy="-5" r="5" />
          <path d="M-9 11 a9 8 0 0 1 18 0" />
        </g>
      )
    case 'shop':
      return (
        <g className="node-icon node-icon--shop">
          <circle r="18" />
          <path d="M-9 -3 H9 L7 10 H-7 Z M-4 -3 a4 5 0 0 1 8 0" />
        </g>
      )
    case 'temple':
      return (
        <g className="node-icon node-icon--temple">
          <circle r="19" />
          <path d="M0 -11 V11 M-7 -4 H7" />
        </g>
      )
  }
}
