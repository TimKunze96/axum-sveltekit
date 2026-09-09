import type { LayoutServerLoad } from './$types';
import type { User } from '$lib/types';

// The guard lives in hooks.server.ts: a guest never reaches this load.
// This only narrows the type for the pages of the shell. Every page
// under (app) keeps a +page.server.ts so a client-side navigation still
// makes the server request the hook inspects.
export const load: LayoutServerLoad = async ({ parent }) => {
  const { user } = await parent();
  return { user: user as User };
};
