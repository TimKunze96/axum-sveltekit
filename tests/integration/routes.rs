//! Route inventory: the status-level contract of every registered route.
//! Feature-level behavior gets its own suite; the inventory grows with
//! them.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::common::{app, json, keys, location, send, send_json, send_to};

#[tokio::test]
async fn unknown_paths_answer_a_json_not_found() {
    for path in ["/", "/nope", "/login", "/api/nope", "/api/health/extra"] {
        let response = send(Method::GET, path).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        let body = json(response).await;
        assert_eq!(keys(&body), ["message"], "{path}");
        assert_eq!(body["message"], "Not Found", "{path}");
    }
}

#[tokio::test]
async fn the_health_probe_is_read_only() {
    for method in [Method::POST, Method::PUT, Method::DELETE] {
        let response = send(method.clone(), "/api/health").await;
        assert_eq!(
            response.status(),
            StatusCode::METHOD_NOT_ALLOWED,
            "{method}"
        );
    }
}

#[tokio::test]
async fn signed_in_routes_answer_guests_a_401() {
    let app = app().await;
    let read = send_to(&app, Method::GET, "/api/me", &[]).await;
    assert_eq!(read.status(), StatusCode::UNAUTHORIZED);
    let body = json(read).await;
    assert_eq!(keys(&body), ["message"]);
    assert_eq!(body["message"], "Unauthenticated.");

    for (method, path) in [
        (Method::PUT, "/api/settings/profile"),
        (Method::PUT, "/api/settings/password"),
        (Method::DELETE, "/api/settings/account"),
    ] {
        let response = send_json(&app, method.clone(), path, &[], json!({})).await;
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {path}"
        );
    }
}

#[tokio::test]
async fn malformed_bodies_answer_a_json_400() {
    let app = app().await;
    let response = send_to(
        &app,
        Method::POST,
        "/api/login",
        &[("content-type", "application/json")],
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json(response).await;
    assert_eq!(keys(&body), ["message"]);
    assert!(
        body["message"]
            .as_str()
            .expect("message")
            .starts_with("Malformed JSON body")
    );
}

#[tokio::test]
async fn a_guest_logout_is_harmless() {
    let browser = send(Method::POST, "/api/logout").await;
    assert_eq!(browser.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&browser), "/");
    assert!(crate::common::clears_cookie(&browser, "starter_session"));
}
