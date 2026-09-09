//! The application configuration: one typed struct read from the
//! environment (after `.env` is loaded by `main`), with a working local
//! default for everything. Every setting has a documented variable in
//! `.env.example`; nothing else in the crate reads `std::env`.

use std::fmt;

/// `APP_ENV`: `local` on a developer machine, anything else (or unset)
/// is production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Local,
    Production,
}

impl AppEnv {
    fn parse(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("local") => AppEnv::Local,
            _ => AppEnv::Production,
        }
    }

    pub fn is_local(self) -> bool {
        self == AppEnv::Local
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// The display name, shown by the frontend (which reads
    /// `PUBLIC_APP_NAME` from the same `.env`) and the prefix of the
    /// app's cookies.
    pub app_name: String,
    pub app_env: AppEnv,
    /// The public origin the app is served on; absolute links and the
    /// production reverse proxy read it.
    pub app_url: String,
    /// The address the JSON API listens on.
    pub bind_addr: String,
    pub database_url: String,
    pub database_max_connections: u32,
    /// How long a login lasts.
    pub session_lifetime_days: i64,
    /// Emails (lowercased) whose accounts are site admins.
    pub admin_emails: Vec<String>,
    /// How long a password reset link works.
    pub password_reset_lifetime_minutes: i64,
    /// The sender of every mail, `Name <address>`.
    pub mail_from: String,
    /// SMTP when set; the mailer logs messages otherwise.
    pub smtp: Option<SmtpConfig>,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub const DEFAULT_APP_NAME: &str = "Starter";

/// The local docker-compose Postgres (see `docker-compose.yml`). Port 5436
/// so it coexists with other stacks' Postgres containers on one machine.
pub const DEFAULT_DATABASE_URL: &str = "postgres://starter:starter@127.0.0.1:5436/starter";

/// The shared dev origin: the SvelteKit dev server (or the composed
/// Caddy) on this port proxies backend paths to the API.
pub const DEFAULT_APP_URL: &str = "http://localhost:8787";

/// 3200 rather than axum's customary 3000 so it coexists with other
/// stacks on one machine.
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3200";

/// Enough for the web handlers plus background work without exhausting
/// Postgres' default connection limit alongside tests and tooling.
const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 10;

/// A "remember me" login.
const DEFAULT_SESSION_LIFETIME_DAYS: i64 = 30;

/// Long enough to read the mail, short enough for a leaked link to be
/// worthless soon.
const DEFAULT_PASSWORD_RESET_LIFETIME_MINUTES: i64 = 60;

/// The submission port with STARTTLS.
const DEFAULT_SMTP_PORT: u16 = 587;

#[derive(Debug)]
pub struct ConfigError {
    pub variable: &'static str,
    pub value: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cannot be {:?}", self.variable, self.value)
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Reads the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Builds the configuration from any name-to-value lookup, so tests
    /// can pass a map instead of mutating the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let string = |name: &str, default: &str| {
            lookup(name)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default.to_owned())
        };
        fn number<T: std::str::FromStr>(
            value: Option<String>,
            variable: &'static str,
            default: T,
        ) -> Result<T, ConfigError> {
            match value.map(|value| value.trim().to_owned()) {
                None => Ok(default),
                Some(value) if value.is_empty() => Ok(default),
                Some(value) => value.parse().map_err(|_| ConfigError { variable, value }),
            }
        }

        let database_max_connections = number(
            lookup("DATABASE_MAX_CONNECTIONS"),
            "DATABASE_MAX_CONNECTIONS",
            DEFAULT_DATABASE_MAX_CONNECTIONS,
        )?;
        if database_max_connections == 0 {
            return Err(ConfigError {
                variable: "DATABASE_MAX_CONNECTIONS",
                value: "0".to_owned(),
            });
        }
        let session_lifetime_days = number(
            lookup("SESSION_LIFETIME_DAYS"),
            "SESSION_LIFETIME_DAYS",
            DEFAULT_SESSION_LIFETIME_DAYS,
        )?;
        if session_lifetime_days <= 0 {
            return Err(ConfigError {
                variable: "SESSION_LIFETIME_DAYS",
                value: session_lifetime_days.to_string(),
            });
        }

        let password_reset_lifetime_minutes = number(
            lookup("PASSWORD_RESET_LIFETIME_MINUTES"),
            "PASSWORD_RESET_LIFETIME_MINUTES",
            DEFAULT_PASSWORD_RESET_LIFETIME_MINUTES,
        )?;
        if password_reset_lifetime_minutes <= 0 {
            return Err(ConfigError {
                variable: "PASSWORD_RESET_LIFETIME_MINUTES",
                value: password_reset_lifetime_minutes.to_string(),
            });
        }
        let app_name = string("APP_NAME", DEFAULT_APP_NAME);
        let optional = |name: &str| {
            lookup(name)
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        };
        let smtp = match optional("SMTP_HOST") {
            Some(host) => Some(SmtpConfig {
                host,
                port: number(lookup("SMTP_PORT"), "SMTP_PORT", DEFAULT_SMTP_PORT)?,
                username: optional("SMTP_USERNAME"),
                password: optional("SMTP_PASSWORD"),
            }),
            None => None,
        };

        Ok(Self {
            mail_from: string("MAIL_FROM", &format!("{app_name} <no-reply@localhost>")),
            password_reset_lifetime_minutes,
            smtp,
            app_name,
            app_env: AppEnv::parse(lookup("APP_ENV").as_deref()),
            app_url: string("APP_URL", DEFAULT_APP_URL)
                .trim_end_matches('/')
                .to_owned(),
            bind_addr: string("BIND_ADDR", DEFAULT_BIND_ADDR),
            database_url: string("DATABASE_URL", DEFAULT_DATABASE_URL),
            database_max_connections,
            session_lifetime_days,
            admin_emails: parse_list(lookup("ADMIN_EMAILS").as_deref().unwrap_or_default()),
        })
    }

    /// Cookies carry `Secure` outside local development: production is
    /// https-only, so the browser must never send a token over plain http.
    pub fn secure_cookies(&self) -> bool {
        !self.app_env.is_local()
    }

    pub fn is_admin_email(&self, email: &str) -> bool {
        self.admin_emails.iter().any(|admin| admin == email)
    }

    /// The app name as a cookie prefix, so two apps built from this
    /// template on one localhost keep their cookies apart. The frontend
    /// derives the same prefix in `src/lib/app.ts`; keep both in step.
    pub fn cookie_prefix(&self) -> String {
        slug(&self.app_name)
    }

    pub fn session_cookie_name(&self) -> String {
        format!("{}_session", self.cookie_prefix())
    }

    pub fn flash_cookie_name(&self) -> String {
        format!("{}_flash", self.cookie_prefix())
    }
}

