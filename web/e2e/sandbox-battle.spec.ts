import { expect, test } from '@playwright/test'

// battle-basic: party #0-#2 (warrior, mage, cleric), enemies #3-#5.
const SCENARIO = '/sandbox/battle-basic'

test('the player attacks an enemy and sees the result over its avatar', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42`)

  const attack = page.getByTestId('command-menu').getByRole('button', { name: 'Attack' })
  await expect(attack).toBeEnabled({ timeout: 15_000 })
  await attack.click()
  await page.getByTestId('combatant-3').click()

  // Damage number or "Miss": either way the hit was resolved on that enemy.
  await expect(page.getByTestId('combatant-3').getByTestId('popup')).toBeVisible()
})

test('auto-battle plays the scenario to victory', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42&auto=1&speed=800`)

  await expect(page.getByTestId('outcome')).toContainText('Victory', { timeout: 30_000 })
})

test('the interface switches to Portuguese', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42`)

  await page.getByTestId('language').selectOption('pt-BR')

  await expect(
    page.getByTestId('debug-panel').getByRole('button', { name: 'Pausar' }),
  ).toBeVisible()
  await expect(page.getByTestId('combatant-3')).toHaveAccessibleName('Lobo da Floresta')
})
