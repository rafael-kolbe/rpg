import type { BoardEntry } from '../protocol/generated/BoardEntry.ts'
import type { QuestId } from '../protocol/generated/QuestId.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { Window } from '../ui/Window.tsx'

interface Props {
  entries: BoardEntry[]
  onAccept: (quest: QuestId) => void
  onClose: () => void
}

/** A Guild board: the missions posted, which the player's rank allows. */
export function BoardWindow({ entries, onAccept, onClose }: Props) {
  const { t } = useTranslation()
  useMenuKeys((key) => {
    if (key !== 'cancel') return false
    onClose()
    return true
  })
  return (
    <Window className="game-window board" testId="board">
      <div className="game-window__head">
        <h2>{t('ui-board-title')}</h2>
        <Button onClick={onClose} testId="board-close">
          {t('ui-close')}
        </Button>
      </div>
      {entries.length === 0 && <p className="game-window__empty">{t('ui-board-empty')}</p>}
      <ul className="board__entries">
        {entries.map((entry) => (
          <li key={entry.quest} className="board__entry">
            <span>{t(`quest-${entry.quest}`)}</span>
            <span className="board__rank">{t(`ui-rank-${entry.rank}`)}</span>
            {entry.status === 'open' ? (
              <Button onClick={() => onAccept(entry.quest)} testId={`accept-${entry.quest}`}>
                {t('ui-board-accept')}
              </Button>
            ) : (
              <span className="board__status">{t(`ui-board-${entry.status}`)}</span>
            )}
          </li>
        ))}
      </ul>
    </Window>
  )
}
