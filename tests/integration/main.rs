//! The single integration-test binary: every suite lives here as a module
//! so a full `cargo test` links one executable instead of one per suite.
//!
//! The suites assume they never run concurrently with each other: they
//! clean the rows they assert about and a few mutate process env vars.
//! `.cargo/config.toml` pins `RUST_TEST_THREADS=1` to keep that
//! assumption true inside this shared process.

mod common;

mod account;
mod auth;
mod health;
mod password_reset;
mod routes;
