// `$app/env/public` reads its values from the global SvelteKit's page
// bootstrap sets in dev, which never runs in vitest's browser project.
// Loaded as a setup file in vite.config.ts.
Object.assign(globalThis, { __sveltekit_dev: { env: { PUBLIC_APP_NAME: 'Starter' } } });
