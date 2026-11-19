import type { QuestView } from '../protocol/generated/QuestView.ts'
import { useTranslation } from '../i18n/use-translation.ts'

interface Props {
  objective: string | null
  quests: QuestView[]
}

/** The campaign's next step on top, the missions under way below. */
export function QuestTracker({ objective, quests }: Props) {
  const { t } = useTranslation()
  if (objective === null && quests.length === 0) return null
  return (
    <aside className="quest-tracker" data-testid="quest-tracker">
      {objective !== null && (
        <p className="quest-tracker__objective" data-testid="objective">
          {t(objective)}
        </p>
      )}
      {quests.length > 0 && (
        <ul className="quest-tracker__quests">
          {quests.map((quest) => (
            <li key={quest.quest} data-testid={`quest-${quest.quest}`}>
              <span>{t(`quest-${quest.quest}`)}</span>
              <span className="quest-tracker__progress">
                {quest.progress}/{quest.count}
              </span>
            </li>
          ))}
        </ul>
      )}
    </aside>
  )
}
