import type { NodeView } from '../protocol/generated/NodeView.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Window } from '../ui/Window.tsx'
import { nodeAction } from './node-action.ts'

interface Props {
  node: NodeView | null
  onInteract: (ambush: boolean) => void
}

/** What the current node offers, as its action button(s), below the map. */
export function NodePanel({ node, onInteract }: Props) {
  const { t } = useTranslation()
  const action = nodeAction(node)
  const args = Object.fromEntries(
    Object.entries(action?.args ?? {}).map(([name, id]) => [name, t(id)]),
  )
  return (
    <Window className="node-panel" testId="node-panel">
      <p className="node-panel__hint">{t('ui-map-keys')}</p>
      {action !== null && (
        <div className="node-panel__actions">
          <Button onClick={() => onInteract(false)} testId="interact">
            {t(action.message, args)}
          </Button>
          {action.ambush && (
            <Button onClick={() => onInteract(true)} testId="ambush">
              {t('ui-node-ambush')}
            </Button>
          )}
        </div>
      )}
    </Window>
  )
}
