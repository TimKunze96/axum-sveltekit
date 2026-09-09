// The signed-in account, resolved once per request by hooks.server.ts
// (which also applies the route guard) and handed to the pages as
// `locals.user`.
import type { User } from '$lib/api';

export async function currentUser(fetch: typeof globalThis.fetch): Promise<User | null> {
  // A 401 is a guest; an unreachable API renders as guest too rather
  // than failing every page.
  return fetch('/api/me')
    .then((response) => (response.ok ? (response.json() as Promise<User>) : null))
    .catch(() => null);
}
