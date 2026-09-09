import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import AppLayout from './app-layout.svelte';
import type { User } from '$lib/types';

const user: User = { id: 1, name: 'Ada Lovelace', email: 'ada@example.com', is_admin: false };

const content = createRawSnippet(() => ({ render: () => '<p data-testid="content">Page</p>' }));

describe('app layout', () => {
  // The sidebar folds into a closed sheet under the mobile breakpoint;
  // vitest's default viewport is phone-sized.
  beforeEach(async () => {
    await page.viewport(1280, 800);
  });

  it('renders the platform links and the page', async () => {
    render(AppLayout, { user, children: content });

    await expect.element(page.getByTestId('content')).toHaveTextContent('Page');
    await expect
      .element(page.getByRole('link', { name: 'Dashboard' }))
      .toHaveAttribute('href', '/dashboard');
  });

  it('opens the account menu from the sidebar footer', async () => {
    render(AppLayout, { user, children: content });

    await expect
      .element(page.getByTestId('sidebar-menu-button'))
      .toMatchTextContent('Ada Lovelace');
    await page.getByTestId('sidebar-menu-button').click();
    await expect.element(page.getByText('ada@example.com')).toBeVisible();
    await expect
      .element(page.getByRole('menuitem', { name: 'Settings' }))
      .toHaveAttribute('href', '/settings/profile');
    await expect.element(page.getByTestId('logout-button')).toHaveTextContent('Log out');
  });

  it('renders breadcrumbs with the last one as the current page', async () => {
    render(AppLayout, {
      user,
      breadcrumbs: [
        { title: 'Settings', href: '/settings/profile' },
        { title: 'Password', href: '/settings/password' },
      ],
      children: content,
    });

    await expect
      .element(page.getByRole('link', { name: 'Settings', exact: true }))
      .toHaveAttribute('href', '/settings/profile');
    // The current page is a disabled link marked aria-current.
    await expect
      .element(page.getByRole('link', { name: 'Password' }))
      .toHaveAttribute('aria-current', 'page');
  });
});
