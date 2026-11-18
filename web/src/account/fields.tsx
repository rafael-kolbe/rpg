import { useTranslation } from '../i18n/use-translation.ts'
import { TextInput } from '../ui/TextInput.tsx'

/** Longest username the server accepts. */
const USERNAME_MAX = 20

/** Longest password the server accepts. */
const PASSWORD_MAX = 64

interface FieldProps {
  value: string
  onChange: (value: string) => void
}

/** Username field with its rule as the label hint. */
export function UsernameField({ value, onChange }: FieldProps) {
  const { t } = useTranslation()
  return (
    <TextInput
      label={t('ui-account-username')}
      value={value}
      onChange={onChange}
      autoComplete="username"
      maxLength={USERNAME_MAX}
      testId="account-username"
    />
  )
}

interface PasswordProps extends FieldProps {
  /** `new` for a password being chosen (sign-up, recovery). */
  kind: 'current' | 'new' | 'confirm'
}

/** Password field: current, new, or the repeated new one. */
export function PasswordField({ value, onChange, kind }: PasswordProps) {
  const { t } = useTranslation()
  return (
    <TextInput
      label={t(`ui-account-password-${kind}`)}
      type="password"
      value={value}
      onChange={onChange}
      autoComplete={kind === 'current' ? 'current-password' : 'new-password'}
      maxLength={PASSWORD_MAX}
      testId={`account-password-${kind}`}
    />
  )
}
