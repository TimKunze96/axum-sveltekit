import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { FORGOT_PASSWORD_PATH } from '$lib/login';

// The mailed link carries the token and the email; without them there
// is nothing to reset.
export const load: PageServerLoad = ({ url }) => {
  const token = url.searchParams.get('token');
  const email = url.searchParams.get('email');
  if (!token || !email) {
    redirect(303, FORGOT_PASSWORD_PATH);
  }
  return { token, email };
};
