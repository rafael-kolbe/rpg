import type { ReactNode } from 'react'

import { classNames } from '../utils/class-names.ts'

interface Props {
  children: ReactNode
  /** Not needed by a `submit` button: its form handles the press. */
  onClick?: () => void
  /** Submits the form it is in. */
  submit?: boolean
  disabled?: boolean
  /** Makes it a toggle: on (`true`) or off (`false`). */
  pressed?: boolean
  testId?: string
}

/** A plain action button, a form's submit button, or a toggle when `pressed` is given. */
export function Button({
  children,
  onClick,
  submit = false,
  disabled = false,
  pressed,
  testId,
}: Props) {
  return (
    <button
      type={submit ? 'submit' : 'button'}
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
