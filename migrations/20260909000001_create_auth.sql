-- Accounts with a password, and server-side sessions addressed by a
-- hashed cookie token (the table never holds the token itself).

create table users (
    id bigserial primary key,
    name text not null,
    email text not null unique,
    password_hash text not null,
    is_admin boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table sessions (
    token text primary key,
    user_id bigint not null references users (id) on delete cascade,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null
);
create index sessions_user_id_index on sessions (user_id);

-- One pending password reset per email: the SHA-256 of the token the
-- mail carries, and when it stops working.
create table password_resets (
    email text primary key,
    token text not null,
    created_at timestamptz not null default now(),
    expires_at timestamptz not null
);
