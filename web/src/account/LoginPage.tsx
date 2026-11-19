import { useState } from 'react'

import { logIn } from '../api/accounts.ts'
import type { AccountError } from '../protocol/generated/AccountError.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { AccountForm } from './AccountForm.tsx'
import { PasswordField, UsernameField } from './fields.tsx'
import './account.css'

/** `/login`. */
export function LoginPage() {
  const { t } = useTranslation()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState<AccountError | null>(null)
  const [busy, setBusy] = useState(false)

  const submit = async () => {
    setBusy(true)
    const result = await logIn({ username, password })
    setBusy(false)
    if (result.ok) window.location.assign('/play')
    else setError(result.error)
  }

  return (
    <main className="app">
      <AccountForm
        title={t('ui-account-log-in')}
        submitLabel={t('ui-account-log-in')}
        error={error}
        busy={busy}
        onSubmit={() => void submit()}
        links={
          <>
            <a href="/signup">{t('ui-account-sign-up')}</a>
            {' · '}
            <a href="/recover">{t('ui-account-forgot')}</a>
          </>
        }
      >
        <UsernameField value={username} onChange={setUsername} />
        <PasswordField kind="current" value={password} onChange={setPassword} />
      </AccountForm>
    </main>
  )
}
