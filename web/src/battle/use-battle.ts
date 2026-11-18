import { useCallback, useEffect, useRef, useState } from 'react'

import type { BattleEvent } from '../protocol/generated/BattleEvent.ts'
import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { ErrorCode } from '../protocol/generated/ErrorCode.ts'
import type { RejectReason } from '../protocol/generated/RejectReason.ts'
import type { ServerMessage } from '../protocol/generated/ServerMessage.ts'

export interface Battle {
  scenario: string
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
  send: (message: ClientMessage) => void
  /** Calls `listener` for every update that carries events; returns the unsubscribe. */
  subscribe: (listener: BatchListener) => () => void
}

function socketUrl(): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}/ws`
}

/**
 * Connects to the battle WebSocket and sends `start` once open. The server
 * is the source of truth: every update replaces the whole state.
 */
export function useBattle(start: ClientMessage): BattleSocket {
  const [connection, setConnection] = useState<Connection>('connecting')
  const [battle, setBattle] = useState<Battle | null>(null)
  const [rejection, setRejection] = useState<RejectReason | null>(null)
  const [error, setError] = useState<ErrorCode | null>(null)
  const socket = useRef<WebSocket | null>(null)
  const batchId = useRef(0)
  const listeners = useRef(new Set<BatchListener>())
  const startMessage = JSON.stringify(start)

  useEffect(() => {
    const ws = new WebSocket(socketUrl())
    socket.current = ws
    ws.addEventListener('open', () => {
      setConnection('open')
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
  }, [startMessage])

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

  return { connection, battle, rejection, error, send, subscribe }
}
