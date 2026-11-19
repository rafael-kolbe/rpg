import type { ReactNode } from 'react'

import type { SceneView } from '../protocol/generated/SceneView.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { Window } from '../ui/Window.tsx'

type Notice = Extract<SceneView, { type: 'victory' | 'defeat' | 'rested' | 'found' | 'game_over' }>

interface Props {
  scene: Notice
  onContinue: () => void
}

/** Rewards, a defeat, a Game Over, a rest or a chest's loot; confirm closes it. */
export function SceneNotice({ scene, onContinue }: Props) {
  const { t } = useTranslation()
  useMenuKeys((key) => {
    if (key !== 'confirm' && key !== 'cancel') return false
    onContinue()
    return true
  })

  let title: string
  let lines: ReactNode[] = []
  switch (scene.type) {
    case 'victory': {
      const { rewards } = scene
      title = t('ui-victory-title')
      lines = [
        t('ui-victory-xp', { xp: rewards.xp }),
        t('ui-victory-kalds', { amount: rewards.kalds }),
        ...Object.entries(rewards.items).map(([item, count]) =>
          t('ui-found-item', { name: t(`item-${item}`), count: count ?? 0 }),
        ),
        ...rewards.levelUps.flatMap((up) => [
          t('ui-level-up', { name: t(`hero-${up.hero}`), level: up.level }),
          ...up.learned.map((action) => t('ui-learned', { action: t(`action-${action}`) })),
        ]),
        ...rewards.questsDone.map((quest) => t('ui-quest-done', { name: t(`quest-${quest}`) })),
      ]
      break
    }
    case 'game_over':
      title = t('ui-game-over-title')
      lines = [t('ui-game-over-text')]
      break
    case 'defeat':
      title = t('ui-defeat-title')
      lines = [t('ui-defeat-text')]
      break
    case 'rested':
      title = t('ui-rested-title')
      lines = [t('ui-rested-text')]
      break
    case 'found': {
      const { loot } = scene
      title = t('ui-found-title')
      lines = [
        ...Object.entries(loot.items).map(([item, count]) =>
          t('ui-found-item', { name: t(`item-${item}`), count: count ?? 0 }),
        ),
        ...loot.equipment.map((piece) => t(`equipment-${piece}`)),
        ...(loot.kalds > 0 ? [t('ui-kalds', { amount: loot.kalds })] : []),
      ]
      break
    }
  }

  return (
    <Window className="game-window notice" testId={`notice-${scene.type}`}>
      <h2>{title}</h2>
      <ul className="notice__lines">
        {lines.map((line, index) => (
          <li key={index}>{line}</li>
        ))}
      </ul>
      <Button onClick={onContinue} testId="notice-continue">
        {t('ui-continue')}
      </Button>
    </Window>
  )
}
