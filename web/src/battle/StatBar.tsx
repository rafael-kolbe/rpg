import { Gauge } from '../ui/Gauge.tsx'

interface Props {
  label: string
  value: number
  max: number
  variant: 'hp' | 'mp'
}

/** "HP 312/400" with its bar right below. */
export function StatBar({ label, value, max, variant }: Props) {
  return (
    <span className="stat-bar">
      <span className="stat-bar__text">
        <span className="stat-bar__label">{label}</span> {value}/{max}
      </span>
      <Gauge value={value} max={max} variant={variant} />
    </span>
  )
}
