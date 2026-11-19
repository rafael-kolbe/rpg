import { useState } from 'react'

import type { ShopEntry } from '../protocol/generated/ShopEntry.ts'
import type { ShopGood } from '../protocol/generated/ShopGood.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { MenuList } from '../ui/MenuList.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { Window } from '../ui/Window.tsx'
import { goodName } from './goods.ts'

interface Props {
  goods: ShopEntry[]
  kalds: number
  onBuy: (good: ShopGood) => void
  onSell: (good: ShopGood) => void
  onLeave: () => void
}

type Mode = 'buy' | 'sell'

/** A shop: buy or sell one at a time; ←/→ switch, Esc leaves. */
export function ShopWindow({ goods, kalds, onBuy, onSell, onLeave }: Props) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<Mode>('buy')
  useMenuKeys((key) => {
    if (key !== 'left' && key !== 'right') return false
    setMode((current) => (current === 'buy' ? 'sell' : 'buy'))
    return true
  })
  const shown = mode === 'buy' ? goods : goods.filter((entry) => entry.owned > 0)
  const price = (entry: ShopEntry) => (mode === 'buy' ? entry.price : Math.floor(entry.price / 2))

  return (
    <Window className="game-window shop" testId="shop">
      <div className="game-window__head">
        <h2>{t('ui-shop-title')}</h2>
        <span className="kalds">{t('ui-kalds', { amount: kalds })}</span>
      </div>
      <div className="shop__modes">
        <Button onClick={() => setMode('buy')} pressed={mode === 'buy'} testId="shop-buy">
          {t('ui-shop-buy')}
        </Button>
        <Button onClick={() => setMode('sell')} pressed={mode === 'sell'} testId="shop-sell">
          {t('ui-shop-sell')}
        </Button>
      </div>
      {shown.length === 0 ? (
        <p className="game-window__empty">{t('ui-shop-nothing')}</p>
      ) : (
        <MenuList
          key={mode}
          keyboard
          onCancel={onLeave}
          entries={shown.map((entry) => ({
            key: `${entry.good.type}-${entry.good.id}`,
            label: t(goodName(entry.good)),
            detail: t('ui-shop-line', { price: price(entry), owned: entry.owned }),
            disabled: mode === 'buy' && entry.price > kalds,
            onSelect: () => (mode === 'buy' ? onBuy(entry.good) : onSell(entry.good)),
          }))}
        />
      )}
      <p className="game-window__hint">{t('ui-shop-keys')}</p>
      <Button onClick={onLeave} testId="shop-leave">
        {t('ui-shop-leave')}
      </Button>
    </Window>
  )
}
