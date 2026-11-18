import { useState } from 'react'

import { readStorage, writeStorage } from '../utils/storage.ts'

/**
 * An on/off preference kept in this browser (`localStorage`), starting at
 * `fallback` until the player changes it. Returns the value and its toggle.
 */
export function useStoredToggle(key: string, fallback: boolean): [boolean, () => void] {
  const [on, setOn] = useState(() => {
    const stored = readStorage(key)
    return stored === null ? fallback : stored === 'on'
  })
  const toggle = () => {
    writeStorage(key, on ? 'off' : 'on')
    setOn(!on)
  }
  return [on, toggle]
}
