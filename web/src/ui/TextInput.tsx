import type { ReactNode } from 'react'

interface Props {
  label: ReactNode
  value: string
  onChange: (value: string) => void
  size?: number
}

/** A labelled single-line text field. */
export function TextInput({ label, value, onChange, size }: Props) {
  return (
    <label className="field">
      <span className="field__label">{label}</span>
      <input value={value} onChange={(event) => onChange(event.target.value)} size={size} />
    </label>
  )
}
