import { error, redirect } from '@sveltejs/kit';
import { LOGIN_PATH } from '$lib/login';

/** The API's fallback sentence when its error body carries no message. */
const UNAVAILABLE_MESSAGE = 'The server is unavailable right now.';

/**
 * Loads a JSON payload from the Axum API for an SSR load function,
 * translating the API's contract into SvelteKit control flow: 401 sends
 * the guest to sign in, other non-2xx statuses become error pages
 * carrying the API's message.
 */
export async function apiGet<T>(fetch: typeof globalThis.fetch, path: string): Promise<T> {
  const response = await fetch(path);

  if (response.status === 401) {
    redirect(303, LOGIN_PATH);
  }
  if (!response.ok) {
    const body: { message?: string } = await response.json().catch(() => ({}));
    error(response.status, body.message ?? UNAVAILABLE_MESSAGE);
  }

  return response.json();
}
