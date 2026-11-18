import { useEffect, useState } from 'react'

import { fetchSession, logOut } from '../api/accounts.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'

type Session = { state: 'checking' } | { state: 'out' } | { state: 'in'; username: string }

/** Who is logged in, with log out; or links to log in and sign up. */
export function AccountStatus() {
  const { t } = useTranslation()
  const [session, setSession] = useState<Session>({ state: 'checking' })

  useEffect(() => {
    let active = true
    void fetchSession().then((result) => {
      if (!active) return
      setSession(result.ok ? { state: 'in', username: result.value.username } : { state: 'out' })
    })
    return () => {
      active = false
    }
  }, [])

  const leave = async () => {
    await logOut()
    setSession({ state: 'out' })
  }

  switch (session.state) {
    case 'checking':
      return null
    case 'out':
      return (
        <p className="account-status" data-testid="account-status">
          <a href="/login">{t('ui-account-log-in')}</a>
          {' · '}
          <a href="/signup">{t('ui-account-sign-up')}</a>
        </p>
      )
    case 'in':
      return (
        <p className="account-status" data-testid="account-status">
          {t('ui-account-logged-in', { username: session.username })}{' '}
          <Button onClick={() => void leave()} testId="account-log-out">
            {t('ui-account-log-out')}
          </Button>
        </p>
      )
  }
}
