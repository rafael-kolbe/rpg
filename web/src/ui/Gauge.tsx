import { classNames } from './class-names.ts'

interface Props {
  value: number
  max: number
  variant: 'atb' | 'limit'
  /** Full and actionable (e.g. a ready ATB gauge). */
  highlight?: boolean
}

/** A thin horizontal bar; width changes animate between server ticks. */
export function Gauge({ value, max, variant, highlight = false }: Props) {
  const percent = max > 0 ? Math.min(100, (value * 100) / max) : 0
  return (
    <span className={classNames('gauge', `gauge--${variant}`, highlight && 'gauge--highlight')}>
      <span style={{ width: `${percent}%` }} />
    </span>
  )
}
