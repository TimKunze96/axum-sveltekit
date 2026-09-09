// The appearance choice as the server sees it (no runes here: the
// server hooks import this too).
import type { Appearance } from '$lib/types';

export const APPEARANCE_COOKIE = 'appearance';

const APPEARANCES: ReadonlySet<string> = new Set<Appearance>(['light', 'dark', 'system']);

export function isAppearance(value: unknown): value is Appearance {
  return typeof value === 'string' && APPEARANCES.has(value);
}
