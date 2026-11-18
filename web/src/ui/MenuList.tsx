import { useEffect, useRef, useState, type ReactNode } from 'react'

import { classNames } from '../utils/class-names.ts'
import { wrapIndex } from '../utils/wrap-index.ts'
import { useMenuKeys, type MenuKey } from './use-menu-keys.ts'

export interface MenuEntry {
  key: string
  label: ReactNode
  /** Right-aligned extra (MP cost, item count). */
  detail?: ReactNode
  disabled?: boolean
  onSelect: () => void
}

interface Props {
  entries: MenuEntry[]
  /** Whether the keyboard drives this list (only one list at a time). */
  keyboard?: boolean
  /** Escape/X: leave the list. */
  onCancel?: () => void
  /** Entry the cursor starts on, when it exists and is enabled. */
  initialKey?: string
}

/**
 * A vertical list of selectable menu entries with a cursor. The cursor starts
 * on `initialKey` or the first enabled entry; remount the list (`key`) to
 * reset it.
 */
export function MenuList({ entries, keyboard = false, onCancel, initialKey }: Props) {
  const [cursor, setCursor] = useState(() => {
    const initial = entries.findIndex((e) => e.key === initialKey && !e.disabled)
    return initial >= 0 ? initial : nextEnabled(entries, -1, 1)
  })
  const current = Math.min(cursor, entries.length - 1)
  const cursorEntry = useRef<HTMLButtonElement>(null)

  const cursorKey = entries[current]?.key

  // Keeps the keyboard cursor visible when the list scrolls inside its
  // window; only when the cursor moves, so the mouse wheel stays free.
  useEffect(() => {
    if (keyboard && cursorKey !== undefined) {
      cursorEntry.current?.scrollIntoView({ block: 'nearest' })
    }
  }, [keyboard, cursorKey])

  useMenuKeys((key: MenuKey) => {
    switch (key) {
      case 'up':
        setCursor(nextEnabled(entries, current, -1))
        return true
      case 'down':
        setCursor(nextEnabled(entries, current, 1))
        return true
      case 'confirm': {
        const entry = entries[current]
        if (entry && !entry.disabled) entry.onSelect()
        return true
      }
      case 'cancel':
        if (!onCancel) return false
        onCancel()
        return true
      default:
        return false
    }
  }, keyboard)

  return (
    <ul className="menu-list">
      {entries.map((entry, index) => (
        <li key={entry.key}>
          <button
            type="button"
            className={classNames(
              'menu-list__entry',
              keyboard && index === current && 'menu-list__entry--cursor',
            )}
            ref={index === current ? cursorEntry : undefined}
            disabled={entry.disabled}
            onClick={entry.onSelect}
            onMouseEnter={() => setCursor(index)}
          >
            <span>{entry.label}</span>
            {entry.detail !== undefined && <span>{entry.detail}</span>}
          </button>
        </li>
      ))}
    </ul>
  )
}

/** Index of the next enabled entry from `from` in `step` direction, wrapping; `from` if none. */
function nextEnabled(entries: MenuEntry[], from: number, step: 1 | -1): number {
  for (let offset = 1; offset <= entries.length; offset++) {
    const index = wrapIndex(from + step * offset, entries.length)
    if (!entries[index]?.disabled) return index
  }
  return Math.max(from, 0)
}
