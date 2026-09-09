import { describe, expect, it, vi } from 'vitest';
import { FLASH_COOKIE } from '$lib/app';
import { load } from './+layout.server';

const user = { id: 1, name: 'Ada', email: 'ada@example.com', is_admin: false };

async function run(cookies: Record<string, string>) {
  const jar = { ...cookies };
  const deleted = vi.fn((name: string) => {
    delete jar[name];
  });
  const event = {
    cookies: { get: (name: string) => jar[name], delete: deleted },
    locals: { appearance: 'dark', user },
  };
  const data = await (load as unknown as (e: unknown) => Promise<Record<string, unknown>>)(event);
  return { data, deleted };
}

describe('root layout load', () => {
  it('reads the flash cookie into the notification and clears it', async () => {
    const flash = encodeURIComponent(
      JSON.stringify({ title: 'Profile updated', body: null, type: 'success' }),
    );
    const { data, deleted } = await run({ [FLASH_COOKIE]: flash });

    expect(data.notification).toEqual({ title: 'Profile updated', body: null, type: 'success' });
    expect(deleted).toHaveBeenCalledWith(FLASH_COOKIE, { path: '/' });
    expect(data.user).toEqual(user);
    expect(data.appearance).toBe('dark');
  });

  it('leaves the cookie jar alone on a plain page load', async () => {
    const { data, deleted } = await run({});

    expect(data.notification).toBeNull();
    expect(deleted).not.toHaveBeenCalled();
  });

  it('drops a malformed flash but still clears the cookie', async () => {
    const { data, deleted } = await run({ [FLASH_COOKIE]: 'not%20json' });

    expect(data.notification).toBeNull();
    expect(deleted).toHaveBeenCalledOnce();
  });

  it('opens the sidebar unless its cookie says closed', async () => {
    expect((await run({})).data.sidebarOpen).toBe(true);
    expect((await run({ sidebar_state: 'true' })).data.sidebarOpen).toBe(true);
    expect((await run({ sidebar_state: 'false' })).data.sidebarOpen).toBe(false);
  });
});
