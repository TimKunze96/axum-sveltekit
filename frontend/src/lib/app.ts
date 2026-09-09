// The app's identity, from the one root .env (PUBLIC_APP_NAME): the
// display name and the cookie prefix derived from it. The API derives
// the same prefix from APP_NAME in src/config.rs; keep both in step.
import { env } from '$env/dynamic/public';

export const DEFAULT_APP_NAME = 'Starter';

export const APP_NAME: string = env.PUBLIC_APP_NAME?.trim() || DEFAULT_APP_NAME;

/**
 * Lowercase letters and digits, every other run of characters a single
 * underscore, none at the ends: `"My App!"` is `my_app`.
 */
export function slug(name: string): string {
  const normalized = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
  return normalized || 'app';
}

export const COOKIE_PREFIX = slug(APP_NAME);

/** The cookie the API sets on a redirect that carries a toast. */
export const FLASH_COOKIE = `${COOKIE_PREFIX}_flash`;
