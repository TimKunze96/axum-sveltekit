import type { PageServerLoad } from './$types';

// A server load, so a client-side navigation here makes the request the
// guard in hooks.server.ts inspects.
export const load: PageServerLoad = () => ({});
