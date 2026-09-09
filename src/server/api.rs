//! The JSON API's shared response shapes, the session guard for JSON
//! routes and the operational endpoints.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

use super::AppState;
use crate::auth::{Auth, AuthError, FieldError, User};

/// The error envelope of every non-2xx JSON answer.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ApiError {
    pub message: String,
}

/// The 422 answer: the headline message and every failing field's
/// messages, the shape `frontend/src/lib/client.ts` turns into form
/// errors.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ValidationErrors {
    pub message: String,
    pub errors: std::collections::BTreeMap<String, Vec<String>>,
}

/// A mutation's answer to a fetch client: where the browser would have
/// been redirected (see `flash::respond`).
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RedirectAnswer {
    pub redirect: String,
}

pub fn error(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ApiError {
            message: message.to_owned(),
        }),
    )
        .into_response()
}

/// A validation failure: 422 with the message and the field's errors,
/// the shape `frontend/src/lib/client.ts` turns into form errors.
pub fn validation_error(field: &str, message: &str) -> Response {
    validation_errors(&[(field.to_owned(), message.to_owned())])
}

/// Several fields failing at once: the first message headlines, with the
/// count of the rest appended, and every field lists its messages.
pub fn validation_errors(errors: &[FieldError]) -> Response {
    let mut fields: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (field, message) in errors {
        fields
            .entry(field.clone())
            .or_default()
            .push(message.clone());
    }
    let first = errors
        .first()
        .map(|(_, message)| message.as_str())
        .unwrap_or_default();
    let message = match errors.len() {
        0 | 1 => first.to_owned(),
        2 => format!("{first} (and 1 more error)"),
        count => format!("{first} (and {} more errors)", count - 1),
    };
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        Json(ValidationErrors {
            message,
            errors: fields,
        }),
    )
        .into_response()
}

pub fn database_error(error: sqlx::Error) -> Response {
    tracing::warn!(%error, "api database error");
    self::error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.")
}

/// Every [`AuthError`] as a response: validation as a 422, bad
/// credentials as a 422 on the email field, the rest as a 500.
pub fn auth_error(error: AuthError) -> Response {
    match error {
        AuthError::Validation(errors) => validation_errors(&errors),
        AuthError::InvalidCredentials => {
            validation_error("email", "These credentials do not match our records.")
        }
        AuthError::Database(error) => database_error(error),
        AuthError::Hash(error) => {
            tracing::error!(%error, "password hashing failed");
            self::error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.")
        }
        AuthError::Mail(error) => {
            tracing::error!(%error, "sending mail failed");
            self::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The mail could not be sent right now.",
            )
        }
    }
}

/// A JSON body that failed to parse answers a 400 in the API's envelope
/// instead of axum's plain-text rejection.
pub fn body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, Response> {
    body.map(|Json(value)| value).map_err(|rejection| {
        self::error(
            StatusCode::BAD_REQUEST,
            &format!("Malformed JSON body: {}", rejection.body_text()),
        )
    })
}

/// The account behind a JSON route; guests get a 401 the SvelteKit
/// `apiGet` helper turns into the login redirect.
pub async fn require_user(auth: &Auth, headers: &HeaderMap) -> Result<User, Response> {
    match auth.user(headers).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => Err(error(StatusCode::UNAUTHORIZED, "Unauthenticated.")),
        Err(error) => Err(database_error(error)),
    }
}

/// `GET /api/health`: for uptime checks and the container health probe,
/// 200 once the database answers, 503 otherwise.
pub async fn health(State(state): State<AppState>) -> Response {
    match sqlx::query_scalar::<_, i32>("select 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => Json(json!({ "status": "ok" })).into_response(),
        Err(error) => {
            tracing::warn!(%error, "health check database probe failed");
            self::error(StatusCode::SERVICE_UNAVAILABLE, "Database unavailable.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    async fn json_body(response: Response) -> serde_json::Value {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        serde_json::from_slice(&bytes).expect("json")
    }

    #[tokio::test]
    async fn validation_errors_headline_the_first_message() {
        let single = validation_error("name", "A name is required.");
        assert_eq!(single.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(single).await,
            json!({
                "message": "A name is required.",
                "errors": { "name": ["A name is required."] },
            })
        );

        let several = validation_errors(&[
            ("name".to_owned(), "A name is required.".to_owned()),
            ("name".to_owned(), "Too short.".to_owned()),
            ("email".to_owned(), "Invalid.".to_owned()),
        ]);
        assert_eq!(
            json_body(several).await,
            json!({
                "message": "A name is required. (and 2 more errors)",
                "errors": { "name": ["A name is required.", "Too short."], "email": ["Invalid."] },
            })
        );
    }

    #[tokio::test]
    async fn bad_credentials_land_on_the_email_field() {
        let response = auth_error(AuthError::InvalidCredentials);
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            json_body(response).await,
            json!({
                "message": "These credentials do not match our records.",
                "errors": { "email": ["These credentials do not match our records."] },
            })
        );
    }
}
