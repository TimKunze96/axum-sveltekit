<h1 align="center">Starter</h1>
<p align="center">
  An Axum + SvelteKit application template with accounts, sessions and settings ready to go.
</p>
<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-a3e635" alt="MIT license"></a>
  <img src="https://img.shields.io/badge/Rust-2024-000000?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/Axum-0.8-000000" alt="Axum">
  <img src="https://img.shields.io/badge/SvelteKit-SSR-ff3e00?logo=svelte&logoColor=white" alt="SvelteKit">
  <img src="https://img.shields.io/badge/Postgres-17-4169e1?logo=postgresql&logoColor=white" alt="Postgres 17">
</p>

A Rust JSON API (Axum, Postgres via sqlx) under `/api` and a SvelteKit
frontend (SSR, adapter-node, shadcn-svelte, Tailwind 4) owning every
other URL. Everything runs on one shared origin so the session cookie
stays first-party: Caddy (or the Vite dev proxy) sends `/api/*` to Axum
and everything else to SvelteKit.

## What is in the box

- Email and password accounts behind one `Auth` facade (`src/auth`):
  register, log in, log out, current user, forgot and reset password by
  mail, profile, password change, account deletion. Argon2id hashes,
  server-side sessions and reset links keyed by the SHA-256 of a random
  token.
- A typed configuration (`src/config.rs`) read from `.env`, including the
  app name that the cookies and the frontend take theirs from, and the
  mailer (SMTP, or the log when no host is set).
- TypeScript types generated from the Rust wire types by ts-rs
  (`frontend/src/lib/api/types`, written by `cargo test`, checked in CI).
- The SvelteKit shell: welcome page, login, registration, forgot and
  reset password, a sidebar app layout with a bare dashboard, settings
  (profile, password, appearance), light and dark mode, toasts carried
  across redirects.
- Tests on both sides: Rust unit and integration suites against a real
  Postgres, vitest in node and headless chromium, Playwright end to end.
- GitHub Actions for CI (fmt, clippy, tests, type check, lint, format,
  Docker builds), Dependabot with auto-merge for non-major bumps, a
  weekly dependency audit, and a deploy workflow that publishes images to
  GHCR and rolls a box over ssh.
- Docker: a one-command stack, a hot-reloading dev profile for both the
  API and the frontend, and production overrides with Caddy holding TLS.

## Make it yours

Copy `.env.example` to `.env` and set `APP_NAME` and `PUBLIC_APP_NAME`
(the display name; the API derives the cookie names from it and the
frontend reads the public one). The crate is called `starter` in
`Cargo.toml`, the database and its user default to `starter` in
`docker-compose.yml`; both are plain settings, change them when you
like. Then replace the welcome page and the dashboard.

## Run it

Native development, with Postgres from the compose file:

```sh
cp .env.example .env
docker compose up -d postgres      # host port 5436
cargo run                          # API on 127.0.0.1:3200, migrates on start
cd frontend && npm install && npm run dev   # http://localhost:8787
```

The Vite dev server proxies `/api` to the API, so the browser only ever
talks to :8787. The one `.env` in the repository root serves
both sides (the frontend's `vite.config.ts` points `envDir` at it).

The one-command stack, everything in containers on the same origin:

```sh
docker compose up --build          # http://localhost:8787
```

The hot-reloading variant of the stack: the API recompiles and restarts
on every change under `src/`, `migrations/` or `Cargo.toml` (watchexec
inside the container, first build takes a few minutes), the frontend is
the Vite dev server. Stop the full stack first, both want :8787.

```sh
docker compose --profile dev up --build frontend-dev
```

`ADMIN_EMAILS` in `.env` lists the accounts that are site admins,
applied on registration and on every login. Password reset mails go to
the API log until `SMTP_HOST` is set; the link is in the log output.

## Configuration

Every setting has a working local default and a documented variable in
`.env.example`; `src/config.rs` reads them into one `Config` struct and
nothing else in the crate touches the environment. `APP_ENV=local`
marks a developer machine (cookies without `Secure`); anything else is
production.

## Tests

```sh
cargo test                       # unit tests plus the integration binary
cargo clippy --all-targets       # zero warnings
cd frontend && npm test          # vitest (node and headless chromium)
cd frontend && npm run test:e2e  # playwright against the running stack
```

The integration suites run against the `starter_test` database, created
automatically on the same Postgres, single-threaded (see
`.cargo/config.toml`). One suite at a time: `cargo test <module>::`.
`cargo test` also rewrites `frontend/src/lib/api/types`; commit the
result, CI checks that it matches.

## Deploy

On a machine with Docker and the domain's DNS pointing at it: create
`/opt/<repo>`, write `.env` with `APP_ENV=production`, `APP_NAME`,
`PUBLIC_APP_NAME`, `POSTGRES_PASSWORD`, `APP_URL` (the public https
origin) and `APP_HOST` (its host), then:

```sh
docker compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

The production stack runs the api and frontend images published to GHCR
(built by `.github/workflows/deploy.yml`, tagged with the commit sha)
rather than building on the box. Deploys are automatic: once CI is green
on `main` the Deploy workflow builds and publishes both images, then
rolls the box over ssh (`deploy/deploy.sh <sha>` pulls, recreates the
containers and waits for `/api/health`). Running the workflow by hand
with an older sha rolls back. The workflow needs the `DEPLOY_HOST`,
`DEPLOY_USER`, `DEPLOY_SSH_KEY` and `DEPLOY_KNOWN_HOSTS` repository
secrets, and a `production` environment.

Caddy (`deploy/Caddyfile`) holds the domain's certificate and sends HSTS;
`GET /api/health` answers 200 while the database does. Back the
`postgres-data` volume up with
`docker compose exec postgres pg_dump -U starter starter`.

## License

MIT, see `LICENSE`.
