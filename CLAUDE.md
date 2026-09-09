# Starter

An application template: a **Rust API (Axum + Postgres via sqlx)**
serving pure JSON under `/api`, and a **SvelteKit frontend (SSR,
adapter-node) in `frontend/`** owning every other URL. One shared origin
(the Vite dev proxy locally, Caddy in production) routes `/api/*` to
Axum and everything else to SvelteKit, so cookies stay same-origin.
Never point the browser at Axum's port directly.

## Testing rules

- Every increment lands **with its tests** in the same commit(s).
- Tests must be **precise**: assert exact JSON key sets (sorted-keys
  comparisons at every nesting level), exact error messages and status
  codes, exact orderings, not just spot values.
- Behavior tests drive the real router (axum `oneshot` through
  `common::send` / `send_to` / `send_json`); external services get local
  mock servers on ephemeral ports, never stubs of our own code.
- Every test takes a fresh pool and router from `common::pool()` /
  `common::app()`. Never cache either across tests: each `#[tokio::test]`
  has its own runtime, and a connection from a finished runtime hangs
  until the acquire timeout.
- Integration suites are modules of the single `tests/integration` binary
  and run single-threaded (`RUST_TEST_THREADS=1` in `.cargo/config.toml`):
  suites clean the rows they assert about (`common::clean`, `register`)
  and must never run concurrently. Run one suite with `cargo test <module>::`.
- DB tests run against the dedicated `starter_test` database
  (`db::test_pool()`, created automatically; `TEST_DATABASE_URL` to
  override) so seeding never wipes development data.
- Frontend tests are vitest suites landing with their code in the same
  commit: `*.test.ts` run in node, `*.svelte.test.ts` in headless
  chromium (`npx playwright install chromium` once). Components that
  call the API take a `fetcher` (and a `navigate`) prop so tests inject
  stubs. `frontend/e2e` holds Playwright tests against the running stack
  (`npm run test:e2e`).
- Run `cargo fmt`, `cargo clippy --all-targets` (zero warnings) and
  `cargo test`, and `npm run check`, `npm run lint`, `npm run format:check`
  and `npm test` in `frontend/`, before committing. CI runs the same, and
  fails when `frontend/src/lib/api/types` is stale.

## Workflow rules

- **Small, coherent commits** with conventional-commit prefixes.
- **No magic numbers**: every literal threshold or tuning value becomes a
  named `const` with a doc comment saying what it is and why.
- Comments explain why, or how a non-obvious piece works; never what the
  next line does.
- **Configuration goes through `src/config.rs`**: one typed `Config`
  read from the environment (`.env` via dotenvy in `main`), every setting
  with a default and a line in `.env.example`. Nothing else calls
  `std::env::var`. Tests build a `Config` with `Config::from_lookup`
  instead of mutating the process environment.
- **Authentication goes through the `Auth` facade** (`src/auth/mod.rs`):
  `auth.user(&headers)`, `auth.is_logged_in`, `auth.register`,
  `auth.login`, `auth.logout`, `auth.request_password_reset`,
  `auth.reset_password`, the profile and password updates. JSON routes
  guard with `api::require_user` (a 401 the SvelteKit `apiGet` turns into
  the login redirect). Handlers never touch the users, sessions or
  password_resets tables directly.
- **Mail goes through `mail::Mailer`** (`src/mail.rs`): SMTP when
  `SMTP_HOST` is set, the log otherwise, and `Mailer::capture` for tests
  (`common::app_with_outbox` hands back the outbox). Never build a
  transport elsewhere.
- **Wire types are Rust structs with `#[derive(ts_rs::TS)]` and
  `#[ts(export)]`**, declared next to the handler that produces or reads
  them (`User` in `src/auth/user.rs`, the request bodies in
  `src/server/auth.rs`, the envelopes in `src/server/api.rs`). `cargo test` writes them to
  `frontend/src/lib/api/types`; `frontend/src/lib/api/index.ts` re-exports
  each one by name (add a line per new type) and `$lib/types` adds the
  UI-only types. Never hand-write a mirror of a Rust type. `i64` fields
  that are JSON numbers carry `#[ts(type = "number")]`.
- **The auth guard lives in `frontend/src/hooks.server.ts` only**:
  `handle` asks `GET /api/me` on every request, stores the answer in
  `locals.user` (null for a guest) and redirects guests out of `(app)`
  and members out of `(guest)` (`routeGuard`). No layout load redirects:
  a layout load does not re-run on every client-side navigation and page
  loads run concurrently with it, so a guard there protects nothing (see
  the SvelteKit docs on load and authentication). The `(app)` layout
  load only narrows the type. Every page under `(app)` and `(guest)`
  keeps a `+page.server.ts`, so a client-side navigation still makes the
  server request the hook inspects; protected data is only ever loaded
  through `apiGet`, whose 401 redirects to the login.
