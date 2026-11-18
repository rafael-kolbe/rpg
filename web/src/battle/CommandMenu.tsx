import type { ActionId } from '../protocol/generated/ActionId.ts'
import type { ActionInfo } from '../protocol/generated/ActionInfo.ts'
import type { BattleState } from '../protocol/generated/BattleState.ts'
import type { Catalog } from '../protocol/generated/Catalog.ts'
import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import type { CommandKind } from '../protocol/generated/CommandKind.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { MenuList, type MenuEntry } from '../ui/MenuList.tsx'
import { Window } from '../ui/Window.tsx'

/** Where the player is in the command menu. */
export type MenuLevel = 'root' | 'magic' | 'ability' | 'item'

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
  onLevel: (level: MenuLevel) => void
  onChoose: (kind: CommandKind, action: ActionInfo | null) => void
  onCancel: () => void
}

/** The command window (bottom left): Attack, Magic, Ability, Item, Defend. */
export function CommandMenu(props: Props) {
  const { t } = useTranslation()
  const { actor, pending, level, onCancel, onLevel } = props

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
  } else if (level === 'magic' || level === 'ability') {
    entries = [...actionEntries(props, actor, level, t), back]
  } else if (level === 'item') {
    entries = [...itemEntries(props, t), back]
  } else {
    entries = rootEntries(props, actor, t)
  }

  return (
    <Window testId="command-menu">
      {pending !== null && <p>{t('ui-command-choose-target')}</p>}
      <MenuList entries={entries} />
    </Window>
  )
}

type Translate = ReturnType<typeof useTranslation>['t']

function rootEntries(props: Props, actor: CombatantState, t: Translate): MenuEntry[] {
  const { catalog, onChoose, onLevel } = props
  const limitReady = actor.limit !== null && actor.limit >= actor.atbFull
  const attackId = limitReady && actor.limitAction ? actor.limitAction : actor.attack
  const attack = catalog.actions[attackId]
  const knows = (category: ActionInfo['category']) =>
    actor.actions.some((id) => catalog.actions[id]?.category === category)

  const entries: MenuEntry[] = [
    {
      key: 'attack',
      label: limitReady ? t('ui-command-limit') : t('ui-command-attack'),
      disabled: !attack,
      onSelect: () => attack && onChoose({ type: 'attack' }, attack),
    },
  ]
  if (knows('magic')) {
    entries.push({ key: 'magic', label: t('ui-command-magic'), onSelect: () => onLevel('magic') })
  }
  if (knows('ability')) {
    entries.push({
      key: 'ability',
      label: t('ui-command-ability'),
      onSelect: () => onLevel('ability'),
    })
  }
  entries.push(
    { key: 'item', label: t('ui-command-item'), onSelect: () => onLevel('item') },
    {
      key: 'defend',
      label: t('ui-command-defend'),
      onSelect: () => onChoose({ type: 'defend' }, null),
    },
  )
  return entries
}

function actionEntries(
  { catalog, onChoose }: Props,
  actor: CombatantState,
  category: 'magic' | 'ability',
  t: Translate,
): MenuEntry[] {
  const silenced = category === 'magic' && actor.statuses.includes('silence')
  return actor.actions.flatMap((id: ActionId) => {
    const action = catalog.actions[id]
    if (action?.category !== category) return []
    return [
      {
        key: id,
        label: t(`action-${id}`),
        detail: action.mpCost > 0 ? t('ui-mp-cost', { cost: action.mpCost }) : undefined,
        disabled: silenced || action.mpCost > actor.mp,
        onSelect: () => onChoose({ type: 'action', id }, action),
      },
    ]
  })
}

function itemEntries({ state, catalog, onChoose }: Props, t: Translate): MenuEntry[] {
  return state.inventory.map(({ item, count }) => {
    const info = catalog.items[item]
    const action = info ? catalog.actions[info.action] : undefined
    return {
      key: item,
      label: t(`item-${item}`),
      detail: t('ui-item-count', { count }),
      disabled: count === 0 || !action,
      onSelect: () => action && onChoose({ type: 'item', id: item }, action),
    }
  })
}
