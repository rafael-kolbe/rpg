import { expect, test, type Page } from '@playwright/test'

/** Signs a new account up; ends on the mode choice. */
async function signUp(page: Page) {
  const username = `camp${Date.now().toString(36)}${Math.floor(Math.random() * 1e4)}`.slice(0, 20)
  await page.goto('/signup')
  await page.getByTestId('account-username').fill(username)
  await page.getByTestId('account-password-new').fill('campaign1')
  await page.getByTestId('account-password-confirm').fill('campaign1')
  await page.getByTestId('account-submit').click()
  await page.getByTestId('recovery-key-saved').check()
  await page.getByTestId('recovery-key-continue').click()
}

test('a new player starts the Campaign, walks and talks, and the game is saved', async ({
  page,
}) => {
  await signUp(page)
  const modes = page.getByTestId('mode-select')
  await expect(modes.getByRole('button', { name: /Multiplayer/ })).toBeDisabled()
  await modes.getByRole('button', { name: /Campaign/ }).click()

  // The opening scene, line by line.
  const dialogue = page.getByTestId('dialogue')
  await expect(dialogue).toContainText('Brookhollow')
  while (await dialogue.isVisible()) {
    const line = await dialogue.textContent()
    await page.keyboard.press('Enter')
    // The next line, or no dialogue at all.
    await expect
      .poll(async () => ((await dialogue.isVisible()) ? await dialogue.textContent() : null))
      .not.toBe(line)
  }

  // The elder is south of the square: walk there and talk.
  await page.keyboard.press('ArrowDown')
  await expect(page.getByTestId('interact')).toContainText('Elder Maren')
  await page.getByTestId('interact').click()
  await expect(dialogue).toContainText('Carvel')

  // Saved on the server: a reload continues at the elder, scene over.
  await page.reload()
  await expect(modes.getByRole('button', { name: /Continue/ })).toBeVisible()
  await modes.getByRole('button', { name: /Campaign/ }).click()
  await expect(page.getByTestId('interact')).toContainText('Elder Maren')
})

test('opening the game in a second tab closes the first one', async ({ page, context }) => {
  await signUp(page)
  await page
    .getByTestId('mode-select')
    .getByRole('button', { name: /Campaign/ })
    .click()
  await expect(page.getByTestId('dialogue')).toBeVisible()

  const second = await context.newPage()
  await second.goto('/play')
  await second
    .getByTestId('mode-select')
    .getByRole('button', { name: /Campaign/ })
    .click()
  await expect(second.getByTestId('dialogue')).toBeVisible()

  await expect(page.getByTestId('game-closed')).toContainText('another tab')
  await expect(second.getByTestId('game-closed')).toHaveCount(0)
})