- **Every Axum route lives under `/api`**, actions included
  (`POST /api/login`, `PUT /api/settings/profile`). That one prefix is
  the whole routing contract: the Vite proxy (`frontend/vite.config.ts`)
  and both Caddyfiles send `/api/*` to Axum and nothing else. Never add
  an Axum route outside `/api`; SvelteKit owns every page URL.
- Frontend mutations call Axum via fetch (`submit` in
  `frontend/src/lib/client.ts`). Page loads go through `apiGet` in
  `frontend/src/lib/server/api.ts`. Mutations answer with `flash::respond`
  (a `{redirect}` JSON for fetch, a 303 for a form) plus a toast cookie,
  or a 422 in the `ValidationErrors` shape.
- The app's name is configuration (`APP_NAME` / `PUBLIC_APP_NAME` in
  `.env`); the cookie prefix derives from it on both sides
  (`Config::cookie_prefix`, `slug` in `frontend/src/lib/app.ts`), keep the
  two slug functions identical.
- Every user-visible string in the frontend is plain English; do not add
  an i18n layer until asked.
- **Always reach for a shadcn-svelte component** (`frontend/src/lib/components/ui`,
  added with `npx shadcn-svelte@latest add <component>`) before writing a
  custom element: forms use `Input`, `Textarea`, `Select`, `Checkbox`,
  `Switch`; lists and data use `Table`, `Card`, `Badge`; overlays use
  `Dialog`, `AlertDialog`, `DropdownMenu`, `Tooltip`. Write a custom
  component only for something the kit has no counterpart for, and say so
  in its header comment. The theme lives in `frontend/src/routes/layout.css`.

## Layout and commands

- `src/config.rs` the `Config` struct, `AppEnv`, the cookie names, the
  list and flag parsers.
- `src/db/` pool, migrations (`migrations/*.sql`, sqlx, run on start),
  the test database.
- `src/auth/` the facade (`mod.rs`), Argon2id hashing and the password
  rules (`password.rs`), sessions (`session.rs`: random token, SHA-256
  stored, cookie builders), the reset tokens (`reset.rs`), the `User`
  row and its queries (`user.rs`).
- `src/mail.rs` the mailer; `src/encoding.rs` percent-encoding for the
  hand-built cookie values and links.
- `src/server/` the axum router: `api.rs` (error envelope, validation
  shape, JSON body rejection, `require_user`, `/api/health`), `auth.rs`
  (`POST /api/register`, `POST /api/login`, `POST /api/logout`,
  `POST /api/forgot-password`, `POST /api/reset-password`,
  `PUT /api/settings/profile`, `PUT /api/settings/password`,
  `DELETE /api/settings/account`), `me.rs` (`GET /api/me`, the account or a 401), `flash.rs` (toasts across redirects,
  the fetch/browser dialects, `back`).
- `tests/integration/` `common` (router, requests, cookies, flash,
  `register`, `me`, `app_with_outbox`), `auth`, `password_reset`,
  `account`, `routes`, `health`.
- `frontend/src/routes/` `+layout.server.ts` (flash, `locals.user`,
  sidebar and appearance cookies), `+page.svelte` (welcome), `(guest)/` (login,
  register, forgot-password, reset-password; signed-in visitors are
  redirected out), `(app)/` (the sidebar shell; guests are redirected to
  `/login`): `dashboard` (a placeholder), `settings/` (profile with
  account deletion, password, appearance).
- `frontend/src/lib/` `app.ts` (name, cookie prefix), `client.ts`
  (`submit`, `follow`, `firstErrors`), `server/api.ts` (`apiGet`),
  `server/current-user.ts` (`GET /api/me` for the hook), `flash.ts`, `appearance*`,
  `types.ts`, `api/` (generated), `components/` (the shell, the forms,
  `ui/` the shadcn kit).
- Ports: API 3200, shared dev origin 8787, Postgres 5436, so the stack
  coexists with other projects on one machine.
- `docker compose up --build` is the full stack; `docker compose
  --profile dev up frontend-dev` the hot-reloading one (`api-dev` runs
  watchexec around `cargo run`, see `Dockerfile.dev`); `solo.yml` lists
  the same commands for the solo runner.
