import { describe, expect, it } from 'vitest';
import { APP_NAME } from './app';
import { pageTitle } from './title';

describe('pageTitle', () => {
  it('suffixes the app name', () => {
    expect(pageTitle('Profile settings')).toBe(`Profile settings - ${APP_NAME}`);
    expect(pageTitle()).toBe(APP_NAME);
  });
});
