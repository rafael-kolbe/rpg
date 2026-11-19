import type { GameMode } from '../protocol/generated/GameMode.ts'
import type { GameSummary } from '../protocol/generated/GameSummary.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { MenuList } from '../ui/MenuList.tsx'
import { Window } from '../ui/Window.tsx'

interface Props {
  summary: GameSummary
  onChoose: (mode: GameMode) => void
}

/** Campaign or Multiplayer; the Multiplayer stays locked until the story opens it. */
export function ModeSelect({ summary, onChoose }: Props) {
  const { t } = useTranslation()
  return (
    <Window className="mode-select" testId="mode-select">
      <h1>{t('ui-game-title')}</h1>
      <MenuList
        keyboard
        entries={[
          {
            key: 'campaign',
            label: t('ui-mode-campaign'),
            detail: t(summary.campaignStarted ? 'ui-mode-continue' : 'ui-mode-new'),
            onSelect: () => onChoose('campaign'),
          },
          {
            key: 'multiplayer',
            label: t('ui-mode-multiplayer'),
            detail: summary.multiplayerUnlocked ? null : t('ui-mode-locked'),
            disabled: !summary.multiplayerUnlocked,
            onSelect: () => onChoose('multiplayer'),
          },
        ]}
      />
      <p className="mode-select__hint">
        {summary.multiplayerUnlocked ? null : t('ui-mode-locked-hint')}
      </p>
    </Window>
  )
}
