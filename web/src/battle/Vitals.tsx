import type { CombatantState } from '../protocol/generated/CombatantState.ts'
import { useTranslation } from '../i18n/use-translation.ts'
import { StatBar } from './StatBar.tsx'

/**
 * HP and MP of a combatant, numbers and bars. Without MP an empty cell keeps
 * grid columns aligned.
 */
export function Vitals({ combatant }: { combatant: CombatantState }) {
  const { t } = useTranslation()
  return (
    <>
      <StatBar label={t('ui-stat-hp')} value={combatant.hp} max={combatant.maxHp} variant="hp" />
      {combatant.maxMp > 0 ? (
        <StatBar label={t('ui-stat-mp')} value={combatant.mp} max={combatant.maxMp} variant="mp" />
      ) : (
        <span />
      )}
    </>
  )
}
