import type { ReactNode } from 'react'

import { classNames } from '../utils/class-names.ts'

interface Props {
  children: ReactNode
  onClick: () => void
  disabled?: boolean
  /** Makes it a toggle: on (`true`) or off (`false`). */
  pressed?: boolean
  testId?: string
}

/** A plain action button, or a toggle when `pressed` is given. */
export function Button({ children, onClick, disabled = false, pressed, testId }: Props) {
  return (
    <button
      type="button"
      className={classNames('button', pressed && 'button--pressed')}
      onClick={onClick}
      disabled={disabled}
      aria-pressed={pressed}
      data-testid={testId}
    >
      {children}
    </button>
  )
}
