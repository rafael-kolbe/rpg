import type { DialogueId } from '../protocol/generated/DialogueId.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { MenuList } from '../ui/MenuList.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { initialsOf } from '../utils/initials.ts'

interface Props {
  dialogue: DialogueId
  line: number
  speaker: string | null
  choices: number
  onNext: () => void
  onChoose: (index: number) => void
}

/**
 * A dialogue line, visual-novel style: the speaker's avatar in the corner,
 * the line below. Confirm or click for the next line; answers show as a
 * list after the last one.
 */
export function DialogueBox({ dialogue, line, speaker, choices, onNext, onChoose }: Props) {
  const { t } = useTranslation()
  useMenuKeys((key) => {
    if (key !== 'confirm') return false
    onNext()
    return true
  }, choices === 0)
  const name = speaker === null ? null : t(speaker)

  return (
    <div className="dialogue" data-testid="dialogue">
      {name !== null && (
        <span className="dialogue__avatar" aria-hidden="true">
          {initialsOf(name)}
        </span>
      )}
      <div
        className="dialogue__box"
        onClick={choices === 0 ? onNext : undefined}
        role={choices === 0 ? 'button' : undefined}
      >
        {name !== null && <p className="dialogue__speaker">{name}</p>}
        <p
          className={name === null ? 'dialogue__text dialogue__text--narration' : 'dialogue__text'}
        >
          {t(`dialogue-${dialogue}-${line + 1}`)}
        </p>
        {choices > 0 ? (
          <MenuList
            key={`${dialogue}-${line}`}
            keyboard
            entries={Array.from({ length: choices }, (_, index) => ({
              key: String(index),
              label: t(`dialogue-${dialogue}-choice-${index + 1}`),
              onSelect: () => onChoose(index),
            }))}
          />
        ) : (
          <span className="dialogue__next" aria-hidden="true">
            ▼
          </span>
        )}
      </div>
    </div>
  )
}
