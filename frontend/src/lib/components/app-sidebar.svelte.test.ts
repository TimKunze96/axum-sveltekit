import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import AppLayout from './app-layout.svelte';
import type { User } from '$lib/types';

const user: User = { id: 1, name: 'Ada Lovelace', email: 'ada@example.com', is_admin: false };

const content = createRawSnippet(() => ({ render: () => '<p>Page</p>' }));

function sidebarState(): string | null {
  return document.querySelector('[data-slot="sidebar"]')?.getAttribute('data-state') ?? null;
}

describe('app sidebar', () => {
  beforeEach(async () => {
    await page.viewport(1280, 800);
  });

  it('collapses to icons from the header trigger and expands again', async () => {
    render(AppLayout, { user, children: content });

    expect(sidebarState()).toBe('expanded');
    await expect.element(page.getByRole('link', { name: 'Dashboard' })).toBeVisible();
    const trigger = page.getByRole('button', { name: 'Toggle Sidebar' });
    await trigger.click();
    expect(sidebarState()).toBe('collapsed');
    await trigger.click();
    expect(sidebarState()).toBe('expanded');
  });

  it('starts collapsed when the sidebar cookie said so', async () => {
    render(AppLayout, { user, sidebarOpen: false, children: content });
    expect(sidebarState()).toBe('collapsed');
  });
});
