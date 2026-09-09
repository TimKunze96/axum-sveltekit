import { describe, expect, it } from 'vitest';
import { currentUser } from './current-user';

const user = { id: 1, name: 'Ada', email: 'ada@example.com', is_admin: false };

describe('currentUser', () => {
  it('answers the account the API knows', async () => {
    const fetch: typeof globalThis.fetch = async (input) => {
      expect(String(input)).toBe('/api/me');
      return new Response(JSON.stringify(user), { status: 200 });
    };
    await expect(currentUser(fetch)).resolves.toEqual(user);
  });

  it('is a guest on a 401, a failure or an unreachable API', async () => {
    const guest: typeof globalThis.fetch = async () =>
      new Response(JSON.stringify({ message: 'Unauthenticated.' }), { status: 401 });
    await expect(currentUser(guest)).resolves.toBeNull();
    const failing: typeof globalThis.fetch = async () => new Response('', { status: 503 });
    await expect(currentUser(failing)).resolves.toBeNull();
    const unreachable: typeof globalThis.fetch = async () => {
      throw new Error('refused');
    };
    await expect(currentUser(unreachable)).resolves.toBeNull();
  });
});