/// Lowercase letters and digits, every other run of characters a single
/// underscore, none at the ends: `"My App!"` is `my_app`.
pub fn slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    let slug = slug.trim_matches('_').to_owned();
    if slug.is_empty() {
        "app".to_owned()
    } else {
        slug
    }
}

/// A comma-separated list: entries trimmed and lowercased, empties dropped.
pub fn parse_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|entry| entry.trim().to_lowercase())
        .filter(|entry| !entry.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let map: HashMap<&str, &str> = vars.iter().copied().collect();
        Config::from_lookup(|name| map.get(name).map(|value| (*value).to_owned()))
    }

    #[test]
    fn everything_has_a_local_default() {
        let config = config(&[]).expect("config");
        assert_eq!(config.app_name, "Starter");
        assert_eq!(config.session_cookie_name(), "starter_session");
        assert_eq!(config.flash_cookie_name(), "starter_flash");
        assert_eq!(config.app_env, AppEnv::Production);
        assert!(config.secure_cookies());
        assert_eq!(config.app_url, DEFAULT_APP_URL);
        assert_eq!(config.bind_addr, DEFAULT_BIND_ADDR);
        assert_eq!(config.database_url, DEFAULT_DATABASE_URL);
        assert_eq!(config.database_max_connections, 10);
        assert_eq!(config.session_lifetime_days, 30);
        assert!(config.admin_emails.is_empty());
        assert_eq!(config.password_reset_lifetime_minutes, 60);
        assert_eq!(config.mail_from, "Starter <no-reply@localhost>");
        assert!(config.smtp.is_none());
    }

    #[test]
    fn smtp_is_on_once_a_host_is_set() {
        let parsed = config(&[
            ("SMTP_HOST", "smtp.example.com"),
            ("SMTP_USERNAME", "postmaster"),
            ("SMTP_PASSWORD", "secret"),
            ("MAIL_FROM", "Starter <hello@example.com>"),
        ])
        .expect("config");
        let smtp = parsed.smtp.expect("smtp");
        assert_eq!(smtp.host, "smtp.example.com");
        assert_eq!(smtp.port, 587);
        assert_eq!(smtp.username.as_deref(), Some("postmaster"));
        assert_eq!(smtp.password.as_deref(), Some("secret"));
        assert_eq!(parsed.mail_from, "Starter <hello@example.com>");
        assert!(config(&[("SMTP_HOST", "x"), ("SMTP_PORT", "lots")]).is_err());
        assert!(config(&[("PASSWORD_RESET_LIFETIME_MINUTES", "0")]).is_err());
    }

    #[test]
    fn values_are_read_and_normalized() {
        let config = config(&[
            ("APP_ENV", "local"),
            ("APP_URL", "https://starter.example/ "),
            ("DATABASE_MAX_CONNECTIONS", " 4 "),
            ("SESSION_LIFETIME_DAYS", "7"),
            ("ADMIN_EMAILS", " Admin@Example.com, , second@example.com"),
        ])
        .expect("config");
        assert!(config.app_env.is_local());
        assert!(!config.secure_cookies());
        assert_eq!(config.app_url, "https://starter.example");
        assert_eq!(config.database_max_connections, 4);
        assert_eq!(config.session_lifetime_days, 7);
        assert_eq!(
            config.admin_emails,
            ["admin@example.com", "second@example.com"]
        );
        assert!(config.is_admin_email("admin@example.com"));
        assert!(!config.is_admin_email("nobody@example.com"));
    }

    #[test]
    fn bad_numbers_name_their_variable() {
        let error = config(&[("DATABASE_MAX_CONNECTIONS", "many")]).expect_err("error");
        assert_eq!(error.variable, "DATABASE_MAX_CONNECTIONS");
        assert_eq!(
            error.to_string(),
            "DATABASE_MAX_CONNECTIONS cannot be \"many\""
        );
        assert!(config(&[("DATABASE_MAX_CONNECTIONS", "0")]).is_err());
        assert!(config(&[("SESSION_LIFETIME_DAYS", "-1")]).is_err());
        // Blank values fall back to the default instead of failing.
        assert_eq!(
            config(&[("SESSION_LIFETIME_DAYS", "")])
                .expect("config")
                .session_lifetime_days,
            30
        );
    }

    #[test]
    fn names_become_cookie_prefixes() {
        assert_eq!(slug("My App!"), "my_app");
        assert_eq!(slug("  Some-Product 2 "), "some_product_2");
        assert_eq!(slug("Étoile"), "toile");
        assert_eq!(slug("!!!"), "app");
        let config = config(&[("APP_NAME", "Other App")]).expect("config");
        assert_eq!(config.session_cookie_name(), "other_app_session");
    }
}
