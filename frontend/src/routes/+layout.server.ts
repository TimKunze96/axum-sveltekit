import type { LayoutServerLoad } from './$types';
import { FLASH_COOKIE, parseFlash } from '$lib/flash';

/** The shadcn sidebar's own cookie. */
const SIDEBAR_COOKIE = 'sidebar_state';

export const load: LayoutServerLoad = async ({ cookies, locals }) => {
  // A toast set by the API on its redirect shows once, then the cookie goes.
  const notification = parseFlash(cookies.get(FLASH_COOKIE));
  if (cookies.get(FLASH_COOKIE) !== undefined) {
    cookies.delete(FLASH_COOKIE, { path: '/' });
  }
  const sidebar = cookies.get(SIDEBAR_COOKIE);
  return {
    // Resolved by the server hook, which also guards the route groups.
    user: locals.user,
    notification,
    appearance: locals.appearance,
    sidebarOpen: sidebar === undefined || sidebar === 'true',
  };
};
