import type { ActionId } from '../protocol/generated/ActionId.ts'
import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { MenuList, type MenuEntry } from '../ui/MenuList.tsx'
import { Window } from '../ui/Window.tsx'
import { rememberedEntry, type RememberedCommand } from './action-memory.ts'
import { usableAction } from './commands.ts'

/** Where the player is in the command menu. */
export type MenuLevel = 'root' | 'magic' | 'technic' | 'item'

/** A command waiting for its target. */
export interface PendingCommand {
  kind: CommandKind
  action: ActionInfo
}

interface Props {
  actor: CombatantState | null
  state: BattleState
  catalog: Catalog
  level: MenuLevel
  pending: PendingCommand | null
  /** Action memory: where the cursor starts. */
  remembered: RememberedCommand | null
  onLevel: (level: MenuLevel) => void
  onChoose: (kind: CommandKind, action: ActionInfo | null) => void
  onCancel: () => void
}

/**
 * The command window (bottom left): Attack, Magic, Technic, Item. Defend has
 * no entry: it is the ← key on the root menu.
 * The keyboard drives it except while a target is being chosen.
 */
export function CommandMenu(props: Props) {
  const { t } = useTranslation()
  const { actor, pending, level, remembered, catalog, onCancel, onLevel } = props

  if (actor === null) {
    return <Window className="command-menu--idle">{t('ui-command-waiting')}</Window>
  }

  const back: MenuEntry = {
    key: 'back',
    label: t('ui-command-back'),
    onSelect: () => onLevel('root'),
  }
  let entries: MenuEntry[]
  if (pending !== null) {
    entries = [{ ...back, onSelect: onCancel }]
  } else if (level === 'magic' || level === 'technic') {
    entries = [...actionEntries(props, actor, level, t), back]
  } else if (level === 'item') {
    entries = [...itemEntries(props, actor, t), back]
  } else {
    entries = rootEntries(props, actor, t)
  }

  return (
    <Window testId="command-menu">
      {pending !== null && <p>{t('ui-command-choose-target')}</p>}
      <MenuList
        key={`${actor.id}-${level}-${pending === null ? 'menu' : 'target'}`}
        entries={entries}
        keyboard={pending === null}
        onCancel={level === 'root' ? undefined : () => onLevel('root')}
        initialKey={rememberedEntry(remembered, level, catalog)}
      />
    </Window>
  )
}

type Translate = ReturnType<typeof useTranslation>['t']

function rootEntries(props: Props, actor: CombatantState, t: Translate): MenuEntry[] {
  const { catalog, onChoose, onLevel } = props
  const attack = catalog.actions[actor.attack]
  const knows = (category: ActionInfo['category']) =>
    actor.actions.some((id) => catalog.actions[id]?.category === category)

  const entries: MenuEntry[] = [
    {
      key: 'attack',
      label: t('ui-command-attack'),
      disabled: !attack,
      onSelect: () => attack && onChoose({ type: 'attack' }, attack),
    },
  ]
  if (knows('magic')) {
    entries.push({ key: 'magic', label: t('ui-command-magic'), onSelect: () => onLevel('magic') })
  }
  if (knows('technic')) {
    entries.push({
      key: 'technic',
      label: t('ui-command-technic'),
      onSelect: () => onLevel('technic'),
    })
  }
  entries.push({ key: 'item', label: t('ui-command-item'), onSelect: () => onLevel('item') })
  return entries
}

function actionEntries(
  { state, catalog, onChoose }: Props,
  actor: CombatantState,
  category: 'magic' | 'technic',
  t: Translate,
): MenuEntry[] {
  return actor.actions.flatMap((id: ActionId) => {
    const action = catalog.actions[id]
    if (action?.category !== category) return []
    const kind: CommandKind = { type: 'action', id }
    return [
      {
        key: id,
        label: t(`action-${id}`),
        detail: action.mpCost > 0 ? t('ui-mp-cost', { cost: action.mpCost }) : undefined,
        disabled: usableAction(kind, actor, state, catalog) === null,
        onSelect: () => onChoose(kind, action),
      },
    ]
  })
}

function itemEntries(
  { state, catalog, onChoose }: Props,
  actor: CombatantState,
  t: Translate,
): MenuEntry[] {
  return state.inventory.map(({ item, count }) => {
    const kind: CommandKind = { type: 'item', id: item }
    const action = usableAction(kind, actor, state, catalog)
    return {
      key: item,
      label: t(`item-${item}`),
      detail: t('ui-item-count', { count }),
      disabled: action === null,
      onSelect: () => action && onChoose(kind, action),
    }
  })
}
