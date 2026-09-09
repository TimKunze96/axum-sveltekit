import type { PageServerLoad } from './$types';

export const load: PageServerLoad = () => ({
  breadcrumbs: [{ title: 'Appearance settings', href: '/settings/appearance' }],
});
