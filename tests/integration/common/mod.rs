//! Helpers shared by the integration suites: the router under test, a
//! request shorthand, cookie and flash readers, and a registered
//! account.

// Not every suite uses every helper.
#![allow(dead_code)]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request};
use axum::response::Response;
use http_body_util::BodyExt;
use serde_json::json;
use starter::config::Config;
use starter::mail::{Mailer, Outbox};
use tower::ServiceExt;

/// The suites run as a local deployment: cookies without `Secure`, the
/// default app name, no admin emails unless a test passes its own vars.
pub fn config_with(vars: &[(&str, &str)]) -> Config {
    Config::from_lookup(|name| {
        vars.iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
            .or_else(|| (name == "APP_ENV").then(|| "local".to_owned()))
    })
    .expect("test config")
}

/// The migrated test database. A fresh pool per call, never shared
/// across tests: every `#[tokio::test]` runs its own runtime, and a
/// connection opened under a finished runtime hangs until the acquire
/// timeout. The suites run one at a time, so the pools never pile up.
pub async fn pool() -> sqlx::PgPool {
    static LOGGING: std::sync::Once = std::sync::Once::new();
    LOGGING.call_once(|| {
        // `RUST_LOG=warn cargo test -- --nocapture` shows what a 500 hid.
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
            )
            .with_test_writer()
            .try_init();
    });
    let pool = starter::db::test_pool()
        .await
        .expect("Postgres not reachable - start it with `docker compose up -d postgres`");
    starter::db::migrate(&pool).await.expect("migrations run");
    pool
}

/// The router the suites drive by default, with a capturing mailer whose
/// outbox nobody reads. Built per call, see [`pool`].
pub async fn app() -> Router {
    app_with(config_with(&[])).await
}

pub async fn app_with(config: Config) -> Router {
    app_with_outbox(config).await.0
}

/// A router whose mail lands in the returned outbox.
pub async fn app_with_outbox(config: Config) -> (Router, Outbox) {
    let (mailer, outbox) = Mailer::capture(&config.mail_from);
    (
        starter::server::router(pool().await, Arc::new(config), mailer),
        outbox,
    )
}

/// Sends a bodyless request through a fresh router.
pub async fn send(method: Method, path: &str) -> Response {
    send_to(&app().await, method, path, &[]).await
}

/// Sends a request with the given headers through a specific router.
pub async fn send_to(
    app: &Router,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    app.clone()
        .oneshot(request.body(Body::empty()).expect("valid request"))
        .await
        .expect("infallible")
}

/// Sends a JSON body through a specific router, as the frontend's
/// `submit` helper does (`accept: application/json`).
pub async fn send_json(
    app: &Router,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
    body: serde_json::Value,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("accept"))
    {
        request = request.header("accept", "application/json");
    }
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    app.clone()
        .oneshot(
            request
                .body(Body::from(body.to_string()))
                .expect("valid request"),
        )
        .await
        .expect("infallible")
}

/// The `Location` header, empty when absent.
pub fn location(response: &Response) -> String {
    response
        .headers()
        .get(axum::http::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

/// The value a `Set-Cookie` header assigns to `name`, ignoring deletions.
pub fn cookie_from(response: &Response, name: &str) -> Option<String> {
    response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|cookie| cookie.starts_with(&format!("{name}=")) && !cookie.contains("Max-Age=0"))
        .and_then(|cookie| cookie.split(';').next())
        .and_then(|pair| pair.split_once('='))
        .map(|(_, value)| value.to_owned())
}

/// Whether the response deletes the named cookie.
pub fn clears_cookie(response: &Response, name: &str) -> bool {
    response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .any(|cookie| cookie.starts_with(&format!("{name}=;")) && cookie.contains("Max-Age=0"))
}

/// The session cookie a login answered with, as a `cookie` header value.
pub fn session_header(response: &Response) -> String {
    let token = cookie_from(response, "starter_session").expect("session cookie");
    format!("starter_session={token}")
}

/// The flash notification a response carries, decoded.
pub fn flash_from(response: &Response) -> Option<serde_json::Value> {
    let raw = cookie_from(response, "starter_flash")?;
    let mut bytes = Vec::with_capacity(raw.len());
    let mut chars = raw.bytes();
    while let Some(byte) = chars.next() {
        if byte == b'%' {
            let hex = [chars.next()?, chars.next()?];
            bytes.push(u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    serde_json::from_slice(&bytes).ok()
}

/// The response body parsed as JSON.
pub async fn json(response: Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body read")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("JSON body")
}

/// The sorted key set of a JSON object, for exact-shape assertions.
pub fn keys(value: &serde_json::Value) -> Vec<&str> {
    let mut keys: Vec<&str> = value
        .as_object()
        .expect("JSON object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys
}

/// `GET /api/me` with the given cookie header: the account, or none for
/// a guest (a 401).
pub async fn me(app: &Router, cookie: &str) -> Option<serde_json::Value> {
    let response = send_to(app, Method::GET, "/api/me", &[("cookie", cookie)]).await;
    match response.status() {
        axum::http::StatusCode::OK => Some(json(response).await),
        axum::http::StatusCode::UNAUTHORIZED => None,
        status => panic!("unexpected /api/me status {status}"),
    }
}

/// The password every test account registers with.
pub const PASSWORD: &str = "correct horse battery staple";

/// Removes the account with this email, so a suite starts clean.
pub async fn clean(pool: &sqlx::PgPool, email: &str) {
    sqlx::query("delete from users where email = $1")
        .bind(email)
        .execute(pool)
        .await
        .expect("clean account");
}

/// Registers a fresh account through the API and answers its session
/// header, the state most signed-in tests start from.
pub async fn register(app: &Router, name: &str, email: &str) -> String {
    clean(&pool().await, email).await;
    let response = send_json(
        app,
        Method::POST,
        "/api/register",
        &[],
        json!({ "name": name, "email": email, "password": PASSWORD }),
    )
    .await;
    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "registration of {email}"
    );
    session_header(&response)
}
