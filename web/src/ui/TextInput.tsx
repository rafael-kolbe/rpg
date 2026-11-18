import type { ReactNode } from 'react'

interface Props {
  label: ReactNode
  value: string
  onChange: (value: string) => void
  size?: number
  /** `password` hides what is typed. */
  type?: 'text' | 'password'
  /** Browser autofill hint, e.g. `username`, `current-password`. */
  autoComplete?: string
  maxLength?: number
  testId?: string
}

/** A labelled single-line text field. */
export function TextInput({
  label,
  value,
  onChange,
  size,
  type = 'text',
  autoComplete,
  maxLength,
  testId,
}: Props) {
  return (
    <label className="field">
      <span className="field__label">{label}</span>
      <input
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        size={size}
        autoComplete={autoComplete}
        maxLength={maxLength}
        data-testid={testId}
      />
    </label>
  )
}
