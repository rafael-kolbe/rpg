import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { ClientMessage } from '../protocol/generated/ClientMessage.ts'
import type { TimeMode } from '../protocol/generated/TimeMode.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Select } from '../ui/Select.tsx'

const SPEEDS = [50, 100, 200, 400, 800]

interface Props {
  state: BattleState
  send: (message: ClientMessage) => void
}

/** Pause, step, speed and Active/Wait. */
export function TimeControls({ state, send }: Props) {
  const { t } = useTranslation()
  const modes: TimeMode[] = ['active', 'wait']
  return (
    <div className="debug-panel__row">
      <Button
        onClick={() => send({ type: 'debug', action: { type: state.paused ? 'resume' : 'pause' } })}
      >
        {state.paused ? t('ui-debug-resume') : t('ui-debug-pause')}
      </Button>
      <Button
        disabled={!state.paused}
        onClick={() => send({ type: 'debug', action: { type: 'step' } })}
      >
        {t('ui-debug-step')}
      </Button>
      <Select
        label={t('ui-debug-speed')}
        value={state.speedPercent}
        options={SPEEDS.map((speed) => ({ value: speed, label: `${speed / 100}×` }))}
        onChange={(percent) => send({ type: 'set_speed', percent })}
        testId="speed"
      />
      <Select
        label={t('ui-debug-time-mode')}
        value={state.timeMode}
        options={modes.map((mode) => ({ value: mode, label: t(`ui-time-${mode}`) }))}
        onChange={(mode) => send({ type: 'set_time_mode', mode })}
      />
    </div>
  )
}
