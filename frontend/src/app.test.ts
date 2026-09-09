import { existsSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// The shell links the icon set and every file is shipped from the static
// directory.
const shell = readFileSync(new URL('./app.html', import.meta.url), 'utf8');

describe('app shell', () => {
  it('links the favicon set', () => {
    expect(shell).toContain('<link rel="icon" href="/favicon.ico" sizes="48x48" />');
    expect(shell).toContain('<link rel="icon" href="/favicon.svg" type="image/svg+xml" />');
    expect(shell).toContain(
      '<link rel="icon" href="/icon-192.png" type="image/png" sizes="192x192" />',
    );
    expect(shell).toContain('<link rel="apple-touch-icon" href="/apple-touch-icon.png" />');
  });

  it.each(['favicon.ico', 'favicon.svg', 'icon-192.png', 'icon-512.png', 'apple-touch-icon.png'])(
    'ships %s',
    (icon) => {
      expect(existsSync(new URL(`../static/${icon}`, import.meta.url))).toBe(true);
    },
  );

  it('carries the appearance placeholders the server hook fills', () => {
    expect(shell).toContain('class="%app.htmlclass%"');
    expect(shell).toContain("const appearance = '%app.appearance%';");
  });
});
