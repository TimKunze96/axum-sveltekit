import { describe, expect, it } from 'vitest';
import { parseFlash } from './flash';

describe('parseFlash', () => {
  it('decodes the percent-encoded JSON the API sets', () => {
    const raw = encodeURIComponent(
      JSON.stringify({ title: 'Signed in', body: 'Welcome back, Ada!', type: 'success' }),
    );
    expect(parseFlash(raw)).toEqual({
      title: 'Signed in',
      body: 'Welcome back, Ada!',
      type: 'success',
    });
  });

  it('keeps a missing body as null', () => {
    const raw = encodeURIComponent(
      JSON.stringify({ title: 'Your account has been deleted', body: null, type: 'success' }),
    );
    expect(parseFlash(raw)).toEqual({
      title: 'Your account has been deleted',
      body: null,
      type: 'success',
    });
  });

  it('ignores absent, malformed or foreign cookies', () => {
    expect(parseFlash(undefined)).toBeNull();
    expect(parseFlash('')).toBeNull();
    expect(parseFlash('%7Bnot-json')).toBeNull();
    expect(parseFlash(encodeURIComponent(JSON.stringify({ type: 'success' })))).toBeNull();
    expect(
      parseFlash(encodeURIComponent(JSON.stringify({ title: 'x', body: null, type: 'info' }))),
    ).toBeNull();
  });
});
