import { useState } from 'react'

import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { StatusKind } from '../protocol/generated/StatusKind.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Select } from '../ui/Select.tsx'

const STATUSES: StatusKind[] = [
  'poison',
  'regen',
  'sleep',
  'silence',
  'haste',
  'slow',
  'barrier',
  'magic_barrier',
  'distracted',
]

interface Props {
  combatants: CombatantState[]
  name: (id: CombatantId) => string
  send: (message: ClientMessage) => void
}

/** Exact HP/ATB of every combatant, and buttons to change them. */
export function CombatantTools({ combatants, name, send }: Props) {
  const { t } = useTranslation()
  const [status, setStatus] = useState<StatusKind>('poison')
  const debug = (action: Extract<ClientMessage, { type: 'debug' }>['action']) =>
    send({ type: 'debug', action })

  return (
    <>
      <table className="debug-panel__combatants">
        <tbody>
          {combatants.map((c) => (
            <tr key={c.id}>
              <th scope="row">{name(c.id)}</th>
              <td>
                {c.hp}/{c.maxHp}
              </td>
              <td>{(c.atb / 100).toFixed(1)}%</td>
              <td>
                <Button onClick={() => debug({ type: 'fill_gauge', target: c.id })}>
                  {t('ui-debug-fill')}
                </Button>
                <Button onClick={() => debug({ type: 'knock_out', target: c.id })}>
                  {t('ui-debug-knock-out')}
                </Button>
                <Button onClick={() => debug({ type: 'apply_status', target: c.id, status })}>
                  {t('ui-debug-apply-status')}
                </Button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <Select
        label={t('ui-debug-status')}
        value={status}
        options={STATUSES.map((kind) => ({ value: kind, label: t(`ui-status-${kind}`) }))}
        onChange={setStatus}
      />
    </>
  )
}
