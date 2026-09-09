//! The account row and its queries. The password hash never leaves this
//! module except to the facade's verification.

use serde::Serialize;
use sqlx::PgPool;

/// The account as the API exposes it; the `Serialize` key set is what
/// `GET /api/me` answers, exported to the frontend by ts-rs.
#[derive(Debug, Clone, PartialEq, Serialize, sqlx::FromRow, ts_rs::TS)]
#[ts(export)]
pub struct User {
    /// JSON numbers on the wire, so `number` rather than ts-rs's `bigint`.
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
}

const COLUMNS: &str = "id, name, email, is_admin";

pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<User>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {COLUMNS} from users where id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// By normalized email (see `auth::normalize_email`).
pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<User>> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {COLUMNS} from users where email = $1"
    )))
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// The account with its password hash, for a login attempt.
pub async fn find_credentials(pool: &PgPool, email: &str) -> sqlx::Result<Option<(User, String)>> {
    let row: Option<(i64, String, String, bool, String)> = sqlx::query_as(
        "select id, name, email, is_admin, password_hash from users where email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id, name, email, is_admin, password_hash)| {
        (
            User {
                id,
                name,
                email,
                is_admin,
            },
            password_hash,
        )
    }))
}

pub async fn password_hash(pool: &PgPool, id: i64) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar("select password_hash from users where id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert(
    pool: &PgPool,
    name: &str,
    email: &str,
    password_hash: &str,
    is_admin: bool,
) -> sqlx::Result<User> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "insert into users (name, email, password_hash, is_admin)
         values ($1, $2, $3, $4)
         returning {COLUMNS}"
    )))
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .bind(is_admin)
    .fetch_one(pool)
    .await
}

pub async fn update_profile(pool: &PgPool, id: i64, name: &str, email: &str) -> sqlx::Result<User> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "update users set name = $2, email = $3, updated_at = now()
         where id = $1
         returning {COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(email)
    .fetch_one(pool)
    .await
}

pub async fn update_password_hash(pool: &PgPool, id: i64, password_hash: &str) -> sqlx::Result<()> {
    sqlx::query("update users set password_hash = $2, updated_at = now() where id = $1")
        .bind(id)
        .bind(password_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_admin(pool: &PgPool, id: i64, is_admin: bool) -> sqlx::Result<()> {
    sqlx::query("update users set is_admin = $2, updated_at = now() where id = $1")
        .bind(id)
        .bind(is_admin)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<()> {
    sqlx::query("delete from users where id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
