import { useState } from 'react'

import { recoverAccount } from '../api/accounts.ts'
import type { AccountError } from '../protocol/generated/AccountError.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { TextInput } from '../ui/TextInput.tsx'
import { AccountForm } from './AccountForm.tsx'
import { PasswordField, UsernameField } from './fields.tsx'
import { RecoveryKeyNotice } from './RecoveryKeyNotice.tsx'
import './account.css'

/** `/recover`: a new password with the recovery key, which is then replaced. */
export function RecoverPage() {
  const { t } = useTranslation()
  const [username, setUsername] = useState('')
  const [key, setKey] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [error, setError] = useState<AccountError | 'mismatch' | null>(null)
  const [busy, setBusy] = useState(false)
  const [newKey, setNewKey] = useState<string | null>(null)

  if (newKey !== null)
    return (
      <main className="app">
        <RecoveryKeyNotice recoveryKey={newKey} />
      </main>
    )

  const submit = async () => {
    if (password !== confirm) {
      setError('mismatch')
      return
    }
    setBusy(true)
    const result = await recoverAccount({ username, recoveryKey: key, newPassword: password })
    setBusy(false)
    if (result.ok) setNewKey(result.value.recoveryKey)
    else setError(result.error)
  }

  return (
    <main className="app">
      <AccountForm
        title={t('ui-account-recover')}
        submitLabel={t('ui-account-recover')}
        error={error}
        busy={busy}
        onSubmit={() => void submit()}
        links={<a href="/login">{t('ui-account-log-in')}</a>}
      >
        <UsernameField value={username} onChange={setUsername} />
        <TextInput
          label={t('ui-account-recovery-key')}
          value={key}
          onChange={setKey}
          autoComplete="off"
          testId="account-recovery-key"
        />
        <PasswordField kind="new" value={password} onChange={setPassword} />
        <PasswordField kind="confirm" value={confirm} onChange={setConfirm} />
        <p className="account__hint">{t('ui-account-password-rule')}</p>
      </AccountForm>
    </main>
  )
}
