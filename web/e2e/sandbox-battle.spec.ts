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

test('the player attacks with the keyboard alone', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42`)

  const attack = page.getByTestId('command-menu').getByRole('button', { name: 'Attack' })
  await expect(attack).toBeEnabled({ timeout: 15_000 })
  // The cursor starts on Attack, then on the first living enemy.
  await page.keyboard.press('Enter')
  await expect(page.getByTestId('combatant-3')).toHaveClass(/combatant--cursor/)
  await page.keyboard.press('Enter')

  await expect(page.getByTestId('combatant-3').getByTestId('popup')).toBeVisible()
})

test('the left arrow defends from the root menu', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42`)

  const menu = page.getByTestId('command-menu')
  await expect(menu.getByRole('button', { name: 'Attack' })).toBeEnabled({ timeout: 15_000 })
  await expect(menu.getByRole('button', { name: 'Defend' })).toHaveCount(0)
  await page.keyboard.press('ArrowLeft')

  await expect(page.getByTestId('event-log')).toContainText('defends')
})

test('M switches the action memory and the browser keeps the choice', async ({ page }) => {
  await page.goto(`${SCENARIO}?seed=42`)
  const toggle = page.getByTestId('action-memory')
  await expect(toggle).toHaveAttribute('aria-pressed', 'false')

  await page.keyboard.press('m')
  await expect(toggle).toHaveAttribute('aria-pressed', 'true')
  await page.reload()

  await expect(page.getByTestId('action-memory')).toHaveAttribute('aria-pressed', 'true')
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
