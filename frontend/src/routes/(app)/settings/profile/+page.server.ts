import type { PageServerLoad } from './$types';

export const load: PageServerLoad = () => ({
  breadcrumbs: [{ title: 'Profile settings', href: '/settings/profile' }],
});
