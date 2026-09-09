import { page } from 'vitest/browser';
import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import LoginForm from './login-form.svelte';

/** The (url, method, JSON body) of the nth call to a stubbed fetch. */
function call(fetcher: { mock: { calls: unknown[][] } }, index = 0) {
  const [url, init] = fetcher.mock.calls[index] as [string, RequestInit];
  return { url, method: init.method, body: JSON.parse(init.body as string) };
}

describe('login form', () => {
  it('posts the credentials and follows the redirect', async () => {
    const fetcher = vi.fn(async () => Response.json({ redirect: '/dashboard' }));
    const navigate = vi.fn(async () => {});
    render(LoginForm, { fetcher, navigate });

    await page.getByLabelText('Email').fill('ada@example.com');
    await page.getByLabelText('Password').fill('correct horse battery staple');
    await page.getByRole('button', { name: 'Log in' }).click();

    await vi.waitFor(() => expect(navigate).toHaveBeenCalledOnce());
    expect(call(fetcher)).toEqual({
      url: '/api/login',
      method: 'POST',
      body: { email: 'ada@example.com', password: 'correct horse battery staple' },
    });
    expect(navigate).toHaveBeenCalledWith({ ok: true, redirect: '/dashboard' });
  });

  it('shows the API field errors inline', async () => {
    const fetcher = vi.fn(async () =>
      Response.json(
        {
          message: 'These credentials do not match our records.',
          errors: { email: ['These credentials do not match our records.'] },
        },
        { status: 422 },
      ),
    );
    const navigate = vi.fn(async () => {});
    render(LoginForm, { fetcher, navigate });

    await page.getByLabelText('Email').fill('ada@example.com');
    await page.getByLabelText('Password').fill('wrong');
    await page.getByRole('button', { name: 'Log in' }).click();

    await expect
      .element(page.getByRole('alert'))
      .toHaveTextContent('These credentials do not match our records.');
    expect(navigate).not.toHaveBeenCalled();
    await expect
      .element(page.getByRole('link', { name: 'Create one' }))
      .toHaveAttribute('href', '/register');
    await expect
      .element(page.getByRole('link', { name: 'Forgot your password?' }))
      .toHaveAttribute('href', '/forgot-password');
  });
});
