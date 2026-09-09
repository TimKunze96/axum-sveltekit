import { redirect, type Handle, type HandleFetch } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
// Relative on purpose: a `$lib` import from the server hooks breaks
// SvelteKit's hooks loading inside vitest's browser mode (a console
// TypeError on every client test run).
import type { User } from './lib/api';
import { APPEARANCE_COOKIE, isAppearance } from './lib/appearance-cookie';
import { HOME_PATH, LOGIN_PATH } from './lib/login';
import { currentUser } from './lib/server/current-user';

/** Where server-side loads reach Axum; the compose stack sets it. */
const AXUM_DEFAULT_URL = 'http://127.0.0.1:3200';

/** The route groups the guard applies to (see src/routes). */
const APP_GROUP = '/(app)';
const GUEST_GROUP = '/(guest)';

/**
 * Where a request must be redirected instead of served: guests out of
 * the app shell, signed-in visitors out of the login pages. Null lets
 * the request through.
 */
export function routeGuard(routeId: string | null, user: User | null): string | null {
  if (routeId === null) return null;
  if (routeId.startsWith(APP_GROUP) && user === null) return LOGIN_PATH;
  if (routeId.startsWith(GUEST_GROUP) && user !== null) return HOME_PATH;
  return null;
}

// Runs on every request, page loads and the data requests behind
// client-side navigation alike, which is why the session is resolved
// and the guard applied here rather than in a layout load (a layout
// load does not re-run for every navigation, see the SvelteKit docs on
// load and authentication). The pages read `locals.user`.
export const handle: Handle = async ({ event, resolve }) => {
  const cookie = event.cookies.get(APPEARANCE_COOKIE);
  const appearance = isAppearance(cookie) ? cookie : 'system';
  event.locals.appearance = appearance;

  event.locals.user = event.route.id === null ? null : await currentUser(event.fetch);
  const destination = routeGuard(event.route.id, event.locals.user);
  if (destination !== null) {
    redirect(303, destination);
  }

  // The cookie's choice reaches the html shell, which carries the dark
  // class for a dark choice and lets an inline script settle "system"
  // before the first paint.
  return resolve(event, {
    transformPageChunk: ({ html }) =>
      html
        .replace('%app.appearance%', appearance)
        .replace('%app.htmlclass%', appearance === 'dark' ? 'dark' : ''),
  });
};

// Server-side loads reach Axum directly (the browser goes through the
// shared-origin proxy instead). The cookie forward is what keeps SSR
// authenticated: without it every render is a guest.
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
  const url = new URL(request.url);
  if (url.origin === event.url.origin && url.pathname.startsWith('/api')) {
    const axum = env.AXUM_URL ?? AXUM_DEFAULT_URL;
    request = new Request(new URL(url.pathname + url.search, axum), request);
    const cookie = event.request.headers.get('cookie');
    if (cookie) {
      request.headers.set('cookie', cookie);
    }
  }

  return fetch(request);
};
