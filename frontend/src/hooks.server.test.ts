import { describe, expect, it, vi } from 'vitest';
import { handle, routeGuard } from './hooks.server';

const user = { id: 1, name: 'Ada', email: 'ada@example.com', is_admin: false };

describe('routeGuard', () => {
  it('keeps guests out of the app and members out of the guest pages', () => {
    expect(routeGuard('/(app)/dashboard', null)).toBe('/login');
    expect(routeGuard('/(app)/settings/profile', null)).toBe('/login');
    expect(routeGuard('/(app)/dashboard', user)).toBeNull();
    expect(routeGuard('/(guest)/login', user)).toBe('/dashboard');
    expect(routeGuard('/(guest)/register', null)).toBeNull();
    expect(routeGuard('/(guest)/reset-password', user)).toBe('/dashboard');
    expect(routeGuard('/', null)).toBeNull();
    expect(routeGuard('/', user)).toBeNull();
    expect(routeGuard(null, null)).toBeNull();
  });
});

/** An event whose `/api/me` answers the account, or a 401 for a guest. */
function event(
  routeId: string | null,
  account: typeof user | null,
  cookies: Record<string, string> = {},
) {
  return {
    route: { id: routeId },
    cookies: { get: (name: string) => cookies[name] },
    fetch: (async () =>
      account
        ? new Response(JSON.stringify(account), { status: 200 })
        : new Response(JSON.stringify({ message: 'Unauthenticated.' }), {
            status: 401,
          })) as typeof fetch,
    locals: {} as Record<string, unknown>,
  };
}

async function run(
  routeId: string | null,
  account: typeof user | null,
  cookies?: Record<string, string>,
) {
  const resolve = vi.fn(async () => new Response('page'));
  const ev = event(routeId, account, cookies);
  const outcome = await (
    handle as unknown as (input: { event: unknown; resolve: unknown }) => Promise<Response>
  )({ event: ev, resolve }).then(
    (response) => ({ response, redirect: null as null | { status: number; location: string } }),
    (thrown: { status: number; location: string }) => ({ response: null, redirect: thrown }),
  );
  return { ...outcome, resolve, locals: ev.locals };
}

describe('handle', () => {
  it('resolves the session into locals and serves the page', async () => {
    const { response, resolve, locals } = await run('/(app)/dashboard', user);
    expect(response?.status).toBe(200);
    expect(resolve).toHaveBeenCalledOnce();
    expect(locals.user).toEqual(user);
    expect(locals.appearance).toBe('system');
  });

  it('redirects a guest out of the app on every request', async () => {
    const { redirect, resolve, locals } = await run('/(app)/settings/password', null);
    expect(redirect).toMatchObject({ status: 303, location: '/login' });
    expect(resolve).not.toHaveBeenCalled();
    expect(locals.user).toBeNull();
  });

  it('redirects a signed-in visitor away from the login', async () => {
    const { redirect } = await run('/(guest)/login', user);
    expect(redirect).toMatchObject({ status: 303, location: '/dashboard' });
  });

  it('treats an unreachable API as a guest and reads the appearance cookie', async () => {
    const resolve = vi.fn(async () => new Response('page'));
    const ev = event('/', null, { appearance: 'dark' });
    ev.fetch = (async () => {
      throw new Error('refused');
    }) as typeof fetch;
    await (handle as unknown as (input: { event: unknown; resolve: unknown }) => Promise<Response>)(
      { event: ev, resolve },
    );
    expect(ev.locals.user).toBeNull();
    expect(ev.locals.appearance).toBe('dark');
    expect(resolve).toHaveBeenCalledOnce();
  });

  it('skips the session lookup for requests outside the routes', async () => {
    const fetcher = vi.fn();
    const resolve = vi.fn(async () => new Response('asset'));
    const ev = { ...event(null, user), fetch: fetcher as unknown as typeof fetch };
    await (handle as unknown as (input: { event: unknown; resolve: unknown }) => Promise<Response>)(
      { event: ev, resolve },
    );
    expect(fetcher).not.toHaveBeenCalled();
    expect(ev.locals.user).toBeNull();
  });
});
