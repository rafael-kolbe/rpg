import { expect, test, type Page } from '@playwright/test'

/** A username nobody else used: tests share the development database. */
function newUsername(): string {
  return `e2e${Date.now().toString(36)}${Math.floor(Math.random() * 1e6)}`.slice(0, 20)
}

async function signUp(page: Page, username: string, password: string): Promise<string> {
  await page.goto('/signup')
  await page.getByTestId('account-username').fill(username)
  await page.getByTestId('account-password-new').fill(password)
  await page.getByTestId('account-password-confirm').fill(password)
  await page.getByTestId('account-submit').click()
  const key = await page.getByTestId('recovery-key').textContent()
  expect(key).toMatch(/^[0-9A-Z]{4}(-[0-9A-Z]{4}){4}$/)
  return key ?? ''
}

/** Confirms the key was saved, lands on the mode choice, then goes home. */
async function continueHome(page: Page) {
  await expect(page.getByTestId('recovery-key-continue')).toBeDisabled()
  await page.getByTestId('recovery-key-saved').check()
  await page.getByTestId('recovery-key-continue').click()
  await expect(page.getByTestId('mode-select')).toBeVisible()
  await page.goto('/')
}

async function logIn(page: Page, username: string, password: string) {
  await page.goto('/login')
  await page.getByTestId('account-username').fill(username)
  await page.getByTestId('account-password-current').fill(password)
  await page.getByTestId('account-submit').click()
}

/** A successful login lands on the mode choice; back home to read the status. */
async function expectLoggedIn(page: Page, username: string) {
  await expect(page.getByTestId('mode-select')).toBeVisible()
  await page.goto('/')
  await expect(page.getByTestId('account-status')).toContainText(username)
}

test('a player signs up, keeps the recovery key, logs out and back in', async ({
  page,
  context,
}) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write'])
  const username = newUsername()
  const key = await signUp(page, username, 'first-pass')

  await page.getByTestId('recovery-key-copy').click()
  expect(await page.evaluate<string>('navigator.clipboard.readText()')).toBe(key)
  await continueHome(page)
  await expect(page.getByTestId('account-status')).toContainText(username)

  await page.getByTestId('account-log-out').click()
  await expect(page.getByTestId('account-status')).not.toContainText(username)

  await logIn(page, username, 'wrong-pass')
  await expect(page.getByTestId('account-error')).toBeVisible()
  await logIn(page, username.toUpperCase(), 'first-pass')
  await expectLoggedIn(page, username)
})

test('a username with symbols or a taken one is refused', async ({ page }) => {
  const username = newUsername()
  await signUp(page, username, 'first-pass')

  await page.goto('/signup')
  await page.getByTestId('account-username').fill('bad_name1')
  await page.getByTestId('account-password-new').fill('first-pass')
  await page.getByTestId('account-password-confirm').fill('first-pass')
  await page.getByTestId('account-submit').click()
  await expect(page.getByTestId('account-error')).toBeVisible()

  await page.getByTestId('account-username').fill(username)
  await page.getByTestId('account-submit').click()
  await expect(page.getByTestId('account-error')).toBeVisible()
  await expect(page.getByTestId('recovery-key')).toHaveCount(0)
})

test('the recovery key changes a forgotten password and is replaced', async ({ page }) => {
  const username = newUsername()
  const key = await signUp(page, username, 'first-pass')
  await continueHome(page)
  await page.getByTestId('account-log-out').click()

  await page.goto('/recover')
  await page.getByTestId('account-username').fill(username)
  await page.getByTestId('account-recovery-key').fill(key.toLowerCase().replaceAll('-', ' '))
  await page.getByTestId('account-password-new').fill('second-pass')
  await page.getByTestId('account-password-confirm').fill('second-pass')
  await page.getByTestId('account-submit').click()
  const newKey = await page.getByTestId('recovery-key').textContent()
  expect(newKey).not.toBe(key)
  await continueHome(page)
  await expect(page.getByTestId('account-status')).toContainText(username)

  await page.getByTestId('account-log-out').click()
  await logIn(page, username, 'first-pass')
  await expect(page.getByTestId('account-error')).toBeVisible()
  await logIn(page, username, 'second-pass')
  await expectLoggedIn(page, username)
})
