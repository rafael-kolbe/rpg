import { useEffect } from 'react'

/**
 * What a key means to a menu, whatever key sent it. `confirm-held` is a
 * confirm key still down after the first press (keyboard auto-repeat);
 * `memory` switches the action memory.
 */
export type MenuKey =
  'up' | 'down' | 'left' | 'right' | 'confirm' | 'confirm-held' | 'cancel' | 'next' | 'memory'

// Z/X beside Enter/Escape: the classic console layout, reachable with one hand.
const KEYS: Record<string, MenuKey> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
  Enter: 'confirm',
  z: 'confirm',
  Z: 'confirm',
  Escape: 'cancel',
  Backspace: 'cancel',
  x: 'cancel',
  X: 'cancel',
  Tab: 'next',
  m: 'memory',
  M: 'memory',
}

const MOVES = new Set<MenuKey>(['up', 'down', 'left', 'right'])

function isTyping(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLTextAreaElement
  )
}

/**
 * Calls `onKey` for menu keys pressed anywhere on the page while `enabled`,
 * except while typing in a form field or holding a modifier. When `onKey`
 * returns `true` the key is consumed: no scrolling, no focus move and no
 * click on a focused button (which would run the command twice). Movement
 * keys auto-repeat; a held confirm key repeats as `confirm-held`; other keys
 * fire once per press.
 */
export function useMenuKeys(onKey: (key: MenuKey) => boolean, enabled = true): void {
  useEffect(() => {
    if (!enabled) return
    const listener = (event: KeyboardEvent) => {
      if (event.ctrlKey || event.altKey || event.metaKey || isTyping(event.target)) return
      const key = KEYS[event.key]
      if (key === undefined) return
      if (event.repeat && key === 'confirm') {
        onKey('confirm-held')
        event.preventDefault()
        return
      }
      if (event.repeat && !MOVES.has(key)) {
        event.preventDefault()
        return
      }
      if (onKey(key)) event.preventDefault()
    }
    window.addEventListener('keydown', listener)
    return () => window.removeEventListener('keydown', listener)
  }, [onKey, enabled])
}
