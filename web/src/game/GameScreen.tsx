import { useMemo, useState } from 'react'

import { BattleScreen } from '../battle/BattleScreen.tsx'
import { useBattle, type Battle } from '../battle/use-battle.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { GameCommand } from '../protocol/generated/GameCommand.ts'
import type { GameMode } from '../protocol/generated/GameMode.ts'
import type { TimeMode } from '../protocol/generated/TimeMode.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Select } from '../ui/Select.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { useStoredChoice } from '../ui/use-stored-choice.ts'
import { DialogueBox } from './DialogueBox.tsx'
import { InstanceMap } from './InstanceMap.tsx'
import { neighborToward } from './map-nav.ts'
import { nodeAction } from './node-action.ts'
import { NodePanel } from './NodePanel.tsx'
import { PartyWindow } from './PartyWindow.tsx'
import { SceneNotice } from './SceneNotice.tsx'
import { ShopWindow } from './ShopWindow.tsx'
import { WorldMapWindow } from './WorldMapWindow.tsx'
import './game.css'

const TIME_MODES: readonly TimeMode[] = ['active', 'wait', 'turn']

type Panel = 'none' | 'party' | 'world'

interface Props {
  mode: GameMode
}

/**
 * The game: the instance map and what happens on it (dialogues, shops,
 * rewards), the party and world map windows, and the battles the game
 * starts.
 */
export function GameScreen({ mode }: Props) {
  const { t } = useTranslation()
  const [timeMode, setTimeMode] = useStoredChoice<TimeMode>('rpg.timeMode', TIME_MODES, 'wait')
  const start = useMemo<ClientMessage>(() => ({ type: 'start_game', mode }), [mode])
  // The stored time mode reaches the server once, before the game starts;
  // later changes are sent as they happen.
  const [initial] = useState<ClientMessage[]>(() => [{ type: 'set_time_mode', mode: timeMode }])
  const socket = useBattle(start, initial)
  const { connection, battle, game, gameError, error, rejection, send, subscribe } = socket
  const [panel, setPanel] = useState<Panel>('none')
  // The battle whose outcome the player closed; a new battle shows again.
  const [closedBattle, setClosedBattle] = useState<Battle | null>(null)

  const act = (command: GameCommand) => send({ type: 'game', command })
  const scene = game?.scene ?? null
  const showBattle =
    battle !== null &&
    battle.scenario === null &&
    battle !== closedBattle &&
    (scene?.type === 'battle' || battle.state.outcome !== null)
  const here = game?.instance.nodes.find((node) => node.id === game.position) ?? null
  const onMap = scene?.type === 'map' && panel === 'none' && !showBattle

  useMenuKeys((key) => {
    if (!game) return false
    switch (key) {
      case 'up':
      case 'down':
      case 'left':
      case 'right': {
        const next = neighborToward(game.instance, game.position, key)
        if (next && next.fog !== 'barrier') act({ type: 'move', node: next.id })
        return true
      }
      case 'confirm':
        if (nodeAction(here) !== null) act({ type: 'interact', ambush: false })
        return true
      case 'cancel':
        setPanel('party')
        return true
      case 'memory':
        setPanel('world')
        return true
      default:
        return false
    }
  }, onMap)

  // Enter closes a finished battle, like its Continue button.
  useMenuKeys(
    (key) => {
      if (key !== 'confirm' || battle === null) return false
      setClosedBattle(battle)
      return true
    },
    showBattle && battle?.state.outcome !== null,
  )

  // A newer tab took the game over: this one has nothing left to show.
  if (game === null || error === 'opened-elsewhere') {
    return (
      <main className="game">
        {error !== null ? (
          <p className="game__error" data-testid="game-closed">
            {t(`ui-error-${error}`)}
          </p>
        ) : (
          <p>{t(connection === 'closed' ? 'ui-battle-disconnected' : 'ui-battle-connecting')}</p>
        )}
      </main>
    )
  }

  const changeTimeMode = (next: TimeMode) => {
    setTimeMode(next)
    send({ type: 'set_time_mode', mode: next })
  }

  return (
    <main className="game">
      <header className="game__header">
        <h1 className="game__place" data-testid="place">
          {t(`instance-${game.instance.id}`)}
        </h1>
        <span className="crowns" data-testid="crowns">
          {t('ui-crowns', { amount: game.crowns })}
        </span>
        <Button onClick={() => setPanel('party')} testId="open-party">
          {t('ui-party-title')}
        </Button>
        <Button onClick={() => setPanel('world')} testId="open-world-map">
          {t('ui-world-map')}
        </Button>
        <Select
          label={t('ui-time-mode')}
          value={timeMode}
          options={TIME_MODES.map((option) => ({ value: option, label: t(`ui-time-${option}`) }))}
          onChange={changeTimeMode}
          testId="time-mode"
        />
      </header>
      {connection === 'closed' && <p className="game__error">{t('ui-battle-disconnected')}</p>}
      {/* `busy` only means a key arrived before the screen caught up. */}
      {gameError !== null && gameError !== 'busy' && (
        <p className="game__error" data-testid="game-error">
          {t(`ui-game-error-${gameError}`)}
        </p>
      )}

      {showBattle ? (
        <BattleScreen
          battle={battle}
          rejection={rejection}
          send={send}
          subscribe={subscribe}
          outcomeLabel={t('ui-continue')}
          onOutcome={() => setClosedBattle(battle)}
        />
      ) : (
        <div className="game__stage">
          <InstanceMap
            instance={game.instance}
            position={game.position}
            onMove={(node) => act({ type: 'move', node })}
            onInteract={() => act({ type: 'interact', ambush: false })}
          />
          {scene?.type === 'dialogue' ? (
            <DialogueBox
              dialogue={scene.dialogue}
              line={scene.line}
              speaker={scene.speaker}
              choices={scene.choices}
              onNext={() => act({ type: 'next' })}
              onChoose={(index) => act({ type: 'choose', index })}
            />
          ) : (
            <NodePanel node={here} onInteract={(ambush) => act({ type: 'interact', ambush })} />
          )}
          {scene?.type === 'shop' && (
            <ShopWindow
              goods={scene.goods}
              crowns={game.crowns}
              onBuy={(good) => act({ type: 'buy', good, count: 1 })}
              onSell={(good) => act({ type: 'sell', good, count: 1 })}
              onLeave={() => act({ type: 'next' })}
            />
          )}
          {(scene?.type === 'victory' ||
            scene?.type === 'defeat' ||
            scene?.type === 'rested' ||
            scene?.type === 'found') && (
            <SceneNotice scene={scene} onContinue={() => act({ type: 'next' })} />
          )}
          {panel === 'party' && (
            <PartyWindow
              party={game.party}
              bag={game.bag}
              inventory={game.inventory}
              crowns={game.crowns}
              onEquip={(hero, slot, id) => act({ type: 'equip', hero, slot, id })}
              onUse={(item, hero) => act({ type: 'use_item', item, hero })}
              onClose={() => setPanel('none')}
            />
          )}
          {panel === 'world' && (
            <WorldMapWindow regions={game.regions} onClose={() => setPanel('none')} />
          )}
        </div>
      )}
    </main>
  )
}
