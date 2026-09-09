import { describe, expect, it } from 'vitest';
import { isCurrentOrParentUrl, isCurrentUrl } from './current-url';

describe('current url', () => {
  it('matches paths exactly', () => {
    expect(isCurrentUrl('/dashboard', '/dashboard')).toBe(true);
    expect(isCurrentUrl('/dashboard', '/dashboard/more')).toBe(false);
    expect(isCurrentUrl('http://localhost:8787/dashboard', '/dashboard')).toBe(true);
    expect(isCurrentUrl('http://[bad', '/dashboard')).toBe(false);
  });

  it('matches parents by prefix', () => {
    expect(isCurrentOrParentUrl('/settings/profile', '/settings/profile/extra')).toBe(true);
    expect(isCurrentOrParentUrl('/settings/appearance', '/settings/profile')).toBe(false);
  });
});
