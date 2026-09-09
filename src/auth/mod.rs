//! Session authentication behind one facade. Handlers and features ask
//! [`Auth`] who is signed in (`user`, `is_logged_in`), register and log
//! accounts in and out, reset and update credentials; the cookie format,
//! the password hashing, the session and reset tables and the mail stay
//! behind it.

pub mod password;
pub mod reset;
pub mod session;
pub mod user;

use std::fmt;
use std::sync::Arc;

use axum::http::HeaderMap;
use sqlx::PgPool;

use crate::config::Config;
use crate::mail::{Mailer, Message};
pub use user::User;

/// Where the reset mail's link lands, a SvelteKit page.
const RESET_PASSWORD_PATH: &str = "/reset-password";
const RESET_MAIL_SUBJECT: &str = "Reset your password";

/// Postgres' unique_violation SQLSTATE: a second account with the same
/// email lost the race with the pre-check.
const PG_UNIQUE_VIOLATION: &str = "23505";

/// Bounds for the profile fields.
const MAX_NAME_LENGTH: usize = 100;
const MAX_EMAIL_LENGTH: usize = 254;

/// A field-level validation message, `(field, message)`, in the shape
/// `server::api::validation_errors` answers.
pub type FieldError = (String, String);

#[derive(Debug)]
pub enum AuthError {
    /// The input was rejected; every failing field carries its message.
    Validation(Vec<FieldError>),
    /// Unknown email or wrong password. Deliberately one error for both,
    /// so the login form cannot be used to probe for accounts.
    InvalidCredentials,
    Database(sqlx::Error),
    /// Hashing failed, which only happens with a broken hasher setup.
    Hash(argon2::password_hash::Error),
    Mail(crate::mail::MailError),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Validation(errors) => {
                write!(f, "validation failed:")?;
                for (field, message) in errors {
                    write!(f, " {field}: {message}")?;
                }
                Ok(())
            }
            AuthError::InvalidCredentials => write!(f, "invalid credentials"),
            AuthError::Database(error) => write!(f, "database: {error}"),
            AuthError::Hash(error) => write!(f, "password hashing: {error}"),
            AuthError::Mail(error) => write!(f, "mail: {error}"),
        }
    }
}

impl From<crate::mail::MailError> for AuthError {
    fn from(error: crate::mail::MailError) -> Self {
        AuthError::Mail(error)
    }
}

impl std::error::Error for AuthError {}

impl From<sqlx::Error> for AuthError {
    fn from(error: sqlx::Error) -> Self {
        AuthError::Database(error)
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(error: argon2::password_hash::Error) -> Self {
        AuthError::Hash(error)
    }
}

/// What a new account is registered with, the body of `POST /api/register`.
#[derive(Debug, Clone, Default, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Registration {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    /// When present it must repeat the password.
    #[serde(default)]
    #[ts(optional)]
    pub password_confirmation: Option<String>,
}

/// A successful login: the account and the session token to set as the
/// cookie (see [`Auth::session_cookie`]).
#[derive(Debug, Clone)]
pub struct Login {
    pub user: User,
    pub token: String,
}

#[derive(Clone)]
pub struct Auth {
    pool: PgPool,
    config: Arc<Config>,
    mailer: Mailer,
}

impl Auth {
    pub fn new(pool: PgPool, config: Arc<Config>, mailer: Mailer) -> Self {
        Self {
            pool,
            config,
            mailer,
        }
    }

    /// The signed-in account behind the request's session cookie, if any.
    pub async fn user(&self, headers: &HeaderMap) -> sqlx::Result<Option<User>> {
        let Some(session) =
            session::session_from_headers(&self.pool, headers, &self.session_cookie_name()).await?
        else {
            return Ok(None);
        };
        user::find(&self.pool, session.user_id).await
    }

    pub async fn is_logged_in(&self, headers: &HeaderMap) -> bool {
        matches!(self.user(headers).await, Ok(Some(_)))
    }

    /// Creates the account and opens its first session.
    pub async fn register(&self, input: Registration) -> Result<Login, AuthError> {
        let name = input.name.trim().to_owned();
        let email = normalize_email(&input.email);
        let mut errors = Vec::new();
        errors.extend(validate_name(&name));
        errors.extend(validate_email(&email));
        errors.extend(password::validate(
            &input.password,
            input.password_confirmation.as_deref(),
        ));
        if errors.is_empty() && user::find_by_email(&self.pool, &email).await?.is_some() {
            errors.push(taken_email());
        }
        if !errors.is_empty() {
            return Err(AuthError::Validation(errors));
        }

        let hash = password::hash(&input.password)?;
        let is_admin = self.config.is_admin_email(&email);
        let user = match user::insert(&self.pool, &name, &email, &hash, is_admin).await {
            Ok(user) => user,
            Err(error) if is_unique_violation(&error) => {
                return Err(AuthError::Validation(vec![taken_email()]));
            }
            Err(error) => return Err(error.into()),
        };
        let token = self.open_session(user.id).await?;
        Ok(Login { user, token })
    }

