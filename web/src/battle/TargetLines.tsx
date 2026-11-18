import { useLayoutEffect, useRef } from 'react'

import type { TargetLine } from './intents.ts'

interface Props {
  lines: TargetLine[]
}

/** Center of the avatar of `id`, relative to `origin`. */
function avatarCenter(field: Element, origin: DOMRect, id: string) {
  const avatar = field.querySelector(`[data-avatar="${id}"]`)
  if (avatar === null) return null
  const box = avatar.getBoundingClientRect()
  return { x: box.left + box.width / 2 - origin.left, y: box.top + box.height / 2 - origin.top }
}

/** Moves every line of `svg` onto the avatars it joins, hiding the broken ones. */
function placeLines(svg: SVGSVGElement) {
  const field = svg.parentElement
  if (field === null) return
  const origin = svg.getBoundingClientRect()
  for (const line of svg.querySelectorAll('line')) {
    const from = avatarCenter(field, origin, line.dataset.from ?? '')
    const to = avatarCenter(field, origin, line.dataset.to ?? '')
    line.style.visibility = from && to ? 'visible' : 'hidden'
    if (!from || !to) continue
    line.setAttribute('x1', String(from.x))
    line.setAttribute('y1', String(from.y))
    line.setAttribute('x2', String(to.x))
    line.setAttribute('y2', String(to.y))
  }
}

/**
 * Red lines from each charging enemy to its announced targets, drawn behind
 * the combatants. Must be a child of the element holding the avatars.
 * Positions come from the layout, so they are written straight to the SVG
 * after every render and on resize instead of going through state.
 */
export function TargetLines({ lines }: Props) {
  const svg = useRef<SVGSVGElement>(null)

  useLayoutEffect(() => {
    if (svg.current) placeLines(svg.current)
  })

  useLayoutEffect(() => {
    const node = svg.current
    if (!node?.parentElement) return
    const observer = new ResizeObserver(() => placeLines(node))
    observer.observe(node.parentElement)
    return () => observer.disconnect()
  }, [])

  return (
    <svg ref={svg} className="target-lines" aria-hidden="true" data-testid="target-lines">
      {lines.map(({ from, to }) => (
        <line key={`${from}-${to}`} data-from={from} data-to={to} />
      ))}
    </svg>
  )
}
