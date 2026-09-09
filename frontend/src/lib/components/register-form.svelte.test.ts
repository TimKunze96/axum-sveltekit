import { page } from 'vitest/browser';
import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import RegisterForm from './register-form.svelte';

describe('register form', () => {
  it('posts every field and shows the errors per field', async () => {
    const fetcher = vi.fn(async () =>
      Response.json(
        {
          message: 'A name is required. (and 1 more error)',
          errors: {
            name: ['A name is required.'],
            password_confirmation: ['The password confirmation does not match.'],
          },
        },
        { status: 422 },
      ),
    );
    const navigate = vi.fn(async () => {});
    render(RegisterForm, { fetcher, navigate });

    await page.getByLabelText('Email').fill('ada@example.com');
    await page.getByLabelText('Password', { exact: true }).fill('correct horse battery staple');
    await page.getByLabelText('Confirm password').fill('something else');
    await page.getByRole('button', { name: 'Create account' }).click();

    await expect.element(page.getByText('A name is required.')).toBeVisible();
    await expect.element(page.getByText('The password confirmation does not match.')).toBeVisible();
    const [url, init] = fetcher.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/register');
    expect(JSON.parse(init.body as string)).toEqual({
      name: '',
      email: 'ada@example.com',
      password: 'correct horse battery staple',
      password_confirmation: 'something else',
    });
    expect(navigate).not.toHaveBeenCalled();
  });
});