    /// Checks the credentials and opens a session. Configured admin
    /// emails are promoted here too, so a listing added later takes
    /// effect on the next login.
    pub async fn login(&self, email: &str, password: &str) -> Result<Login, AuthError> {
        let email = normalize_email(email);
        let credentials = user::find_credentials(&self.pool, &email).await?;
        // The hash is verified even for an unknown email so the response
        // time does not reveal whether the account exists.
        let (user, hash) = match credentials {
            Some((user, hash)) => (Some(user), hash),
            None => (None, password::DUMMY_HASH.to_owned()),
        };
        let matches = password::verify(&hash, password);
        let Some(mut user) = user.filter(|_| matches) else {
            return Err(AuthError::InvalidCredentials);
        };
        if !user.is_admin && self.config.is_admin_email(&user.email) {
            user::set_admin(&self.pool, user.id, true).await?;
            user.is_admin = true;
        }
        let token = self.open_session(user.id).await?;
        Ok(Login { user, token })
    }

    /// Ends the request's session; a missing or dead session is not an
    /// error.
    pub async fn logout(&self, headers: &HeaderMap) -> sqlx::Result<()> {
        if let Some(token) = session::cookie_value(headers, &self.session_cookie_name()) {
            session::delete_session(&self.pool, &token).await?;
        }
        Ok(())
    }

    /// Ends every session of the account (a password change, a deletion).
    pub async fn logout_everywhere(&self, user_id: i64) -> sqlx::Result<()> {
        session::delete_user_sessions(&self.pool, user_id).await
    }

