import type { ReactNode } from 'react'

interface SelectOption<T extends string | number> {
  value: T
  label: ReactNode
}

interface Props<T extends string | number> {
  label: ReactNode
  value: T
  options: SelectOption<T>[]
  onChange: (value: T) => void
  testId?: string
}

/** A labelled drop-down. `onChange` receives the typed option value. */
export function Select<T extends string | number>({
  label,
  value,
  options,
  onChange,
  testId,
}: Props<T>) {
  return (
    <label className="field">
      <span className="field__label">{label}</span>
      <select
        value={value}
        onChange={(event) => {
          const chosen = options[event.target.selectedIndex]
          if (chosen) onChange(chosen.value)
        }}
        data-testid={testId}
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  )
}
