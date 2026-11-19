import { useState } from 'react'

import { readStorage, writeStorage } from '../utils/storage.ts'

/**
 * One of `choices`, kept in this browser (`localStorage`), starting at
 * `fallback` until the player picks another. Returns the value and its setter.
 */
export function useStoredChoice<T extends string>(
  key: string,
  choices: readonly T[],
  fallback: T,
): [T, (value: T) => void] {
  const [value, setValue] = useState<T>(() => {
    const stored = readStorage(key)
    return choices.find((choice) => choice === stored) ?? fallback
  })
  const choose = (next: T) => {
    writeStorage(key, next)
    setValue(next)
  }
  return [value, choose]
}
