import { page } from 'vitest/browser';
import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import AppearanceTabs from './appearance-tabs.svelte';

afterEach(() => {
  localStorage.removeItem('appearance');
  document.documentElement.classList.remove('dark');
});

describe('appearance tabs', () => {
  it('switches the theme and marks the choice', async () => {
    render(AppearanceTabs);

    await page.getByRole('button', { name: 'Dark' }).click();
    await expect
      .element(page.getByRole('button', { name: 'Dark' }))
      .toHaveAttribute('aria-pressed', 'true');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    expect(localStorage.getItem('appearance')).toBe('dark');

    await page.getByRole('button', { name: 'Light' }).click();
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    await expect
      .element(page.getByRole('button', { name: 'Dark' }))
      .toHaveAttribute('aria-pressed', 'false');
  });
});