    pub async fn update_profile(
        &self,
        user: &User,
        name: &str,
        email: &str,
    ) -> Result<User, AuthError> {
        let name = name.trim().to_owned();
        let email = normalize_email(email);
        let mut errors = Vec::new();
        errors.extend(validate_name(&name));
        errors.extend(validate_email(&email));
        if errors.is_empty()
            && email != user.email
            && user::find_by_email(&self.pool, &email).await?.is_some()
        {
            errors.push(taken_email());
        }
        if !errors.is_empty() {
            return Err(AuthError::Validation(errors));
        }
        match user::update_profile(&self.pool, user.id, &name, &email).await {
            Ok(updated) => Ok(updated),
            Err(error) if is_unique_violation(&error) => {
                Err(AuthError::Validation(vec![taken_email()]))
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Replaces the password after checking the current one. Other
    /// sessions of the account are ended; the caller keeps its own by
    /// opening a fresh one with the returned token.
    pub async fn change_password(
        &self,
        user: &User,
        current: &str,
        new: &str,
        confirmation: Option<&str>,
    ) -> Result<String, AuthError> {
        let mut errors = Vec::new();
        if !self.password_matches(user.id, current).await? {
            errors.push((
                "current_password".to_owned(),
                "The password is incorrect.".to_owned(),
            ));
        }
        errors.extend(password::validate(new, confirmation));
        if !errors.is_empty() {
            return Err(AuthError::Validation(errors));
        }
        user::update_password_hash(&self.pool, user.id, &password::hash(new)?).await?;
        self.logout_everywhere(user.id).await?;
        Ok(self.open_session(user.id).await?)
    }

    /// Deletes the account after checking its password; sessions cascade.
    pub async fn delete_account(&self, user: &User, password: &str) -> Result<(), AuthError> {
        if !self.password_matches(user.id, password).await? {
            return Err(AuthError::Validation(vec![(
                "password".to_owned(),
                "The password is incorrect.".to_owned(),
            )]));
        }
        user::delete(&self.pool, user.id).await?;
        Ok(())
    }

    /// Mails a reset link when the email belongs to an account. An
    /// unknown email answers the same success, so the form cannot be used
    /// to probe for accounts; only a malformed email is rejected.
    pub async fn request_password_reset(&self, email: &str) -> Result<(), AuthError> {
        let email = normalize_email(email);
        if let Some(error) = validate_email(&email) {
            return Err(AuthError::Validation(vec![error]));
        }
        let Some(user) = user::find_by_email(&self.pool, &email).await? else {
            return Ok(());
        };
        let token = session::random_token();
        reset::store(
            &self.pool,
            &email,
            &token,
            self.config.password_reset_lifetime_minutes,
        )
        .await?;
        let link = format!(
            "{}{RESET_PASSWORD_PATH}?token={token}&email={}",
            self.config.app_url,
            crate::encoding::percent_encode(&email)
        );
        self.mailer
            .send(Message {
                to: email,
                subject: RESET_MAIL_SUBJECT.to_owned(),
                text: format!(
                    "Hello {},\n\nOpen this link to choose a new password for {}:\n\n{link}\n\nThe link works for {} minutes. If you did not ask for a reset, ignore this mail.\n",
                    user.name, self.config.app_name, self.config.password_reset_lifetime_minutes
                ),
            })
            .await?;
        Ok(())
    }

    /// Sets the password from a reset link; the link dies with it and
    /// every session of the account ends.
    pub async fn reset_password(
        &self,
        email: &str,
        token: &str,
        password: &str,
        confirmation: Option<&str>,
    ) -> Result<(), AuthError> {
        let email = normalize_email(email);
        let mut errors = password::validate(password, confirmation);
        let user = user::find_by_email(&self.pool, &email).await?;
        let valid = match &user {
            Some(_) => reset::is_valid(&self.pool, &email, token).await?,
            None => false,
        };
        if !valid {
            errors.insert(
                0,
                (
                    "email".to_owned(),
                    "This password reset link is invalid or has expired.".to_owned(),
                ),
            );
        }
        if !errors.is_empty() {
            return Err(AuthError::Validation(errors));
        }
        let user = user.expect("checked above");
        user::update_password_hash(&self.pool, user.id, &password::hash(password)?).await?;
        reset::delete(&self.pool, &email).await?;
        self.logout_everywhere(user.id).await?;
        Ok(())
    }

    async fn password_matches(&self, user_id: i64, password: &str) -> sqlx::Result<bool> {
        let hash = user::password_hash(&self.pool, user_id).await?;
        Ok(hash.is_some_and(|hash| password::verify(&hash, password)))
    }

    async fn open_session(&self, user_id: i64) -> sqlx::Result<String> {
        session::create_session(&self.pool, user_id, self.config.session_lifetime_days).await
    }

    /// The name of the session cookie, `<app>_session`.
    pub fn session_cookie_name(&self) -> String {
        self.config.session_cookie_name()
    }

    /// The `Set-Cookie` value carrying a login.
    pub fn session_cookie(&self, token: &str) -> String {
        session::session_cookie(
            &self.session_cookie_name(),
            token,
            self.config.session_lifetime_days,
            self.config.secure_cookies(),
        )
    }

    /// The `Set-Cookie` value ending a login.
    pub fn clear_session_cookie(&self) -> String {
        session::clear_cookie(&self.session_cookie_name(), self.config.secure_cookies())
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .and_then(|db_error| db_error.code())
        .is_some_and(|code| code == PG_UNIQUE_VIOLATION)
}

fn taken_email() -> FieldError {
    (
        "email".to_owned(),
        "This email is already registered.".to_owned(),
    )
}

/// Emails are compared case-insensitively; the lowercased, trimmed form
/// is what the database stores.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_name(name: &str) -> Option<FieldError> {
    if name.is_empty() {
        Some(("name".to_owned(), "A name is required.".to_owned()))
    } else if name.chars().count() > MAX_NAME_LENGTH {
        Some((
            "name".to_owned(),
            format!("The name may not be longer than {MAX_NAME_LENGTH} characters."),
        ))
    } else {
        None
    }
}

/// The shape check a mail server would reject anyway: one `@`, a local
/// part, and a domain with a dot, no whitespace.
pub fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !email.chars().any(char::is_whitespace)
        && !domain.contains('@')
}

fn validate_email(email: &str) -> Option<FieldError> {
    if email.is_empty() {
        Some(("email".to_owned(), "An email is required.".to_owned()))
    } else if email.len() > MAX_EMAIL_LENGTH || !is_valid_email(email) {
        Some((
            "email".to_owned(),
            "Enter a valid email address.".to_owned(),
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_are_normalized_and_shape_checked() {
        assert_eq!(normalize_email("  Ada@Example.COM "), "ada@example.com");
        assert!(is_valid_email("ada@example.com"));
        assert!(is_valid_email("first.last+tag@sub.example.co"));
        assert!(!is_valid_email("ada"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("ada@"));
        assert!(!is_valid_email("ada@localhost"));
        assert!(!is_valid_email("ada@.example.com"));
        assert!(!is_valid_email("a da@example.com"));
        assert!(!is_valid_email("a@b@example.com"));
    }

    #[test]
    fn profile_fields_have_messages() {
        assert_eq!(
            validate_name("").map(|(field, _)| field).as_deref(),
            Some("name")
        );
        assert!(validate_name(&"x".repeat(MAX_NAME_LENGTH)).is_none());
        assert!(validate_name(&"x".repeat(MAX_NAME_LENGTH + 1)).is_some());
        assert_eq!(
            validate_email(""),
            Some(("email".to_owned(), "An email is required.".to_owned()))
        );
        assert_eq!(
            validate_email("nope"),
            Some((
                "email".to_owned(),
                "Enter a valid email address.".to_owned()
            ))
        );
        assert!(validate_email("ada@example.com").is_none());
    }
}
