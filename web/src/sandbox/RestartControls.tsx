import { useState } from 'react'

import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Checkbox } from '../ui/Checkbox.tsx'
import { TextInput } from '../ui/TextInput.tsx'

interface Props {
  seed: number
  /** Whether the current battle is an auto-battle (the checkbox starts there). */
  auto: boolean
  onRestart: (seed: number | null, auto: boolean) => void
}

/** Restart with a chosen seed (empty = random), optionally in auto-battle. */
export function RestartControls({ seed, auto: initialAuto, onRestart }: Props) {
  const { t } = useTranslation()
  const [nextSeed, setNextSeed] = useState(String(seed))
  const [auto, setAuto] = useState(initialAuto)
  const parsed = nextSeed.trim() === '' ? null : Number(nextSeed)
  return (
    <div className="debug-panel__row">
      <TextInput label={t('ui-debug-seed')} value={nextSeed} onChange={setNextSeed} size={10} />
      <Checkbox label={t('ui-debug-auto')} checked={auto} onChange={setAuto} testId="auto" />
      <Button onClick={() => onRestart(parsed, auto)} testId="restart">
        {t('ui-debug-restart')}
      </Button>
    </div>
  )
}
