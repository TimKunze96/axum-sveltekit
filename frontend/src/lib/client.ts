// Mutations against the Axum API (every action lives under /api) from
// the browser: the API answers a fetch client with `{redirect}` plus the
// flash cookie, or a 422 in the validation shape (see src/server/api.rs
// in the Rust crate).
import { goto, invalidateAll } from '$app/navigation';
import type { RedirectAnswer, ValidationErrors } from '$lib/api';

export interface Failure {
  ok: false;
  status: number;
  message: string;
  errors: Record<string, string[]>;
}

export interface Success {
  ok: true;
  redirect: string;
}

export type Outcome = Success | Failure;

/** The first message per field, for the form's inline errors. */
export function firstErrors(errors: Record<string, string[]>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(errors).flatMap(([field, messages]) =>
      messages.length ? [[field, messages[0]]] : [],
    ),
  );
}

/**
 * Sends a mutation. A JSON object body is posted as JSON, a FormData as
 * multipart (files); a null body sends nothing.
 */
export async function submit(
  method: 'POST' | 'PUT' | 'PATCH' | 'DELETE',
  action: string,
  body: Record<string, unknown> | FormData | null = null,
  fetcher: typeof globalThis.fetch = globalThis.fetch,
): Promise<Outcome> {
  const headers: Record<string, string> = { accept: 'application/json' };
  let payload: BodyInit | undefined;
  if (body instanceof FormData) {
    payload = body;
  } else if (body !== null) {
    headers['content-type'] = 'application/json';
    payload = JSON.stringify(body);
  }
  const response = await fetcher(action, { method, headers, body: payload });
  const data: Partial<RedirectAnswer & ValidationErrors> = await response.json().catch(() => ({}));
  if (response.ok) {
    return { ok: true, redirect: data.redirect ?? location.pathname };
  }
  return {
    ok: false,
    status: response.status,
    message: data.message ?? 'The server is unavailable right now.',
    errors: data.errors ?? {},
  };
}

/**
 * Follows a successful mutation: navigates to where the API sent us,
 * reloading every load so the toast and the fresh data arrive; staying
 * on the same page only reloads.
 */
export async function follow(outcome: Success): Promise<void> {
  const target = new URL(outcome.redirect, location.origin);
  if (target.pathname === location.pathname && target.search === location.search) {
    await invalidateAll();
  } else {
    await goto(target.pathname + target.search, { invalidateAll: true });
  }
}
