import type { Notification } from '$lib/api';

export { FLASH_COOKIE } from '$lib/app';

/**
 * Decodes the flash cookie's percent-encoded JSON. Anything malformed
 * is ignored rather than breaking the page.
 */
export function parseFlash(raw: string | undefined): Notification | null {
  if (!raw) return null;
  try {
    const value: unknown = JSON.parse(decodeURIComponent(raw));
    if (
      typeof value === 'object' &&
      value !== null &&
      typeof (value as Notification).title === 'string' &&
      ((value as Notification).type === 'success' || (value as Notification).type === 'error')
    ) {
      const { title, body, type } = value as Notification;
      return { title, body: typeof body === 'string' ? body : null, type };
    }
  } catch {
    // Not ours to show.
  }
  return null;
}
