import { describe, expect, it } from 'vitest';
import { APP_NAME, COOKIE_PREFIX, FLASH_COOKIE, slug } from './app';

describe('app identity', () => {
  it('derives the cookie prefix the API derives', () => {
    // Mirrors the cases of `config::tests::names_become_cookie_prefixes`.
    expect(slug('My App!')).toBe('my_app');
    expect(slug('  Some-Product 2 ')).toBe('some_product_2');
    expect(slug('Étoile')).toBe('toile');
    expect(slug('!!!')).toBe('app');
    expect(slug('Starter')).toBe('starter');
  });

  it('names the flash cookie after the app', () => {
    expect(COOKIE_PREFIX).toBe(slug(APP_NAME));
    expect(FLASH_COOKIE).toBe(`${slug(APP_NAME)}_flash`);
  });
});
