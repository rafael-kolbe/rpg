import type { ReactNode } from 'react'

import type { StatusKind } from '../protocol/generated/StatusKind.ts'
import type { StatusState } from '../protocol/generated/StatusState.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { classNames } from '../utils/class-names.ts'
import { percentOf } from '../utils/percent.ts'

/** Glyphs drawn inside the 24×24 badge, within 6..18 to clear the border. */
const GLYPHS: Record<StatusKind, ReactNode> = {
  poison: (
    <>
      <path d="M12 5c-3 4-4.5 6.5-4.5 8.5a4.5 4.5 0 0 0 9 0C16.5 11.5 15 9 12 5z" />
      <circle className="status-icon__cut" cx="10.5" cy="13.5" r="1.2" />
    </>
  ),
  regen: <path d="M10.5 6h3v4.5H18v3h-4.5V18h-3v-4.5H6v-3h4.5z" />,
  sleep: <path className="status-icon__line" d="M7 7h5l-5 5h5M13 12h4l-4 5h4" />,
  silence: (
    <>
      <path d="M6 7h12v7h-6l-3.5 3v-3H6z" />
      <path className="status-icon__line status-icon__cross" d="M5 18L19 5" />
    </>
  ),
  haste: <path className="status-icon__line" d="M6.5 7l5 5-5 5M12.5 7l5 5-5 5" />,
  slow: <path className="status-icon__line" d="M7 6h10M7 18h10M8 6l8 12M16 6l-8 12" />,
  barrier: <path d="M12 5.5l6 2v4.5c0 3.5-2.5 5.5-6 6.5-3.5-1-6-3-6-6.5V7.5z" />,
  magic_barrier: (
    <>
      <path d="M12 5.5l6 2v4.5c0 3.5-2.5 5.5-6 6.5-3.5-1-6-3-6-6.5V7.5z" />
      <path
        className="status-icon__cut"
        d="M12 8.5l1 2.3 2.4.2-1.8 1.6.6 2.4-2.2-1.3-2.2 1.3.6-2.4-1.8-1.6 2.4-.2z"
      />
    </>
  ),
  distracted: (
    <path
      className="status-icon__line"
      d="M12 11a1 1 0 1 1-1 1a2.5 2.5 0 1 1 2.5 2.5a4.5 4.5 0 1 1 4.5-4.5"
    />
  ),
}

/** The badge border, starting at 12 o'clock and going clockwise. */
const BORDER = 'M12 1.5H22.5V22.5H1.5V1.5Z'

/**
 * A square status badge with its own icon; the name is the tooltip. On a
 * timed status the border is a clock: it empties clockwise from the top as
 * the time runs out. Red border for harmful statuses, blue for the rest.
 */
export function StatusIcon({ status }: { status: StatusState }) {
  const { t } = useTranslation()
  const name = t(`ui-status-${status.kind}`)
  const { timer } = status
  const left = timer === null ? 100 : percentOf(timer.remainingTicks, timer.durationTicks)

  return (
    <span
      className={classNames(
        'status-icon',
        `status-icon--${status.kind}`,
        status.harmful && 'status-icon--harmful',
      )}
      title={name}
      role="img"
      aria-label={name}
      data-testid={`status-${status.kind}`}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path className="status-icon__track" d={BORDER} />
        <path
          className="status-icon__border"
          d={BORDER}
          pathLength={100}
          strokeDasharray="100 100"
          strokeDashoffset={left - 100}
        />
        <g className="status-icon__glyph">{GLYPHS[status.kind]}</g>
      </svg>
    </span>
  )
}
