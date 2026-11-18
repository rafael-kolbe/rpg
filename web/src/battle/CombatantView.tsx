import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { classNames } from '../utils/class-names.ts'
import { initialsOf } from '../utils/initials.ts'
import type { Popup, Strike } from './popups.ts'

interface Props {
  combatant: CombatantState
  popups: Popup[]
  strikes: Strike[]
  targetable: boolean
  onTarget: () => void
}

/** Avatar placeholder, name, statuses and floating numbers of one combatant. */
export function CombatantView({ combatant, popups, strikes, targetable, onTarget }: Props) {
  const { t } = useTranslation()
  const name = t(combatant.name)
  const distracted = combatant.statuses.includes('distracted')
  const className = classNames(
    'combatant',
    `combatant--${combatant.side}`,
    combatant.hp === 0 && 'combatant--knocked-out',
    distracted && 'combatant--distracted',
    targetable && 'combatant--targetable',
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
      <span className="combatant__avatar" aria-hidden="true">
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
      <span className="combatant__statuses">
        {combatant.statuses.map((status) => (
          <span key={status} className={`status status--${status}`}>
            {t(`ui-status-${status}`)}
          </span>
        ))}
      </span>
      {popups.map((popup) => (
        <span key={popup.key} className={`popup popup--${popup.kind}`} data-testid="popup">
          {popup.amount === undefined ? t(`ui-popup-${popup.kind}`) : popup.amount}
        </span>
      ))}
    </button>
  )
}
