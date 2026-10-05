import { defineEnvVars } from '@sveltejs/kit/env';

// Both optional: an unset variable stays undefined so the readers apply
// their defaults (hooks.server.ts, #lib/app.ts).
export const variables = defineEnvVars({
  AXUM_URL: { schema: (input) => input },
  PUBLIC_APP_NAME: { public: true, schema: (input) => input },
});
