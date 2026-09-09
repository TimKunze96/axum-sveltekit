//! The account actions: register, log in, log out, the password reset
//! pair, and the settings forms. Every action takes a JSON body, validates through the `Auth`
//! facade, and answers a redirect with a toast in both dialects (see
//! `flash::respond`), or a 422 in the validation shape.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Deserialize;

use super::AppState;
use super::api::{auth_error, body, require_user};
use super::flash::{Notification, append_cookie, back, respond};
use crate::auth::Registration;

/// Where a login lands; the SvelteKit hook owns the guest redirect.
pub const HOME_PATH: &str = "/dashboard";

/// Where the reset flow sends the visitor afterwards.
const LOGIN_PAGE: &str = "/login";

/// `POST /api/register`: create the account and sign it in.
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<Registration>, JsonRejection>,
) -> Response {
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    // A signed-in visitor registering another account leaves the old
    // session behind.
    if let Err(error) = state.auth.logout(&headers).await {
        return super::api::database_error(error);
    }
    match state.auth.register(input).await {
        Ok(login) => {
            let mut response = respond(
                &state.config,
                &headers,
                HOME_PATH,
                Some(&Notification::success(
                    "Welcome",
                    Some(&format!("Your account is ready, {}.", login.user.name)),
                )),
            );
            append_cookie(&mut response, &state.auth.session_cookie(&login.token));
            response
        }
        Err(error) => auth_error(error),
    }
}

/// The body of `POST /api/login`.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Credentials {
    #[serde(default)]
    email: String,
    #[serde(default)]
    password: String,
}

/// `POST /api/login`: check the credentials and open a fresh session (any
/// previous one dies, so a fixated cookie never survives a login).
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<Credentials>, JsonRejection>,
) -> Response {
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    if let Err(error) = state.auth.logout(&headers).await {
        return super::api::database_error(error);
    }
    match state.auth.login(&input.email, &input.password).await {
        Ok(login) => {
            let mut response = respond(
                &state.config,
                &headers,
                HOME_PATH,
                Some(&Notification::success(
                    "Signed in",
                    Some(&format!("Welcome back, {}!", login.user.name)),
                )),
            );
            append_cookie(&mut response, &state.auth.session_cookie(&login.token));
            response
        }
        Err(error) => auth_error(error),
    }
}

/// `POST /api/logout`: end the session and go home.
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(error) = state.auth.logout(&headers).await {
        return super::api::database_error(error);
    }
    let mut response = respond(&state.config, &headers, "/", None);
    append_cookie(&mut response, &state.auth.clear_session_cookie());
    response
}

/// The body of `POST /api/forgot-password`.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ForgotPasswordInput {
    #[serde(default)]
    email: String,
}

/// `POST /api/forgot-password`: mail a reset link. Answers the same
/// success for unknown emails.
pub async fn forgot_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<ForgotPasswordInput>, JsonRejection>,
) -> Response {
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    match state.auth.request_password_reset(&input.email).await {
        Ok(()) => respond(
            &state.config,
            &headers,
            LOGIN_PAGE,
            Some(&Notification::success(
                "Check your mail",
                Some("If that address is registered, a reset link is on its way."),
            )),
        ),
        Err(error) => auth_error(error),
    }
}

/// The body of `POST /api/reset-password`: the link's token and email
/// plus the new password.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ResetPasswordInput {
    #[serde(default)]
    email: String,
    #[serde(default)]
    token: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    #[ts(optional)]
    password_confirmation: Option<String>,
}

/// `POST /api/reset-password`: set the password from a reset link.
pub async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<ResetPasswordInput>, JsonRejection>,
) -> Response {
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    match state
        .auth
        .reset_password(
            &input.email,
            &input.token,
            &input.password,
            input.password_confirmation.as_deref(),
        )
        .await
    {
        Ok(()) => respond(
            &state.config,
            &headers,
            LOGIN_PAGE,
            Some(&Notification::success(
                "Password reset",
                Some("Sign in with your new password."),
            )),
        ),
        Err(error) => auth_error(error),
    }
}

/// The body of `PUT /api/settings/profile`.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ProfileInput {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
}

/// `PUT /api/settings/profile`: the name and email.
pub async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<ProfileInput>, JsonRejection>,
) -> Response {
    let user = match require_user(&state.auth, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    match state
        .auth
        .update_profile(&user, &input.name, &input.email)
        .await
    {
        Ok(_) => respond(
            &state.config,
            &headers,
            &back(&headers),
            Some(&Notification::success("Profile updated", None)),
        ),
        Err(error) => auth_error(error),
    }
}

/// The body of `PUT /api/settings/password`.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct PasswordInput {
    #[serde(default)]
    current_password: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    #[ts(optional)]
    password_confirmation: Option<String>,
}

/// `PUT /api/settings/password`: replace the password; other sessions of the
/// account end, this one continues on a fresh token.
pub async fn update_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<PasswordInput>, JsonRejection>,
) -> Response {
    let user = match require_user(&state.auth, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    match state
        .auth
        .change_password(
            &user,
            &input.current_password,
            &input.password,
            input.password_confirmation.as_deref(),
        )
        .await
    {
        Ok(token) => {
            let mut response = respond(
                &state.config,
                &headers,
                &back(&headers),
                Some(&Notification::success("Password updated", None)),
            );
            append_cookie(&mut response, &state.auth.session_cookie(&token));
            response
        }
        Err(error) => auth_error(error),
    }
}

/// The body of `DELETE /api/settings/account`.
#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DeleteInput {
    #[serde(default)]
    password: String,
}

/// `DELETE /api/settings/account`: delete the account after a password check.
pub async fn delete_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    input: Result<Json<DeleteInput>, JsonRejection>,
) -> Response {
    let user = match require_user(&state.auth, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let input = match body(input) {
        Ok(input) => input,
        Err(response) => return response,
    };
    match state.auth.delete_account(&user, &input.password).await {
        Ok(()) => {
            let mut response = respond(
                &state.config,
                &headers,
                "/",
                Some(&Notification::success(
                    "Your account has been deleted",
                    None,
                )),
            );
            append_cookie(&mut response, &state.auth.clear_session_cookie());
            response
        }
        Err(error) => auth_error(error),
    }
}
