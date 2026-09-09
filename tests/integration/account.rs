//! The settings actions behind a login: profile, password and account
//! deletion.

use axum::http::{Method, StatusCode};
use serde_json::json;
use starter::auth::password;

use crate::common::{
    PASSWORD, app, clean, clears_cookie, flash_from, json, location, me, pool, register, send_json,
    send_to, session_header,
};

const EMAIL: &str = "settings@example.com";
const OTHER: &str = "taken@example.com";

#[tokio::test]
async fn the_profile_updates_and_keeps_emails_unique() {
    let pool = pool().await;
    let app = app().await;
    // The rename below leaves this address behind when a run aborts.
    clean(&pool, "renamed@example.com").await;
    register(&app, "Taken", OTHER).await;
    let session = register(&app, "Ada", EMAIL).await;

    let updated = send_json(
        &app,
        Method::PUT,
        "/api/settings/profile",
        &[("cookie", &session), ("referer", "/api/settings/profile")],
        json!({ "name": " Renamed ", "email": "Renamed@Example.com" }),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::OK);
    assert_eq!(
        json(updated).await,
        json!({ "redirect": "/api/settings/profile" })
    );
    let account = me(&app, &session).await.expect("signed in");
    assert_eq!(account["name"], "Renamed");
    assert_eq!(account["email"], "renamed@example.com");

    // Keeping one's own email is fine; taking another account's is not.
    let same = send_json(
        &app,
        Method::PUT,
        "/api/settings/profile",
        &[("cookie", &session)],
        json!({ "name": "Renamed", "email": "renamed@example.com" }),
    )
    .await;
    assert_eq!(same.status(), StatusCode::OK);
    let taken = send_json(
        &app,
        Method::PUT,
        "/api/settings/profile",
        &[("cookie", &session)],
        json!({ "name": "Renamed", "email": OTHER }),
    )
    .await;
    assert_eq!(taken.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(taken).await["errors"],
        json!({ "email": ["This email is already registered."] })
    );
    let blank = send_json(
        &app,
        Method::PUT,
        "/api/settings/profile",
        &[("cookie", &session)],
        json!({ "name": "", "email": "" }),
    )
    .await;
    assert_eq!(
        json(blank).await["errors"],
        json!({ "name": ["A name is required."], "email": ["An email is required."] })
    );
}

#[tokio::test]
async fn the_password_changes_and_other_sessions_end() {
    let pool = pool().await;
    let app = app().await;
    let session = register(&app, "Ada", EMAIL).await;
    let other = session_header(
        &send_json(
            &app,
            Method::POST,
            "/api/login",
            &[],
            json!({ "email": EMAIL, "password": PASSWORD }),
        )
        .await,
    );

    let wrong = send_json(
        &app,
        Method::PUT,
        "/api/settings/password",
        &[("cookie", &session)],
        json!({ "current_password": "nope", "password": "short", "password_confirmation": "other" }),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(wrong).await["errors"],
        json!({
            "current_password": ["The password is incorrect."],
            "password": ["The password must be at least 8 characters."],
            "password_confirmation": ["The password confirmation does not match."],
        })
    );

    let new_password = "a brand new passphrase";
    let changed = send_json(
        &app,
        Method::PUT,
        "/api/settings/password",
        &[("cookie", &session), ("referer", "/api/settings/password")],
        json!({
            "current_password": PASSWORD,
            "password": new_password,
            "password_confirmation": new_password,
        }),
    )
    .await;
    assert_eq!(changed.status(), StatusCode::OK);
    assert_eq!(
        flash_from(&changed).expect("flash"),
        json!({ "title": "Password updated", "body": null, "type": "success" })
    );
    let continued = session_header(&changed);
    assert_ne!(continued, session);

    let hash: String = sqlx::query_scalar("select password_hash from users where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("hash");
    assert!(password::verify(&hash, new_password));
    assert!(!password::verify(&hash, PASSWORD));

    // The old sessions are gone; the fresh one works.
    for dead in [&session, &other] {
        assert_eq!(me(&app, dead).await, None);
    }
    assert_eq!(
        me(&app, &continued).await.expect("signed in")["email"],
        EMAIL
    );
}

#[tokio::test]
async fn deleting_the_account_needs_the_password() {
    let pool = pool().await;
    let app = app().await;
    let session = register(&app, "Ada", EMAIL).await;

    let wrong = send_json(
        &app,
        Method::DELETE,
        "/api/settings/account",
        &[("cookie", &session)],
        json!({ "password": "nope" }),
    )
    .await;
    assert_eq!(wrong.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(wrong).await["errors"],
        json!({ "password": ["The password is incorrect."] })
    );

    let deleted = send_to(
        &app,
        Method::DELETE,
        "/api/settings/account",
        &[("cookie", &session), ("content-type", "application/json")],
    )
    .await;
    // A bodyless delete is a malformed body, not a deletion.
    assert_eq!(deleted.status(), StatusCode::BAD_REQUEST);

    let deleted = send_json(
        &app,
        Method::DELETE,
        "/api/settings/account",
        &[("cookie", &session), ("accept", "text/html")],
        json!({ "password": PASSWORD }),
    )
    .await;
    assert_eq!(deleted.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&deleted), "/");
    assert!(clears_cookie(&deleted, "starter_session"));
    assert_eq!(
        flash_from(&deleted).expect("flash"),
        json!({ "title": "Your account has been deleted", "body": null, "type": "success" })
    );
    let exists: bool = sqlx::query_scalar("select exists (select 1 from users where email = $1)")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("account");
    assert!(!exists);
    assert_eq!(me(&app, &session).await, None);
}
