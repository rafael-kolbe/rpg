import type { RegionView } from '../protocol/generated/RegionView.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { Window } from '../ui/Window.tsx'
import { classNames } from '../utils/class-names.ts'

interface Props {
  regions: RegionView[]
  onClose: () => void
}

/** The world map: the pieces the player has been to; the rest is fog. */
export function WorldMapWindow({ regions, onClose }: Props) {
  const { t } = useTranslation()
  useMenuKeys((key) => {
    if (key !== 'cancel' && key !== 'memory' && key !== 'confirm') return false
    onClose()
    return true
  })
  return (
    <Window className="game-window world-map" testId="world-map">
      <h2>{t('ui-world-map')}</h2>
      <svg viewBox="0 0 1000 1000" className="world-map__svg">
        <rect className="world-map__fog" width="1000" height="1000" />
        {regions.map((region) => (
          <g key={region.id} className={classNames('region', region.current && 'region--current')}>
            <path d={region.shape} />
            <text x={region.label[0]} y={region.label[1]} textAnchor="middle">
              {t(`region-${region.id}`)}
            </text>
          </g>
        ))}
      </svg>
      <Button onClick={onClose}>{t('ui-close')}</Button>
    </Window>
  )
}
