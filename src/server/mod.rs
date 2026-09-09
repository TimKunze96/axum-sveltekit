//! The axum application: everything under `/api` (reads and account
//! actions alike), everything else a JSON 404. Pages live in the
//! SvelteKit frontend, which owns every other URL; the shared-origin
//! proxy (Vite in dev, Caddy in production) sends only `/api/*` here.

pub mod api;
pub mod auth;
pub mod flash;
pub mod me;

use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;

use crate::auth::Auth;
use crate::config::Config;
use crate::mail::Mailer;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub auth: Auth,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

/// Anything not owned by the API answers a JSON 404; pages live in the
/// SvelteKit frontend behind the shared proxy.
async fn json_not_found() -> axum::response::Response {
    api::error(StatusCode::NOT_FOUND, "Not Found")
}

pub fn router(pool: PgPool, config: Arc<Config>, mailer: Mailer) -> Router {
    let state = AppState {
        auth: Auth::new(pool.clone(), config.clone(), mailer),
        pool,
        config,
    };

    Router::new()
        .nest("/api", api_router())
        .fallback(json_not_found)
        .with_state(state)
}

/// Every route answers JSON. Reads serve the page loads; the actions
/// answer a redirect in both dialects (see `flash::respond`). Signed-in
/// routes use `api::require_user` and answer 401 to guests.
fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(api::health))
        .route("/me", get(me::show))
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/forgot-password", post(auth::forgot_password))
        .route("/reset-password", post(auth::reset_password))
        .route("/settings/profile", put(auth::update_profile))
        .route("/settings/password", put(auth::update_password))
        .route("/settings/account", delete(auth::delete_account))
}
