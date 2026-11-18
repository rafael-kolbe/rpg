import type { ReactNode } from 'react'

interface Props {
  children: ReactNode
  onClick: () => void
  disabled?: boolean
  testId?: string
}

/** A plain action button. */
export function Button({ children, onClick, disabled = false, testId }: Props) {
  return (
    <button
      type="button"
      className="button"
      onClick={onClick}
      disabled={disabled}
      data-testid={testId}
    >
      {children}
    </button>
  )
}
