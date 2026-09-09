import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// A redirect to the first settings page.
export const load: PageServerLoad = () => {
  redirect(302, '/settings/profile');
};
