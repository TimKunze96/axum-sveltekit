import { describe, expect, it } from 'vitest';
import { apiGet } from './api';

function fetchAnswering(status: number, body: unknown): typeof globalThis.fetch {
  return async () => new Response(JSON.stringify(body), { status });
}

describe('apiGet', () => {
  it('returns the parsed payload of a 2xx answer', async () => {
    await expect(apiGet(fetchAnswering(200, { status: 'ok' }), '/api/health')).resolves.toEqual({
      status: 'ok',
    });
  });

  it('redirects a 401 into the login', async () => {
    await expect(apiGet(fetchAnswering(401, {}), '/api/dashboard')).rejects.toMatchObject({
      status: 303,
      location: '/login',
    });
  });

  it("raises the API's status and message for other failures", async () => {
    await expect(
      apiGet(fetchAnswering(404, { message: 'Not Found' }), '/api/nope'),
    ).rejects.toMatchObject({ status: 404, body: { message: 'Not Found' } });
  });

  it('falls back to a generic sentence when the error body is not JSON', async () => {
    const fetch: typeof globalThis.fetch = async () => new Response('gateway', { status: 502 });
    await expect(apiGet(fetch, '/api/health')).rejects.toMatchObject({
      status: 502,
      body: { message: 'The server is unavailable right now.' },
    });
  });
});
