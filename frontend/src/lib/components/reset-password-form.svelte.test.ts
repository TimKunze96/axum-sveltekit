import { page } from 'vitest/browser';
import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ResetPasswordForm from './reset-password-form.svelte';

describe('reset password form', () => {
  it('posts the token, the email and the new password', async () => {
    const fetcher = vi.fn(async () => Response.json({ redirect: '/login' }));
    const navigate = vi.fn(async () => {});
    render(ResetPasswordForm, { token: 'tok-1', email: 'ada@example.com', fetcher, navigate });

    await expect.element(page.getByLabelText('Email')).toHaveValue('ada@example.com');
    await page.getByLabelText('New password').fill('correct horse battery staple');
    await page.getByLabelText('Confirm password').fill('correct horse battery staple');
    await page.getByRole('button', { name: 'Reset password' }).click();

    await vi.waitFor(() => expect(navigate).toHaveBeenCalledOnce());
    const [url, init] = fetcher.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/reset-password');
    expect(JSON.parse(init.body as string)).toEqual({
      token: 'tok-1',
      email: 'ada@example.com',
      password: 'correct horse battery staple',
      password_confirmation: 'correct horse battery staple',
    });
  });

  it('shows a spent link as the email error', async () => {
    const message = 'This password reset link is invalid or has expired.';
    const fetcher = vi.fn(async () =>
      Response.json({ message, errors: { email: [message] } }, { status: 422 }),
    );
    render(ResetPasswordForm, {
      token: 'old',
      email: 'ada@example.com',
      fetcher,
      navigate: vi.fn(async () => {}),
    });

    await page.getByLabelText('New password').fill('correct horse battery staple');
    await page.getByRole('button', { name: 'Reset password' }).click();
    await expect.element(page.getByRole('alert')).toHaveTextContent(message);
  });
});
