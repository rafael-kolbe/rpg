import { classNames } from '../utils/class-names.ts'
import { percentOf } from '../utils/percent.ts'

interface Props {
  value: number
  max: number
  variant: 'atb' | 'hp' | 'mp'
  /** Full and actionable (e.g. a ready ATB gauge). */
  highlight?: boolean
}

/** A thin horizontal bar; width changes animate between server ticks. */
export function Gauge({ value, max, variant, highlight = false }: Props) {
  return (
    <span className={classNames('gauge', `gauge--${variant}`, highlight && 'gauge--highlight')}>
      <span style={{ width: `${percentOf(value, max)}%` }} />
    </span>
  )
}
