import { useState } from 'react'

import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Checkbox } from '../ui/Checkbox.tsx'
import { Window } from '../ui/Window.tsx'

interface Props {
  recoveryKey: string
}

/**
 * Shows a new recovery key once, with a copy button and the warning that it
 * is the only way back into the account; the player confirms saving it
 * before going on.
 */
export function RecoveryKeyNotice({ recoveryKey }: Props) {
  const { t } = useTranslation()
  const [copied, setCopied] = useState(false)
  const [saved, setSaved] = useState(false)
  const copy = () => {
    navigator.clipboard.writeText(recoveryKey).then(
      () => setCopied(true),
      () => setCopied(false),
    )
  }
  return (
    <Window className="account">
      <div className="account__form">
        <h1 className="account__title">{t('ui-recovery-key-title')}</h1>
        <p>{t('ui-recovery-key-explain')}</p>
        <code className="account__key" data-testid="recovery-key">
          {recoveryKey}
        </code>
        <Button onClick={copy} testId="recovery-key-copy">
          {t(copied ? 'ui-recovery-key-copied' : 'ui-recovery-key-copy')}
        </Button>
        <p className="account__warning">{t('ui-recovery-key-warning')}</p>
        <Checkbox
          label={t('ui-recovery-key-saved')}
          checked={saved}
          onChange={setSaved}
          testId="recovery-key-saved"
        />
        <Button
          onClick={() => window.location.assign('/play')}
          disabled={!saved}
          testId="recovery-key-continue"
        >
          {t('ui-recovery-key-continue')}
        </Button>
      </div>
    </Window>
  )
}
