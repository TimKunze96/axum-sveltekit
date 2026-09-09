import { describe, expect, it } from 'vitest';
import { firstErrors, submit } from './client';

function answering(status: number, body: unknown): typeof globalThis.fetch {
  return async () => new Response(JSON.stringify(body), { status });
}

describe('submit', () => {
  it('posts JSON with the JSON dialect and returns the redirect', async () => {
    let seen: { url: string; init: RequestInit } | null = null;
    const fetcher: typeof globalThis.fetch = async (url, init) => {
      seen = { url: String(url), init: init ?? {} };
      return new Response(JSON.stringify({ redirect: '/dashboard' }), { status: 200 });
    };
    const outcome = await submit('POST', '/api/login', { email: 'a@b.co', password: 'x' }, fetcher);
    expect(outcome).toEqual({ ok: true, redirect: '/dashboard' });
    const request = seen as { url: string; init: RequestInit } | null;
    if (!request) throw new Error('fetch was not called');
    const headers = request.init.headers as Record<string, string>;
    expect(request.url).toBe('/api/login');
    expect(request.init.method).toBe('POST');
    expect(headers.accept).toBe('application/json');
    expect(headers['content-type']).toBe('application/json');
    expect(request.init.body).toBe('{"email":"a@b.co","password":"x"}');
  });

  it('sends FormData without a content type so the browser sets the boundary', async () => {
    let headers: Record<string, string> = {};
    const fetcher: typeof globalThis.fetch = async (_, init) => {
      headers = init?.headers as Record<string, string>;
      return new Response(JSON.stringify({ redirect: '/settings/profile' }), { status: 200 });
    };
    const form = new FormData();
    form.set('name', 'Ada');
    await submit('PUT', '/api/settings/profile', form, fetcher);
    expect(headers['content-type']).toBeUndefined();
  });

  it('turns a 422 into the field errors', async () => {
    const outcome = await submit(
      'POST',
      '/api/register',
      {},
      answering(422, {
        message: 'A name is required. (and 1 more error)',
        errors: { name: ['A name is required.'], email: ['An email is required.'] },
      }),
    );
    expect(outcome).toEqual({
      ok: false,
      status: 422,
      message: 'A name is required. (and 1 more error)',
      errors: { name: ['A name is required.'], email: ['An email is required.'] },
    });
    expect(firstErrors({ name: ['a', 'b'], email: [] })).toEqual({ name: 'a' });
  });

  it('reports an unreadable failure with a generic sentence', async () => {
    const fetcher: typeof globalThis.fetch = async () => new Response('gateway', { status: 502 });
    const outcome = await submit('DELETE', '/api/settings/account', null, fetcher);
    expect(outcome).toMatchObject({
      ok: false,
      status: 502,
      message: 'The server is unavailable right now.',
    });
  });
});
