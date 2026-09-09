//! The forgot-password mail and the reset it links to.

use axum::http::{Method, StatusCode};
use serde_json::json;
use starter::auth::session::token_hash;

use crate::common::{
    PASSWORD, app_with_outbox, clean, config_with, flash_from, json, location, me, pool, register,
    send_json, session_header,
};

const EMAIL: &str = "reset@example.com";

/// The token and email the mailed link carries.
fn link_parameters(text: &str) -> (String, String) {
    let link = text
        .split_whitespace()
        .find(|word| word.contains("/reset-password?"))
        .expect("a reset link in the mail");
    let query = link.split_once('?').expect("query").1;
    let mut token = None;
    let mut email = None;
    for pair in query.split('&') {
        match pair.split_once('=') {
            Some(("token", value)) => token = Some(value.to_owned()),
            Some(("email", value)) => email = Some(value.replace("%40", "@")),
            _ => {}
        }
    }
    (token.expect("token"), email.expect("email"))
}

#[tokio::test]
async fn a_registered_email_gets_a_link_and_an_unknown_one_the_same_answer() {
    let pool = pool().await;
    let (app, outbox) =
        app_with_outbox(config_with(&[("APP_URL", "https://starter.example/")])).await;
    register(&app, "Ada", EMAIL).await;

    let response = send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[("accept", "text/html")],
        json!({ "email": " Reset@Example.com " }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&response), "/login");
    assert_eq!(
        flash_from(&response).expect("flash"),
        json!({
            "title": "Check your mail",
            "body": "If that address is registered, a reset link is on its way.",
            "type": "success",
        })
    );
    let mails = outbox.lock().expect("outbox").clone();
    assert_eq!(mails.len(), 1);
    assert_eq!(mails[0].to, EMAIL);
    assert_eq!(mails[0].subject, "Reset your password");
    assert!(mails[0].text.starts_with("Hello Ada,"));
    let (token, email) = link_parameters(&mails[0].text);
    assert_eq!(email, EMAIL);
    assert!(mails[0].text.contains(&format!(
        "https://starter.example/reset-password?token={token}&email=reset%40example.com"
    )));
    assert!(mails[0].text.contains("works for 60 minutes"));
    // The table holds the hash, never the token itself.
    let stored: String = sqlx::query_scalar("select token from password_resets where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("reset row");
    assert_eq!(stored, token_hash(&token));

    // Unknown: the same answer, no mail.
    let unknown = send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[],
        json!({ "email": "nobody@example.com" }),
    )
    .await;
    assert_eq!(unknown.status(), StatusCode::OK);
    assert_eq!(json(unknown).await, json!({ "redirect": "/login" }));
    assert_eq!(outbox.lock().expect("outbox").len(), 1);

    // Only a malformed email is rejected.
    let bad = send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[],
        json!({ "email": "nope" }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(bad).await["errors"],
        json!({ "email": ["Enter a valid email address."] })
    );

    // Asking again replaces the pending token.
    send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[],
        json!({ "email": EMAIL }),
    )
    .await;
    let rows: i64 = sqlx::query_scalar("select count(*) from password_resets where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(rows, 1);
    let replaced: String = sqlx::query_scalar("select token from password_resets where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("reset row");
    assert_ne!(replaced, stored);
}

#[tokio::test]
async fn the_link_resets_the_password_once_and_ends_every_session() {
    let pool = pool().await;
    let (app, outbox) = app_with_outbox(config_with(&[])).await;
    let session = register(&app, "Ada", EMAIL).await;
    send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[],
        json!({ "email": EMAIL }),
    )
    .await;
    let (token, _) = link_parameters(&outbox.lock().expect("outbox")[0].text);

    let short = send_json(
        &app,
        Method::POST,
        "/api/reset-password",
        &[],
        json!({ "email": EMAIL, "token": token, "password": "short", "password_confirmation": "other" }),
    )
    .await;
    assert_eq!(short.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(short).await["errors"],
        json!({
            "password": ["The password must be at least 8 characters."],
            "password_confirmation": ["The password confirmation does not match."],
        })
    );

    let new_password = "a brand new passphrase";
    let reset = send_json(
        &app,
        Method::POST,
        "/api/reset-password",
        &[],
        json!({
            "email": "RESET@example.com",
            "token": token,
            "password": new_password,
            "password_confirmation": new_password,
        }),
    )
    .await;
    assert_eq!(reset.status(), StatusCode::OK);
    assert_eq!(json(reset).await, json!({ "redirect": "/login" }));

    // The old session is gone, the new password works, the link is spent.
    assert_eq!(me(&app, &session).await, None);
    let login = send_json(
        &app,
        Method::POST,
        "/api/login",
        &[],
        json!({ "email": EMAIL, "password": new_password }),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    assert!(me(&app, &session_header(&login)).await.is_some());
    let again = send_json(
        &app,
        Method::POST,
        "/api/reset-password",
        &[],
        json!({ "email": EMAIL, "token": token, "password": new_password }),
    )
    .await;
    assert_eq!(again.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(again).await["errors"],
        json!({ "email": ["This password reset link is invalid or has expired."] })
    );
    let rows: i64 = sqlx::query_scalar("select count(*) from password_resets where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(rows, 0);
}

#[tokio::test]
async fn wrong_expired_and_foreign_links_are_rejected() {
    let pool = pool().await;
    let (app, outbox) = app_with_outbox(config_with(&[])).await;
    register(&app, "Ada", EMAIL).await;
    clean(&pool, "other@example.com").await;
    send_json(
        &app,
        Method::POST,
        "/api/forgot-password",
        &[],
        json!({ "email": EMAIL }),
    )
    .await;
    let (token, _) = link_parameters(&outbox.lock().expect("outbox")[0].text);
    let invalid = json!({ "email": ["This password reset link is invalid or has expired."] });

    for (email, candidate) in [
        (EMAIL, "not-the-token"),
        ("other@example.com", token.as_str()),
        ("", ""),
    ] {
        let response = send_json(
            &app,
            Method::POST,
            "/api/reset-password",
            &[],
            json!({ "email": email, "token": candidate, "password": PASSWORD }),
        )
        .await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{email}"
        );
        assert_eq!(json(response).await["errors"], invalid, "{email}");
    }

    sqlx::query(
        "update password_resets set expires_at = now() - interval '1 second' where email = $1",
    )
    .bind(EMAIL)
    .execute(&pool)
    .await
    .expect("expire");
    let expired = send_json(
        &app,
        Method::POST,
        "/api/reset-password",
        &[],
        json!({ "email": EMAIL, "token": token, "password": PASSWORD }),
    )
    .await;
    assert_eq!(expired.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json(expired).await["errors"], invalid);
    let unchanged = send_json(
        &app,
        Method::POST,
        "/api/login",
        &[],
        json!({ "email": EMAIL, "password": PASSWORD }),
    )
    .await;
    assert_eq!(
        unchanged.status(),
        StatusCode::OK,
        "the password did not change"
    );
}
