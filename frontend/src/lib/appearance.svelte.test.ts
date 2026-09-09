import { afterEach, describe, expect, it } from 'vitest';
import {
  getAppearance,
  getResolvedAppearance,
  initializeTheme,
  isAppearance,
  updateAppearance,
} from './appearance.svelte';

afterEach(() => {
  localStorage.removeItem('appearance');
  document.cookie = 'appearance=;path=/;max-age=0';
  document.documentElement.classList.remove('dark');
});

describe('appearance', () => {
  it('recognises the three choices', () => {
    expect(isAppearance('dark')).toBe(true);
    expect(isAppearance('system')).toBe(true);
    expect(isAppearance('blue')).toBe(false);
    expect(isAppearance(null)).toBe(false);
  });

  it('stores the choice in localStorage and the cookie and toggles the class', () => {
    updateAppearance('dark');
    expect(getAppearance()).toBe('dark');
    expect(getResolvedAppearance()).toBe('dark');
    expect(localStorage.getItem('appearance')).toBe('dark');
    expect(document.cookie).toContain('appearance=dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);

    updateAppearance('light');
    expect(document.documentElement.classList.contains('dark')).toBe(false);
    expect(getResolvedAppearance()).toBe('light');
  });

  it('starts from the saved preference', () => {
    localStorage.setItem('appearance', 'dark');
    const stop = initializeTheme();
    expect(getAppearance()).toBe('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    stop();
  });

  it("falls back to the server's cookie choice when nothing is stored", () => {
    const stop = initializeTheme('dark');
    expect(getAppearance()).toBe('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
    stop();
    expect(localStorage.getItem('appearance')).toBeNull();
  });
});
