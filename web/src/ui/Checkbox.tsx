import type { ReactNode } from 'react'

interface Props {
  label: ReactNode
  checked: boolean
  onChange: (checked: boolean) => void
  testId?: string
}

/** A labelled checkbox. */
export function Checkbox({ label, checked, onChange, testId }: Props) {
  return (
    <label className="field">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        data-testid={testId}
      />
      <span className="field__label">{label}</span>
    </label>
  )
}
