//! Server-side sessions in Postgres, addressed by a random token in an
//! HttpOnly cookie. The table holds the token's SHA-256, so a database
//! read or backup never yields a usable cookie.

use axum::http::HeaderMap;
use rand::RngExt;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct Session {
    /// The stored (hashed) token.
    pub token: String,
    pub user_id: i64,
}

/// 32 random bytes, hex encoded.
pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes[..]);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The stored form of a session token.
pub fn token_hash(token: &str) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Opens a session and answers the token to hand to the browser.
pub async fn create_session(
    pool: &PgPool,
    user_id: i64,
    lifetime_days: i64,
) -> sqlx::Result<String> {
    let token = random_token();

    // Expired sessions are dead tokens; every login sweeps them.
    sqlx::query("delete from sessions where expires_at < now()")
        .execute(pool)
        .await?;

    sqlx::query(
        "insert into sessions (token, user_id, expires_at)
         values ($1, $2, now() + make_interval(days => $3))",
    )
    .bind(token_hash(&token))
    .bind(user_id)
    .bind(i32::try_from(lifetime_days).unwrap_or(i32::MAX))
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn session_by_token(pool: &PgPool, token: &str) -> sqlx::Result<Option<Session>> {
    let row =
        sqlx::query("select token, user_id from sessions where token = $1 and expires_at > now()")
            .bind(token_hash(token))
            .fetch_optional(pool)
            .await?;

    Ok(row.map(|row| Session {
        token: row.get("token"),
        user_id: row.get("user_id"),
    }))
}

pub async fn delete_session(pool: &PgPool, token: &str) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where token = $1")
        .bind(token_hash(token))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_user_sessions(pool: &PgPool, user_id: i64) -> sqlx::Result<()> {
    sqlx::query("delete from sessions where user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// The session of the request's cookie, if there is a live one.
pub async fn session_from_headers(
    pool: &PgPool,
    headers: &HeaderMap,
    cookie_name: &str,
) -> sqlx::Result<Option<Session>> {
    match cookie_value(headers, cookie_name) {
        Some(token) => session_by_token(pool, &token).await,
        None => Ok(None),
    }
}

/// Reads a cookie from the request headers.
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookies = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;

    cookies.split(';').find_map(|pair| {
        let (cookie_name, value) = pair.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_owned())
    })
}

fn secure_flag(secure: bool) -> &'static str {
    if secure { "; Secure" } else { "" }
}

pub fn session_cookie(name: &str, token: &str, lifetime_days: i64, secure: bool) -> String {
    let max_age = lifetime_days * 24 * 60 * 60;
    format!(
        "{name}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        secure_flag(secure)
    )
}

/// A short-lived cookie for a multi-step flow (an OAuth state, a marker).
pub fn short_cookie(name: &str, value: &str, max_age_seconds: i64, secure: bool) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}{}",
        secure_flag(secure)
    )
}

pub fn clear_cookie(name: &str, secure: bool) -> String {
    format!(
        "{name}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
        secure_flag(secure)
    )
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, header};

    use super::*;

    #[test]
    fn cookies_parse_out_of_the_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "other=1; starter_session=abc123; last=x"
                .parse()
                .expect("header"),
        );

        assert_eq!(
            cookie_value(&headers, "starter_session"),
            Some("abc123".to_owned())
        );
        assert_eq!(cookie_value(&headers, "missing"), None);
    }

    #[test]
    fn tokens_are_random_hex_and_hash_deterministically() {
        let token = random_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(token, random_token());
        assert_eq!(token_hash(&token), token_hash(&token));
        assert_ne!(token_hash(&token), token);
    }

    #[test]
    fn cookies_carry_the_lifetime_and_the_secure_flag() {
        assert_eq!(
            session_cookie("starter_session", "t", 30, true),
            "starter_session=t; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000; Secure"
        );
        assert_eq!(
            session_cookie("starter_session", "t", 1, false),
            "starter_session=t; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400"
        );
        assert_eq!(
            clear_cookie("starter_session", false),
            "starter_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"
        );
        assert!(short_cookie("state", "abc", 600, true).ends_with("Max-Age=600; Secure"));
    }
}
