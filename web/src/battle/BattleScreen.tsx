import { useEffect, useState } from 'react'

import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { CombatantId } from '../protocol/generated/CombatantId.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'
import type { RejectReason } from '../protocol/generated/RejectReason.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { SOUNDS } from '../audio/sounds.ts'
import { play } from '../audio/synth.ts'
import { Button } from '../ui/Button.tsx'
import { useStoredToggle } from '../ui/use-stored-toggle.ts'
import { useMenuKeys, type MenuKey } from '../ui/use-menu-keys.ts'
import { wrapIndex } from '../utils/wrap-index.ts'
import { CombatantView } from './CombatantView.tsx'
import { repeatCommand } from './action-memory.ts'
import { soundsOf } from './battle-sounds.ts'
import { CommandMenu, type MenuLevel, type PendingCommand } from './CommandMenu.tsx'
import { sameCommand } from './commands.ts'
import { PartyPanel } from './PartyPanel.tsx'
import { effectsOf, type Popup, type Strike } from './popups.ts'
import { canTarget, needsTarget } from './targeting.ts'
import { intentMessage, targetLines } from './intents.ts'
import { TargetLines } from './TargetLines.tsx'
import { enqueueTurns, menuActor } from './turn-queue.ts'
import { useActionMemory } from './use-action-memory.ts'
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
  const [pending, setPending] = useState<
    (PendingCommand & { actor: CombatantId; cursor: CombatantId | null }) | null
  >(null)
  // Gauges in the order they filled: the menu serves them as a queue.
  const [readyOrder, setReadyOrder] = useState<CombatantId[]>([])
  const [popups, setPopups] = useState<Popup[]>([])
  const [strikes, setStrikes] = useState<Strike[]>([])
  const memory = useActionMemory()
  const [soundOn, toggleSound] = useStoredToggle('rpg.sound', true)

  const party = state.combatants.filter((c) => c.side === 'party')
  const enemies = state.combatants.filter((c) => c.side === 'enemies')
  const ready = party.filter((c) => c.playerControlled && c.turn === 'awaiting_command')
  const actorId = menuActor(
    ready.map((c) => c.id),
    selected,
    readyOrder,
  )
  const actor = state.outcome === null ? (ready.find((c) => c.id === actorId) ?? null) : null

  // Menu state belongs to one actor; a stale actor means a fresh root menu.
  const level = menu && actor && menu.actor === actor.id ? menu.level : 'root'
  const current = pending && actor && pending.actor === actor.id ? pending : null
  const held = actor !== null && (level !== 'root' || current !== null)
  const remembered = actor !== null ? memory.recall(actor.id) : null

  // The keyboard cursor falls back to the first valid target when its own
  // target stops being valid (knocked out, revived).
  const targets =
    current !== null && actor !== null
      ? state.combatants.filter((c) => canTarget(current.action, actor, c))
      : []
  const cursor = targets.find((c) => c.id === current?.cursor) ?? targets[0] ?? null

  useEffect(() => {
    send({ type: 'hold', held })
  }, [held, send])

  useEffect(() => {
    const timers = new Set<number>()
    const unsubscribe = subscribe((batch) => {
      if (soundOn) for (const name of soundsOf(batch.events, catalog)) play(SOUNDS[name])
      const filled = batch.events.flatMap((e) => (e.type === 'turn_ready' ? [e.combatant] : []))
      if (filled.length > 0) {
        setReadyOrder((order) => enqueueTurns(order, filled))
      }
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
  }, [subscribe, catalog, soundOn])

  const command = (kind: CommandKind, target: CombatantId | null) => {
    if (actor === null) return
    send({ type: 'command', command: { actor: actor.id, kind, target } })
    memory.remember(actor.id, { kind, target })
    setMenu(null)
    setPending(null)
    // A pick lasts one turn; the next turn of the same hero waits in line.
    setSelected(null)
  }

  const choose = (kind: CommandKind, action: ActionInfo | null) => {
    if (actor === null) return
    if (action !== null && needsTarget(action)) {
      // The remembered target first; then healing and buffs start on the
      // actor itself, attacks on the first enemy.
      const self = canTarget(action, actor, actor) ? actor.id : null
      const start =
        remembered !== null && sameCommand(remembered.kind, kind) ? remembered.target : self
      setPending({ actor: actor.id, kind, action, cursor: start })
    } else {
      command(kind, null)
    }
  }

  const onKey = (key: MenuKey): boolean => {
    if (key === 'memory') {
      memory.toggle()
      return true
    }
    if (actor === null) return false
    // Holding confirm sends the hero's last command again, target included.
    if (key === 'confirm-held') {
      const again = remembered && repeatCommand(remembered, actor, state, catalog)
      if (!again) return false
      command(again.kind, again.target)
      return true
    }
    if (key === 'next') {
      const index = ready.findIndex((c) => c.id === actor.id)
      setSelected(ready[wrapIndex(index + 1, ready.length)]?.id ?? null)
      return true
    }
    // Defend lives off the menu: rarely worth a turn, so it gets no entry.
    if (key === 'left' && current === null && level === 'root') {
      command({ type: 'defend' }, null)
      return true
    }
    if (current === null || cursor === null) return false
    const move = (step: number) => {
      const index = targets.findIndex((c) => c.id === cursor.id)
      const next = targets[wrapIndex(index + step, targets.length)]
      if (next) setPending({ ...current, actor: actor.id, cursor: next.id })
    }
    switch (key) {
      case 'up':
      case 'left':
        move(-1)
        return true
      case 'down':
      case 'right':
        move(1)
        return true
      case 'confirm':
        command(current.kind, cursor.id)
        return true
      case 'cancel':
        setPending(null)
        return true
    }
  }
  useMenuKeys(onKey, state.outcome === null)

  const intentText = ({ side, intent }: (typeof state.combatants)[number]): string | null =>
    side === 'enemies' && intent !== null ? t(intentMessage(intent)) : null

  const view = (combatant: (typeof state.combatants)[number]) => (
    <CombatantView
      key={combatant.id}
      combatant={combatant}
      popups={popups.filter((p) => p.target === combatant.id)}
      strikes={strikes.filter((s) => s.target === combatant.id)}
      targetable={targets.includes(combatant)}
      cursor={cursor === combatant}
      intent={intentText(combatant)}
      onTarget={() => current && command(current.kind, combatant.id)}
    />
  )

  return (
    <div className="battle">
      <div className="battle__field">
        <TargetLines lines={targetLines(state.combatants)} />
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
          remembered={remembered}
          onLevel={(next) => actor && setMenu({ actor: actor.id, level: next })}
          onChoose={choose}
          onCancel={() => setPending(null)}
        />
        <PartyPanel party={party} active={actor?.id ?? null} onSelect={setSelected} />
      </div>
      <div className="battle__options">
        <Button onClick={memory.toggle} pressed={memory.enabled} testId="action-memory">
          {t('ui-battle-memory', { state: memory.enabled ? 'on' : 'off' })}
        </Button>
        <Button onClick={toggleSound} pressed={soundOn} testId="sound">
          {t('ui-battle-sound', { state: soundOn ? 'on' : 'off' })}
        </Button>
        <p className="battle__keys">{t('ui-battle-keys')}</p>
      </div>
    </div>
  )
}
