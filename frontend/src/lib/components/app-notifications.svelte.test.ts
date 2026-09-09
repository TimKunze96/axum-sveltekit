import { page } from 'vitest/browser';
import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import AppNotifications from './app-notifications.svelte';

describe('app notifications', () => {
  // First, before any toast of this file lingers in the shared toaster.
  it('shows nothing on a plain page load', async () => {
    render(AppNotifications, { notification: null });

    await expect.element(page.getByRole('region', { name: /notifications/i })).toBeInTheDocument();
    expect(document.querySelectorAll('[data-sonner-toast]')).toHaveLength(0);
  });

  it('toasts a success flash with its title and body', async () => {
    render(AppNotifications, {
      notification: { title: 'Signed in', body: 'Welcome back, Ada!', type: 'success' },
    });

    const toast = page.getByText('Signed in').element().closest('[data-sonner-toast]');
    expect(toast).toHaveAttribute('data-type', 'success');
    expect(toast).toMatchTextContent('Welcome back, Ada!');
  });

  it('toasts an error flash as an error', async () => {
    render(AppNotifications, {
      notification: { title: 'Sign-in failed', body: null, type: 'error' },
    });

    const toast = page.getByText('Sign-in failed').element().closest('[data-sonner-toast]');
    expect(toast).toHaveAttribute('data-type', 'error');
  });
});
