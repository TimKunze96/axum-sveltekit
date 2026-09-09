import type { PageServerLoad } from './$types';

export const load: PageServerLoad = () => ({
  breadcrumbs: [{ title: 'Password settings', href: '/settings/password' }],
});
