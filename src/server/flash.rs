//! One-shot toast notifications carried across a redirect. The API sets a
//! short-lived cookie; the SvelteKit root layout reads and clears it.

use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::config::Config;

/// Long enough to survive the redirect it rides on, short enough not to
/// resurface on an unrelated visit.
const FLASH_LIFETIME_SECONDS: i64 = 60;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum NotificationType {
    Success,
    Error,
}

/// The toast a redirect carries in the flash cookie; the SvelteKit root
/// layout decodes it with the exported type.
#[derive(Debug, Clone, Serialize, PartialEq, ts_rs::TS)]
#[ts(export)]
pub struct Notification {
    pub title: String,
    pub body: Option<String>,
    #[serde(rename = "type")]
    pub kind: NotificationType,
}

impl Notification {
    pub fn success(title: &str, body: Option<&str>) -> Self {
        Self {
            title: title.to_owned(),
            body: body.map(str::to_owned),
            kind: NotificationType::Success,
        }
    }

    pub fn error(title: &str, body: Option<&str>) -> Self {
        Self {
            title: title.to_owned(),
            body: body.map(str::to_owned),
            kind: NotificationType::Error,
        }
    }

    /// The `Set-Cookie` value carrying this notification: the JSON,
    /// percent-encoded so it survives cookie syntax.
    pub fn cookie(&self, config: &Config) -> String {
        let json = serde_json::to_string(self).expect("notification serializes");
        crate::auth::session::short_cookie(
            &config.flash_cookie_name(),
            &crate::encoding::percent_encode(&json),
            FLASH_LIFETIME_SECONDS,
            config.secure_cookies(),
        )
    }
}

/// Whether the request came from a fetch that wants JSON rather than a
/// browser navigation, decided by its `Accept` header.
pub fn wants_json(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|accept| accept.contains("application/json"))
}

/// The redirect-with-toast answer in both dialects: a 303 for a browser
/// form, or `{"redirect": location}` with the same flash cookie for a
/// fetch client, which cannot read where an opaque redirect went.
pub fn respond(
    config: &Config,
    headers: &HeaderMap,
    location: &str,
    notification: Option<&Notification>,
) -> Response {
    let mut response = if wants_json(headers) {
        axum::Json(super::api::RedirectAnswer {
            redirect: location.to_owned(),
        })
        .into_response()
    } else {
        axum::response::Redirect::to(location).into_response()
    };
    if let Some(notification) = notification {
        append_cookie(&mut response, &notification.cookie(config));
    }
    response
}

pub fn append_cookie(response: &mut Response, cookie: &str) {
    if let Ok(value) = HeaderValue::from_str(cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}

/// The referring page, or home: where an action returns to.
pub fn back(headers: &HeaderMap) -> String {
    headers
        .get(header::REFERER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| "/".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn config(env: &str) -> Config {
        Config::from_lookup(|name| (name == "APP_ENV").then(|| env.to_owned())).expect("config")
    }

    #[test]
    fn the_flash_cookie_carries_encoded_json() {
        let notification = Notification::success("Signed in", Some("Welcome back, Ada!"));
        let cookie = notification.cookie(&config("production"));
        assert!(cookie.starts_with("starter_flash=%7B%22title%22%3A%22Signed%20in%22"));
        assert!(cookie.contains("%22type%22%3A%22success%22"));
        assert!(cookie.ends_with("; Path=/; HttpOnly; SameSite=Lax; Max-Age=60; Secure"));
        assert!(!notification.cookie(&config("local")).contains("Secure"));

        let error = Notification::error("Your account has been deleted", None);
        assert_eq!(
            serde_json::to_value(&error).expect("json"),
            serde_json::json!({
                "title": "Your account has been deleted",
                "body": null,
                "type": "error",
            })
        );
    }

    #[test]
    fn fetch_clients_get_the_redirect_as_json() {
        let config = config("local");
        let mut headers = HeaderMap::new();
        assert!(!wants_json(&headers));
        assert_eq!(back(&headers), "/");
        headers.insert(header::ACCEPT, "application/json".parse().expect("header"));
        headers.insert(header::REFERER, "/settings".parse().expect("header"));
        assert!(wants_json(&headers));
        assert_eq!(back(&headers), "/settings");

        let response = respond(&config, &headers, "/dashboard", None);
        assert_eq!(response.status(), StatusCode::OK);
        let browser = respond(&config, &HeaderMap::new(), "/dashboard", None);
        assert_eq!(browser.status(), StatusCode::SEE_OTHER);
        assert_eq!(browser.headers()[header::LOCATION], "/dashboard");
    }
}
