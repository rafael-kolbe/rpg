import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Gauge } from '../ui/Gauge.tsx'
import { Window } from '../ui/Window.tsx'
import { classNames } from '../utils/class-names.ts'
import { Vitals } from './Vitals.tsx'

interface Props {
  party: CombatantState[]
  active: CombatantId | null
  onSelect: (id: CombatantId) => void
}

/** HP, MP and ATB of each party member (bottom right). */
export function PartyPanel({ party, active, onSelect }: Props) {
  return (
    <Window className="party-panel">
      {party.map((member) => (
        <PartyRow
          key={member.id}
          member={member}
          active={member.id === active}
          onSelect={() => onSelect(member.id)}
        />
      ))}
    </Window>
  )
}

interface RowProps {
  member: CombatantState
  active: boolean
  onSelect: () => void
}

function PartyRow({ member, active, onSelect }: RowProps) {
  const { t } = useTranslation()
  const ready = member.playerControlled && member.turn === 'awaiting_command'
  return (
    <button
      type="button"
      className={classNames('party-row', active && 'party-row--active')}
      disabled={!ready}
      onClick={onSelect}
    >
      <span className="party-row__name">{t(member.name)}</span>
      <Vitals combatant={member} />
      <span className="party-row__gauges">
        <Gauge value={member.atb} max={member.atbFull} variant="atb" highlight={ready} />
      </span>
    </button>
  )
}
