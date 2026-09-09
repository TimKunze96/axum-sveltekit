//! The password reset tokens: one pending reset per email, stored as the
//! SHA-256 of the token the mail carries.

use sqlx::PgPool;

use super::session::token_hash;

pub async fn store(
    pool: &PgPool,
    email: &str,
    token: &str,
    lifetime_minutes: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into password_resets (email, token, expires_at)
         values ($1, $2, now() + make_interval(mins => $3))
         on conflict (email) do update set
             token = excluded.token,
             created_at = now(),
             expires_at = excluded.expires_at",
    )
    .bind(email)
    .bind(token_hash(token))
    .bind(i32::try_from(lifetime_minutes).unwrap_or(i32::MAX))
    .execute(pool)
    .await?;
    Ok(())
}

/// Whether the token is the live reset of the email.
pub async fn is_valid(pool: &PgPool, email: &str, token: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        "select exists (
            select 1 from password_resets
            where email = $1 and token = $2 and expires_at > now()
         )",
    )
    .bind(email)
    .bind(token_hash(token))
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, email: &str) -> sqlx::Result<()> {
    sqlx::query("delete from password_resets where email = $1")
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}
