import type { BagEntry } from '../protocol/generated/BagEntry.ts'
import type { EquipmentId } from '../protocol/generated/EquipmentId.ts'
import type { HeroView } from '../protocol/generated/HeroView.ts'
import type { ItemId } from '../protocol/generated/ItemId.ts'
import type { Slot } from '../protocol/generated/Slot.ts'
import { StatBar } from '../battle/StatBar.tsx'
import { useTranslation } from '../i18n/use-translation.ts'
import { Button } from '../ui/Button.tsx'
import { Select } from '../ui/Select.tsx'
import { useMenuKeys } from '../ui/use-menu-keys.ts'
import { Window } from '../ui/Window.tsx'

interface Props {
  party: HeroView[]
  bag: BagEntry[]
  inventory: Partial<Record<ItemId, number>>
  kalds: number
  onEquip: (hero: number, slot: Slot, id: EquipmentId | null) => void
  onUse: (item: ItemId, hero: number) => void
  onClose: () => void
}

/** Items usable outside battle. */
const FIELD_ITEMS: ItemId[] = ['potion', 'hi-potion', 'ether']

const NONE = ''

/** Heroes, their stats and gear, and the items usable on the map. Esc closes. */
export function PartyWindow({ party, bag, inventory, kalds, onEquip, onUse, onClose }: Props) {
  const { t } = useTranslation()
  useMenuKeys((key) => {
    if (key !== 'cancel') return false
    onClose()
    return true
  })

  const gearSelect = (hero: HeroView, index: number, slot: Slot) => {
    const worn = slot === 'weapon' ? hero.weapon : hero.armor
    const fitting = bag.filter((entry) => entry.slot === slot && entry.classes.includes(hero.class))
    const options = [
      { value: NONE, label: t('ui-party-none') },
      ...(worn === null ? [] : [{ value: worn, label: t(`equipment-${worn}`) }]),
      ...fitting.map((entry) => ({
        value: entry.id,
        label: `${t(`equipment-${entry.id}`)} ×${entry.count}`,
      })),
    ]
    return (
      <Select
        label={t(`ui-party-${slot}`)}
        value={worn ?? NONE}
        options={options}
        onChange={(value) => onEquip(index, slot, value === NONE ? null : value)}
        testId={`equip-${hero.hero}-${slot}`}
      />
    )
  }

  return (
    <Window className="game-window party-window" testId="party-window">
      <div className="game-window__head">
        <h2>{t('ui-party-title')}</h2>
        <span className="kalds">{t('ui-kalds', { amount: kalds })}</span>
      </div>
      <div className="party-window__heroes">
        {party.map((hero, index) => (
          <section key={hero.hero} className="hero-card" data-testid={`hero-${hero.hero}`}>
            <h3>
              {t(`hero-${hero.hero}`)}{' '}
              <small>
                {t(`class-${hero.class}`)} · {t('ui-party-level', { level: hero.level })}
              </small>
            </h3>
            <StatBar label={t('ui-stat-hp')} value={hero.hp} max={hero.maxHp} variant="hp" />
            <StatBar label={t('ui-stat-mp')} value={hero.mp} max={hero.maxMp} variant="mp" />
            <p className="hero-card__xp">{t('ui-party-xp', { xp: hero.xp, next: hero.xpNext })}</p>
            <dl className="hero-card__stats">
              {(
                [
                  ['attack', hero.stats.attack],
                  ['defense', hero.stats.defense],
                  ['magic-attack', hero.stats.magicAttack],
                  ['magic-defense', hero.stats.magicDefense],
                  ['dexterity', hero.stats.dexterity],
                  ['luck', hero.stats.luck],
                ] as const
              ).map(([stat, value]) => (
                <div key={stat}>
                  <dt>{t(`ui-stat-${stat}`)}</dt>
                  <dd>{value}</dd>
                </div>
              ))}
            </dl>
            {gearSelect(hero, index, 'weapon')}
            {gearSelect(hero, index, 'armor')}
            <p className="hero-card__actions">
              {hero.actions.map((action) => t(`action-${action}`)).join(' · ') ||
                t('ui-party-no-actions')}
            </p>
            <div className="hero-card__items">
              {FIELD_ITEMS.filter((item) => (inventory[item] ?? 0) > 0).map((item) => (
                <Button
                  key={item}
                  onClick={() => onUse(item, index)}
                  testId={`use-${item}-${hero.hero}`}
                >
                  {t('ui-party-use', { item: t(`item-${item}`), count: inventory[item] ?? 0 })}
                </Button>
              ))}
            </div>
          </section>
        ))}
      </div>
      <Button onClick={onClose} testId="party-close">
        {t('ui-close')}
      </Button>
    </Window>
  )
}
