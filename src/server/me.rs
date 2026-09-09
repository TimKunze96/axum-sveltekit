//! `GET /api/me`: the signed-in account, a 401 for guests. The SvelteKit
//! server hook asks it on every request.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};

use super::AppState;

pub async fn show(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match super::api::require_user(&state.auth, &headers).await {
        Ok(user) => Json(user).into_response(),
        Err(response) => response,
    }
}
