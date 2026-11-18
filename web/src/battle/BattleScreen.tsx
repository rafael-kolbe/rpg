import { useEffect, useState } from 'react'

import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'
import type { RejectReason } from '../protocol/generated/RejectReason.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { CombatantView } from './CombatantView.tsx'
import { CommandMenu, type MenuLevel, type PendingCommand } from './CommandMenu.tsx'
import { PartyPanel } from './PartyPanel.tsx'
import { effectsOf, type Popup, type Strike } from './popups.ts'
import { canTarget, needsTarget } from './targeting.ts'
import type { Battle, BattleSocket } from './use-battle.ts'
import './battle.css'

/** How long a floating number or strike stays on screen. */
const EFFECT_MS = 1200

interface Props {
  battle: Battle
  rejection: RejectReason | null
  send: (message: ClientMessage) => void
  subscribe: BattleSocket['subscribe']
  onRestart: () => void
}

/** The battle screen: enemies left, party right, menus at the bottom. */
export function BattleScreen({ battle, rejection, send, subscribe, onRestart }: Props) {
  const { t } = useTranslation()
  const { state, catalog } = battle
  const [selected, setSelected] = useState<CombatantId | null>(null)
  const [menu, setMenu] = useState<{ actor: CombatantId; level: MenuLevel } | null>(null)
  const [pending, setPending] = useState<(PendingCommand & { actor: CombatantId }) | null>(null)
  const [popups, setPopups] = useState<Popup[]>([])
  const [strikes, setStrikes] = useState<Strike[]>([])

  const party = state.combatants.filter((c) => c.side === 'party')
  const enemies = state.combatants.filter((c) => c.side === 'enemies')
  const ready = party.filter((c) => c.playerControlled && c.turn === 'awaiting_command')
  const actor =
    state.outcome === null ? (ready.find((c) => c.id === selected) ?? ready[0] ?? null) : null

  // Menu state belongs to one actor; a stale actor means a fresh root menu.
  const level = menu && actor && menu.actor === actor.id ? menu.level : 'root'
  const current = pending && actor && pending.actor === actor.id ? pending : null
  const held = actor !== null && (level !== 'root' || current !== null)

  useEffect(() => {
    send({ type: 'hold', held })
  }, [held, send])

  useEffect(() => {
    const timers = new Set<number>()
    const unsubscribe = subscribe((batch) => {
      const effects = effectsOf(batch.id, batch.events, catalog)
      if (effects.popups.length === 0 && effects.strikes.length === 0) return
      const popupKeys = new Set(effects.popups.map((p) => p.key))
      const strikeKeys = new Set(effects.strikes.map((s) => s.key))
      setPopups((existing) => [...existing, ...effects.popups])
      setStrikes((existing) => [...existing, ...effects.strikes])
      const timer = window.setTimeout(() => {
        timers.delete(timer)
        setPopups((existing) => existing.filter((p) => !popupKeys.has(p.key)))
        setStrikes((existing) => existing.filter((s) => !strikeKeys.has(s.key)))
      }, EFFECT_MS)
      timers.add(timer)
    })
    return () => {
      unsubscribe()
      for (const timer of timers) window.clearTimeout(timer)
    }
  }, [subscribe, catalog])

  const command = (kind: CommandKind, target: CombatantId | null) => {
    if (actor === null) return
    send({ type: 'command', command: { actor: actor.id, kind, target } })
    setMenu(null)
    setPending(null)
  }

  const choose = (kind: CommandKind, action: ActionInfo | null) => {
    if (actor === null) return
    if (action !== null && needsTarget(action)) {
      setPending({ actor: actor.id, kind, action })
    } else {
      command(kind, null)
    }
  }

  const view = (combatant: (typeof state.combatants)[number]) => (
    <CombatantView
      key={combatant.id}
      combatant={combatant}
      popups={popups.filter((p) => p.target === combatant.id)}
      strikes={strikes.filter((s) => s.target === combatant.id)}
      targetable={current !== null && actor !== null && canTarget(current.action, actor, combatant)}
      onTarget={() => current && command(current.kind, combatant.id)}
    />
  )

  return (
    <div className="battle">
      <div className="battle__field">
        <div className="battle__side battle__side--enemies">{enemies.map(view)}</div>
        <div className="battle__side battle__side--party">{party.map(view)}</div>
        {state.outcome !== null && (
          <div className="battle__outcome" data-testid="outcome">
            <p>{t(`ui-outcome-${state.outcome}`)}</p>
            <Button onClick={onRestart}>{t('ui-outcome-restart')}</Button>
          </div>
        )}
      </div>
      {rejection !== null && <p className="battle__notice">{t(`ui-reject-${rejection}`)}</p>}
      <div className="battle__hud">
        <CommandMenu
          actor={actor}
          state={state}
          catalog={catalog}
          level={level}
          pending={current}
          onLevel={(next) => actor && setMenu({ actor: actor.id, level: next })}
          onChoose={choose}
          onCancel={() => setPending(null)}
        />
        <PartyPanel party={party} active={actor?.id ?? null} onSelect={setSelected} />
      </div>
    </div>
  )
}
