import type { FormEvent, ReactNode } from 'react'

import type { AccountError } from '../protocol/generated/AccountError.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Window } from '../ui/Window.tsx'

interface Props {
  title: string
  submitLabel: string
  /** Server error, or `mismatch` when the two passwords differ. */
  error: AccountError | 'mismatch' | null
  busy: boolean
  onSubmit: () => void
  /** The fields. */
  children: ReactNode
  /** Links to the other account pages. */
  links: ReactNode
}

/** The frame shared by the sign-up, login and recovery forms. */
export function AccountForm({ title, submitLabel, error, busy, onSubmit, children, links }: Props) {
  const { t } = useTranslation()
  const submit = (event: FormEvent) => {
    event.preventDefault()
    if (!busy) onSubmit()
  }
  return (
    <Window className="account">
      <form className="account__form" onSubmit={submit}>
        <h1 className="account__title">{title}</h1>
        {children}
        {error !== null && (
          <p className="account__error" role="alert" data-testid="account-error">
            {t(`ui-account-error-${error}`)}
          </p>
        )}
        <Button submit disabled={busy} testId="account-submit">
          {submitLabel}
        </Button>
        <p className="account__links">{links}</p>
      </form>
    </Window>
  )
}
