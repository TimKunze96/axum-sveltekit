import type { PageServerLoad } from './$types';

// The first page behind the login. Load its data through `apiGet` from
// $lib/server/api once there is some.
export const load: PageServerLoad = () => ({
  breadcrumbs: [{ title: 'Dashboard', href: '/dashboard' }],
});
