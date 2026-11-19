import { useCallback, useEffect, useRef, useState } from 'react'

import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { ErrorCode } from '../protocol/generated/ErrorCode.ts'
import type { GameError } from '../protocol/generated/GameError.ts'
import type { GameView } from '../protocol/generated/GameView.ts'
import type { RejectReason } from '../protocol/generated/RejectReason.ts'
import type { ServerMessage } from '../protocol/generated/ServerMessage.ts'

export interface Battle {
  /** Sandbox scenario; `null` for a battle of the game. */
  scenario: string | null
  seed: number
  catalog: Catalog
  state: BattleState
}

/** Events of one server update, numbered so effects run once per batch. */
interface EventBatch {
  id: number
  /** Battle tick right after these events. */
  tick: number
  events: BattleEvent[]
}

type Connection = 'connecting' | 'open' | 'closed'

type BatchListener = (batch: EventBatch) => void

export interface BattleSocket {
  connection: Connection
  battle: Battle | null
  rejection: RejectReason | null
  error: ErrorCode | null
  /** The open game, after `start_game`. */
  game: GameView | null
  /** The last refused game command, until the next accepted one. */
  gameError: GameError | null
  send: (message: ClientMessage) => void
  /** Calls `listener` for every update that carries events; returns the unsubscribe. */
  subscribe: (listener: BatchListener) => () => void
}

function socketUrl(): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}/ws`
}

/**
 * Connects to the game WebSocket and sends `start` (then `extra`) once open:
 * a sandbox battle or a game. The server is the source of truth: every
 * update replaces the whole state.
 */
export function useBattle(start: ClientMessage, extra: ClientMessage[] = []): BattleSocket {
  const [connection, setConnection] = useState<Connection>('connecting')
  const [battle, setBattle] = useState<Battle | null>(null)
  const [rejection, setRejection] = useState<RejectReason | null>(null)
  const [error, setError] = useState<ErrorCode | null>(null)
  const [game, setGame] = useState<GameView | null>(null)
  const [gameError, setGameError] = useState<GameError | null>(null)
  const socket = useRef<WebSocket | null>(null)
  const batchId = useRef(0)
  const listeners = useRef(new Set<BatchListener>())
  const startMessage = JSON.stringify(start)
  const extraMessages = JSON.stringify(extra)

  useEffect(() => {
    const ws = new WebSocket(socketUrl())
    socket.current = ws
    ws.addEventListener('open', () => {
      setConnection('open')
      for (const message of JSON.parse(extraMessages) as ClientMessage[]) {
        ws.send(JSON.stringify(message))
      }
      ws.send(startMessage)
    })
    ws.addEventListener('close', () => setConnection('closed'))
    ws.addEventListener('message', (event: MessageEvent<string>) => {
      const message = JSON.parse(event.data) as ServerMessage
      switch (message.type) {
        case 'battle_started':
          setBattle({
            scenario: message.scenario,
            seed: message.seed,
            catalog: message.catalog,
            state: message.state,
          })
          setRejection(null)
          setError(null)
          break
        case 'battle_update':
          setBattle((current) => (current ? { ...current, state: message.state } : current))
          if (message.events.length > 0) {
            batchId.current += 1
            const batch = { id: batchId.current, tick: message.state.tick, events: message.events }
            for (const listener of listeners.current) listener(batch)
          }
          break
        case 'game_update':
          setGame(message.view)
          setGameError(null)
          break
        case 'game_rejected':
          setGameError(message.error)
          break
        case 'command_rejected':
          setRejection(message.reason)
          break
        case 'error':
          setError(message.code)
          break
      }
    })
    return () => {
      socket.current = null
      ws.close()
    }
  }, [startMessage, extraMessages])

  const send = useCallback((message: ClientMessage) => {
    if (message.type === 'command') setRejection(null)
    const ws = socket.current
    if (ws?.readyState === WebSocket.OPEN) ws.send(JSON.stringify(message))
  }, [])

  const subscribe = useCallback((listener: BatchListener) => {
    const all = listeners.current
    all.add(listener)
    return () => {
      all.delete(listener)
    }
  }, [])

  return { connection, battle, rejection, error, game, gameError, send, subscribe }
}
