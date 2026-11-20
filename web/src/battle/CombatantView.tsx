import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Gauge } from '../ui/Gauge.tsx'
import { classNames } from '../utils/class-names.ts'
import { initialsOf } from '../utils/initials.ts'
import type { Popup, Strike } from './popups.ts'
import { StatusIcon } from './StatusIcon.tsx'
import { Vitals } from './Vitals.tsx'

interface Props {
  combatant: CombatantState
  popups: Popup[]
  strikes: Strike[]
  targetable: boolean
  /** The keyboard target cursor is on it. */
  cursor: boolean
  /** Enemies only: the name of its announced action, ready to show. */
  intent: string | null
  onTarget: () => void
}

/**
 * Avatar placeholder, name, statuses and floating numbers of one combatant.
 * Enemies also show HP, MP and ATB here; the party has its own panel.
 */
export function CombatantView({
  combatant,
  popups,
  strikes,
  targetable,
  cursor,
  intent,
  onTarget,
}: Props) {
  const { t } = useTranslation()
  const name = t(combatant.name)
  const distracted = combatant.statuses.some((s) => s.kind === 'distracted')
  const className = classNames(
    'combatant',
    `combatant--${combatant.side}`,
    combatant.hp === 0 && 'combatant--knocked-out',
    combatant.fled && 'combatant--fled',
    distracted && 'combatant--distracted',
    targetable && 'combatant--targetable',
    cursor && 'combatant--cursor',
  )

  return (
    <button
      type="button"
      className={className}
      disabled={!targetable}
      onClick={onTarget}
      data-testid={`combatant-${combatant.id}`}
      aria-label={name}
    >
      <span className="combatant__avatar" aria-hidden="true" data-avatar={combatant.id}>
        <svg viewBox="0 0 64 64">
          <circle cx="32" cy="32" r="30" />
          <text x="32" y="39" textAnchor="middle">
            {initialsOf(name)}
          </text>
        </svg>
        {strikes.map((strike) => (
          <span key={strike.key} className={`strike strike--${strike.kind}`} />
        ))}
      </span>
      <span className="combatant__name">{name}</span>
      {combatant.side === 'enemies' && (
        <span className="combatant__vitals">
          <Vitals combatant={combatant} />
          <Gauge value={combatant.atb} max={combatant.atbFull} variant="atb" />
          {intent !== null && (
            <span className="combatant__intent" data-testid="intent">
              {intent}
            </span>
          )}
        </span>
      )}
      <span className="combatant__statuses">
        {combatant.statuses.map((status) => (
          <StatusIcon key={status.kind} status={status} />
        ))}
      </span>
      {popups.map((popup) => (
        <span
          key={popup.key}
          className={classNames(
            'popup',
            `popup--${popup.kind}`,
            popup.element && `popup--${popup.element}`,
          )}
          data-testid="popup"
        >
          {popup.amount === undefined ? t(`ui-popup-${popup.kind}`) : popup.amount}
        </span>
      ))}
    </button>
  )
}
