import { useState } from 'react'

import { signUp } from '../api/accounts.ts'
import type { AccountError } from '../protocol/generated/AccountError.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { AccountForm } from './AccountForm.tsx'
import { PasswordField, UsernameField } from './fields.tsx'
import { RecoveryKeyNotice } from './RecoveryKeyNotice.tsx'
import './account.css'

/** `/signup`: creates the account, then shows its recovery key once. */
export function SignUpPage() {
  const { t } = useTranslation()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<AccountError | 'mismatch' | null>(null)
  const [busy, setBusy] = useState(false)
  const [recoveryKey, setRecoveryKey] = useState<string | null>(null)

  if (recoveryKey !== null)
    return (
      <main className="app">
        <RecoveryKeyNotice recoveryKey={recoveryKey} />
      </main>
    )

  const submit = async () => {
    if (password !== confirm) {
      setError('mismatch')
      return
    }
    setBusy(true)
    const result = await signUp({ username, password })
    setBusy(false)
    if (result.ok) setRecoveryKey(result.value.recoveryKey)
    else setError(result.error)
  }

  return (
    <main className="app">
      <AccountForm
        title={t('ui-account-sign-up')}
        submitLabel={t('ui-account-sign-up')}
        error={error}
        busy={busy}
        onSubmit={() => void submit()}
        links={<a href="/login">{t('ui-account-have-account')}</a>}
      >
        <UsernameField value={username} onChange={setUsername} />
        <p className="account__hint">{t('ui-account-username-rule')}</p>
        <PasswordField kind="new" value={password} onChange={setPassword} />
        <PasswordField kind="confirm" value={confirm} onChange={setConfirm} />
        <p className="account__hint">{t('ui-account-password-rule')}</p>
      </AccountForm>
    </main>
  )
}
