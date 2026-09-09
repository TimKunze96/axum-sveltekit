import { page } from 'vitest/browser';
import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ForgotPasswordForm from './forgot-password-form.svelte';

describe('forgot password form', () => {
  it('posts the email and follows the redirect', async () => {
    const fetcher = vi.fn(async () => Response.json({ redirect: '/login' }));
    const navigate = vi.fn(async () => {});
    render(ForgotPasswordForm, { fetcher, navigate });

    await page.getByLabelText('Email').fill('ada@example.com');
    await page.getByRole('button', { name: 'Email a reset link' }).click();

    await vi.waitFor(() => expect(navigate).toHaveBeenCalledWith({ ok: true, redirect: '/login' }));
    const [url, init] = fetcher.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/forgot-password');
    expect(JSON.parse(init.body as string)).toEqual({ email: 'ada@example.com' });
  });

  it('shows the email error inline', async () => {
    const fetcher = vi.fn(async () =>
      Response.json(
        {
          message: 'Enter a valid email address.',
          errors: { email: ['Enter a valid email address.'] },
        },
        { status: 422 },
      ),
    );
    render(ForgotPasswordForm, { fetcher, navigate: vi.fn(async () => {}) });

    await page.getByLabelText('Email').fill('nope');
    await page.getByRole('button', { name: 'Email a reset link' }).click();
    await expect.element(page.getByRole('alert')).toHaveTextContent('Enter a valid email address.');
  });
});
