//! Postgres access: connection pool, migrations and the test database.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;

pub async fn connect(config: &Config) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}

/// The dedicated test database, so test seeding never wipes development
/// data.
pub const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://starter:starter@127.0.0.1:5436/starter_test";

pub fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_owned())
}

/// Connects to the test database, creating it on first use.
pub async fn test_pool() -> sqlx::Result<PgPool> {
    let url = test_database_url();
    ensure_database(&url).await?;

    PgPoolOptions::new().max_connections(5).connect(&url).await
}

/// Creates the database of the given URL if it does not exist yet, via the
/// server's maintenance database.
async fn ensure_database(url: &str) -> sqlx::Result<()> {
    let (server_url, database) = url
        .rsplit_once('/')
        .expect("database URL with a database name");

    // Guard the identifier: it is interpolated into CREATE DATABASE, which
    // cannot take bind parameters.
    assert!(
        database
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "unsafe test database name: {database}",
    );

    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(&format!("{server_url}/postgres"))
        .await?;

    let exists: Option<i32> = sqlx::query_scalar("select 1 from pg_database where datname = $1")
        .bind(database)
        .fetch_optional(&admin)
        .await?;
    if exists.is_none() {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            r#"create database "{database}""#
        )))
        .execute(&admin)
        .await?;
    }

    Ok(())
}
