// The light/dark/system choice lives in localStorage for the browser and
// in the `appearance` cookie for the server render, and toggles the
// `dark` class on the document.
import { APPEARANCE_COOKIE, isAppearance } from '$lib/appearance-cookie';
import type { Appearance, ResolvedAppearance } from '$lib/types';

export { APPEARANCE_COOKIE, isAppearance };
export const APPEARANCE_STORAGE_KEY = 'appearance';

/** The cookie outlives the localStorage copy by design: a year. */
const COOKIE_DAYS = 365;

function prefersDark(): boolean {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;
}

export function updateTheme(value: Appearance): void {
  if (typeof document === 'undefined') return;
  const dark = value === 'system' ? prefersDark() : value === 'dark';
  document.documentElement.classList.toggle('dark', dark);
}

function storedAppearance(): Appearance | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const stored = localStorage.getItem(APPEARANCE_STORAGE_KEY);
    return isAppearance(stored) ? stored : null;
  } catch {
    return null;
  }
}

let appearance = $state<Appearance>('system');

export function getAppearance(): Appearance {
  return appearance;
}

export function getResolvedAppearance(): ResolvedAppearance {
  if (appearance === 'system') return prefersDark() ? 'dark' : 'light';
  return appearance;
}

export function updateAppearance(value: Appearance): void {
  appearance = value;
  try {
    localStorage.setItem(APPEARANCE_STORAGE_KEY, value);
  } catch {
    // Private mode; the cookie still carries it.
  }
  const maxAge = COOKIE_DAYS * 24 * 60 * 60;
  document.cookie = `${APPEARANCE_COOKIE}=${value};path=/;max-age=${maxAge};SameSite=Lax`;
  updateTheme(value);
}

/**
 * Applies the saved preference on page load and follows the system
 * theme while the preference is `system`. `fallback` is the cookie's
 * choice as the server rendered it, for a browser whose localStorage
 * has nothing yet. Returns the teardown.
 */
export function initializeTheme(fallback: Appearance = 'system'): () => void {
  if (typeof window === 'undefined') return () => {};
  appearance = storedAppearance() ?? fallback;
  updateTheme(appearance);

  const query = window.matchMedia('(prefers-color-scheme: dark)');
  const follow = () => updateTheme(storedAppearance() ?? fallback);
  query.addEventListener('change', follow);
  return () => query.removeEventListener('change', follow);
}
