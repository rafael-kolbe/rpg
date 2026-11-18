import type { ReactNode } from 'react'

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
}

/** A vertical list of selectable menu entries. */
export function MenuList({ entries }: Props) {
  return (
    <ul className="menu-list">
      {entries.map((entry) => (
        <li key={entry.key}>
          <button
            type="button"
            className="menu-list__entry"
            disabled={entry.disabled}
            onClick={entry.onSelect}
          >
            <span>{entry.label}</span>
            {entry.detail !== undefined && <span>{entry.detail}</span>}
          </button>
        </li>
      ))}
    </ul>
  )
}
