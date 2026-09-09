//! The operational probe the container health check and the deploy
//! script wait on.

use axum::http::{Method, StatusCode, header};

use crate::common::{json, keys, send};

#[tokio::test]
async fn the_health_probe_answers_while_the_database_does() {
    let response = send(Method::GET, "/api/health").await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    let body = json(response).await;
    assert_eq!(keys(&body), ["status"]);
    assert_eq!(body["status"], "ok");
}
