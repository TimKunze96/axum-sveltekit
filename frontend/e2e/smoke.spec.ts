import { expect, test } from '@playwright/test';

const APP_NAME = process.env.PUBLIC_APP_NAME ?? 'Starter';

test('the welcome page renders through the shared origin', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: APP_NAME })).toBeVisible();
  await expect(page.getByTestId('login')).toHaveAttribute('href', '/login');
  await expect(page.getByTestId('register')).toHaveAttribute('href', '/register');
  await expect(page).toHaveTitle(APP_NAME);
});

test('the API health probe answers on the shared origin', async ({ request }) => {
  const response = await request.get('/api/health');
  expect(response.status()).toBe(200);
  expect(await response.json()).toEqual({ status: 'ok' });
});

test('guests are sent to the login from the app shell', async ({ page }) => {
  const responses = await Promise.all(
    ['/dashboard', '/settings/profile'].map((path) => page.request.get(path, { maxRedirects: 0 })),
  );
  for (const response of responses) {
    expect(response.status(), response.url()).toBe(303);
    expect(response.headers()['location'], response.url()).toBe('/login');
  }
});

test('an account can register, see the dashboard and log out', async ({ page }) => {
  const email = `e2e-${Date.now()}@example.com`;
  await page.goto('/register');
  await page.getByLabel('Name').fill('E2E Account');
  await page.getByLabel('Email').fill(email);
  await page.getByLabel('Password', { exact: true }).fill('correct horse battery staple');
  await page.getByLabel('Confirm password').fill('correct horse battery staple');
  await page.getByRole('button', { name: 'Create account' }).click();
  await expect(page).toHaveURL(/\/dashboard$/);
  await expect(page.getByText('Welcome, E2E Account.')).toBeVisible();

  await page.getByTestId('sidebar-menu-button').click();
  await page.getByTestId('logout-button').click();
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByTestId('login')).toBeVisible();
});

test('a forgotten password can be requested from the login page', async ({ page }) => {
  await page.goto('/login');
  await page.getByRole('link', { name: 'Forgot your password?' }).click();
  await expect(page).toHaveURL(/\/forgot-password$/);
  await page.getByLabel('Email').fill(`nobody-${Date.now()}@example.com`);
  await page.getByRole('button', { name: 'Email a reset link' }).click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByText('Check your mail')).toBeVisible();

  const bare = await page.request.get('/reset-password', { maxRedirects: 0 });
  expect(bare.status()).toBe(303);
  expect(bare.headers()['location']).toBe('/forgot-password');
});

test('the appearance cookie decides the server-rendered theme', async ({ page, baseURL }) => {
  await page.context().addCookies([{ name: 'appearance', value: 'dark', url: baseURL }]);
  await page.goto('/');
  await expect(page.locator('html')).toHaveClass(/dark/);
});
