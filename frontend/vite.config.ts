import { fileURLToPath } from 'node:url';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vitest/config';
import type { ProxyOptions } from 'vite';
import { playwright } from '@vitest/browser-playwright';
import { sveltekit } from '@sveltejs/kit/vite';

// Dev-only shared origin (Caddy plays this role in production): `/api`
// goes to Axum, every other URL is a SvelteKit page, cookies never cross
// an origin. Never point the browser at Axum's port directly. Match the
// API's BIND_ADDR with AXUM_URL when the default port is taken (the SSR
// fetch rewrite in hooks.server.ts reads the same variable).
const AXUM_DEV_URL = process.env.AXUM_URL ?? 'http://127.0.0.1:3200';

/** The one prefix Axum owns; deploy/Caddyfile* carry the same rule. */
const API_PREFIX = '/api';

// The one development port: the composed stack (docker-compose.yml)
// serves on it too, so the app is always at http://localhost:8787.
const DEV_PORT = 8787;

// Inside the frontend-dev container the source is a bind mount and file
// events don't cross Docker's VM boundary, so hot reload silently dies.
// VITE_POLL_WATCH (set by docker-compose) switches the watcher to
// polling there; native dev keeps event-based watching.
const watch = process.env.VITE_POLL_WATCH ? { usePolling: true, interval: 300 } : undefined;

const proxy: Record<string, ProxyOptions> = { [API_PREFIX]: { target: AXUM_DEV_URL } };

export default defineConfig({
  // One .env for the whole repository: the API reads it through
  // src/config.rs, the frontend through $env (PUBLIC_APP_NAME, AXUM_URL).
  envDir: '..',
  server: { port: DEV_PORT, strictPort: true, proxy, watch },
  plugins: [tailwindcss(), sveltekit()],
  test: {
    expect: { requireAssertions: true },
    projects: [
      {
        extends: './vite.config.ts',
        resolve: {
          // Component tests render without SvelteKit's page bootstrap,
          // which is where the runtime public env comes from.
          alias: {
            '$env/dynamic/public': fileURLToPath(
              new URL('./src/lib/test/env-public.ts', import.meta.url),
            ),
          },
        },
        test: {
          name: 'client',
          browser: {
            enabled: true,
            provider: playwright(),
            instances: [{ browser: 'chromium', headless: true }],
          },
          include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
          exclude: ['src/lib/server/**'],
        },
      },
      {
        extends: './vite.config.ts',
        test: {
          name: 'server',
          environment: 'node',
          include: ['src/**/*.{test,spec}.{js,ts}'],
          exclude: ['src/**/*.svelte.{test,spec}.{js,ts}'],
        },
      },
    ],
  },
});
