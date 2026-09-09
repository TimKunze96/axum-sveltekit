// The `$env/dynamic/public` stand-in for vitest's browser project: the
// real module reads SvelteKit's page bootstrap, which never runs there.
// Aliased in vite.config.ts.
export const env: Record<string, string | undefined> = { PUBLIC_APP_NAME: 'Starter' };
