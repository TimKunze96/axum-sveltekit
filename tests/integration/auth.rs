//! Registration, login and logout through the HTTP actions, and the
//! account `GET /api/me` answers with.

use axum::http::{Method, StatusCode};
use serde_json::json;
use starter::auth::session::token_hash;

use crate::common::{
    PASSWORD, app, app_with, clean, clears_cookie, config_with, cookie_from, flash_from, json,
    keys, location, me, pool, register, send_json, send_to, session_header,
};

const EMAIL: &str = "ada@example.com";

#[tokio::test]
async fn registering_creates_the_account_and_signs_it_in() {
    let pool = pool().await;
    clean(&pool, EMAIL).await;
    let app = app().await;

    let response = send_json(
        &app,
        Method::POST,
        "/api/register",
        &[],
        json!({
            "name": "  Ada  ",
            "email": " Ada@Example.com ",
            "password": PASSWORD,
            "password_confirmation": PASSWORD,
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        flash_from(&response).expect("flash"),
        json!({ "title": "Welcome", "body": "Your account is ready, Ada.", "type": "success" })
    );
    let session = session_header(&response);
    let body = json(response).await;
    assert_eq!(body, json!({ "redirect": "/dashboard" }));

    let (name, email, is_admin, hash): (String, String, bool, String) =
        sqlx::query_as("select name, email, is_admin, password_hash from users where email = $1")
            .bind(EMAIL)
            .fetch_one(&pool)
            .await
            .expect("account");
    assert_eq!(
        (name.as_str(), email.as_str(), is_admin),
        ("Ada", EMAIL, false)
    );
    assert!(hash.starts_with("$argon2id$"), "the password is hashed");
    assert_ne!(hash, PASSWORD);

    let account = me(&app, &session).await.expect("signed in");
    assert_eq!(keys(&account), ["email", "id", "is_admin", "name"]);
    assert_eq!(account["name"], "Ada");
    assert_eq!(account["email"], EMAIL);
    assert_eq!(account["is_admin"], false);

    // The session row holds the hash of the cookie, never the token.
    let token = session.trim_start_matches("starter_session=");
    let stored: bool =
        sqlx::query_scalar("select exists (select 1 from sessions where token = $1)")
            .bind(token_hash(token))
            .fetch_one(&pool)
            .await
            .expect("session");
    assert!(stored);
    let raw: bool = sqlx::query_scalar("select exists (select 1 from sessions where token = $1)")
        .bind(token)
        .fetch_one(&pool)
        .await
        .expect("session");
    assert!(!raw);
}

#[tokio::test]
async fn registration_validates_every_field_and_the_taken_email() {
    let pool = pool().await;
    clean(&pool, EMAIL).await;
    let app = app().await;

    let empty = send_json(&app, Method::POST, "/api/register", &[], json!({})).await;
    assert_eq!(empty.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json(empty).await;
    assert_eq!(keys(&body), ["errors", "message"]);
    assert_eq!(body["message"], "A name is required. (and 2 more errors)");
    assert_eq!(
        body["errors"],
        json!({
            "name": ["A name is required."],
            "email": ["An email is required."],
            "password": ["The password must be at least 8 characters."],
        })
    );

    let bad = send_json(
        &app,
        Method::POST,
        "/api/register",
        &[],
        json!({
            "name": "Ada",
            "email": "not-an-email",
            "password": PASSWORD,
            "password_confirmation": "something else",
        }),
    )
    .await;
    assert_eq!(bad.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(bad).await["errors"],
        json!({
            "email": ["Enter a valid email address."],
            "password_confirmation": ["The password confirmation does not match."],
        })
    );

    register(&app, "Ada", EMAIL).await;
    let taken = send_json(
        &app,
        Method::POST,
        "/api/register",
        &[],
        json!({ "name": "Other", "email": "ADA@example.com", "password": PASSWORD }),
    )
    .await;
    assert_eq!(taken.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        json(taken).await,
        json!({
            "message": "This email is already registered.",
            "errors": { "email": ["This email is already registered."] },
        })
    );
    let accounts: i64 = sqlx::query_scalar("select count(*) from users where email = $1")
        .bind(EMAIL)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(accounts, 1);
}

#[tokio::test]
async fn logging_in_opens_a_fresh_session_and_rejects_bad_credentials() {
    let pool = pool().await;
    let app = app().await;
    let first = register(&app, "Ada", EMAIL).await;

    // A browser form gets the redirect; the case of the email is ignored.
    let login = send_json(
        &app,
        Method::POST,
        "/api/login",
        &[("accept", "text/html")],
        json!({ "email": "ADA@example.com", "password": PASSWORD }),
    )
    .await;
    assert_eq!(login.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&login), "/dashboard");
    assert_eq!(
        flash_from(&login).expect("flash"),
        json!({ "title": "Signed in", "body": "Welcome back, Ada!", "type": "success" })
    );
    let second = session_header(&login);
    assert_ne!(first, second);

    // Logging in on top of a session replaces it.
    let third = send_json(
        &app,
        Method::POST,
        "/api/login",
        &[("cookie", &second)],
        json!({ "email": EMAIL, "password": PASSWORD }),
    )
    .await;
    assert_eq!(third.status(), StatusCode::OK);
    let replaced = session_header(&third);
    assert_eq!(me(&app, &second).await, None);
    assert_eq!(
        me(&app, &replaced).await.expect("signed in")["email"],
        EMAIL
    );

    for (email, password) in [
        (EMAIL, "wrong password"),
        ("nobody@example.com", PASSWORD),
        ("", ""),
    ] {
        let rejected = send_json(
            &app,
            Method::POST,
            "/api/login",
            &[],
            json!({ "email": email, "password": password }),
        )
        .await;
        assert_eq!(
            rejected.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "{email}"
        );
        assert!(cookie_from(&rejected, "starter_session").is_none());
        assert_eq!(
            json(rejected).await,
            json!({
                "message": "These credentials do not match our records.",
                "errors": { "email": ["These credentials do not match our records."] },
            }),
            "{email}"
        );
    }

    let sessions: i64 = sqlx::query_scalar(
        "select count(*) from sessions s join users u on u.id = s.user_id where u.email = $1",
    )
    .bind(EMAIL)
    .fetch_one(&pool)
    .await
    .expect("sessions");
    assert_eq!(
        sessions, 2,
        "the first registration session plus the live login"
    );
}

#[tokio::test]
async fn logging_out_ends_the_session() {
    let pool = pool().await;
    let app = app().await;
    let session = register(&app, "Ada", EMAIL).await;

    let logout = send_json(
        &app,
        Method::POST,
        "/api/logout",
        &[("cookie", &session)],
        json!({}),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::OK);
    assert_eq!(json(logout).await, json!({ "redirect": "/" }));

    let browser = send_to(&app, Method::POST, "/api/logout", &[("cookie", &session)]).await;
    assert_eq!(browser.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&browser), "/");
    assert!(clears_cookie(&browser, "starter_session"));

    assert_eq!(me(&app, &session).await, None);
    let token = session.trim_start_matches("starter_session=");
    let alive: bool = sqlx::query_scalar("select exists (select 1 from sessions where token = $1)")
        .bind(token_hash(token))
        .fetch_one(&pool)
        .await
        .expect("session");
    assert!(!alive);
}

#[tokio::test]
async fn guests_and_expired_sessions_are_guests() {
    let pool = pool().await;
    let app = app().await;
    let guest = send_to(&app, Method::GET, "/api/me", &[]).await;
    assert_eq!(guest.status(), StatusCode::UNAUTHORIZED);
    let body = json(guest).await;
    assert_eq!(keys(&body), ["message"]);
    assert_eq!(body["message"], "Unauthenticated.");
    assert_eq!(me(&app, "starter_session=not-a-token").await, None);

    let session = register(&app, "Ada", EMAIL).await;
    let token = session.trim_start_matches("starter_session=");
    sqlx::query("update sessions set expires_at = now() - interval '1 second' where token = $1")
        .bind(token_hash(token))
        .execute(&pool)
        .await
        .expect("expire");
    assert_eq!(me(&app, &session).await, None);
}

#[tokio::test]
async fn configured_admin_emails_become_admins_on_registration_and_login() {
    let pool = pool().await;
    clean(&pool, EMAIL).await;
    let plain = app().await;
    let admin_app = app_with(config_with(&[("ADMIN_EMAILS", "Ada@example.com")])).await;

    // Registered before the listing: promoted on the next login.
    let session = register(&plain, "Ada", EMAIL).await;
    assert_eq!(
        me(&plain, &session).await.expect("signed in")["is_admin"],
        false
    );
    let login = send_json(
        &admin_app,
        Method::POST,
        "/api/login",
        &[],
        json!({ "email": EMAIL, "password": PASSWORD }),
    )
    .await;
    let session = session_header(&login);
    assert_eq!(
        me(&plain, &session).await.expect("signed in")["is_admin"],
        true
    );

    // Registered under the listing: an admin from the start.
    clean(&pool, EMAIL).await;
    let response = send_json(
        &admin_app,
        Method::POST,
        "/api/register",
        &[],
        json!({ "name": "Ada", "email": EMAIL, "password": PASSWORD }),
    )
    .await;
    let session = session_header(&response);
    assert_eq!(
        me(&plain, &session).await.expect("signed in")["is_admin"],
        true
    );
}

#[tokio::test]
async fn production_cookies_are_secure() {
    let pool = pool().await;
    clean(&pool, EMAIL).await;
    let production = app_with(config_with(&[("APP_ENV", "production")])).await;
    let response = send_json(
        &production,
        Method::POST,
        "/api/register",
        &[],
        json!({ "name": "Ada", "email": EMAIL, "password": PASSWORD }),
    )
    .await;
    let cookies: Vec<&str> = response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().expect("cookie"))
        .collect();
    assert_eq!(cookies.len(), 2, "the flash and the session");
    for cookie in cookies {
        assert!(cookie.ends_with("; Secure"), "{cookie}");
        assert!(cookie.contains("HttpOnly"), "{cookie}");
        assert!(cookie.contains("SameSite=Lax"), "{cookie}");
    }
}
