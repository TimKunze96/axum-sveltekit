//! Outbound mail behind one [`Mailer`]: SMTP when `SMTP_HOST` is set,
//! the log otherwise (the reset link then lands in the API's output,
//! which is what a developer wants), and an in-memory outbox for tests.

use std::fmt;
use std::sync::{Arc, Mutex};

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[derive(Debug)]
pub enum MailError {
    Address(lettre::address::AddressError),
    Build(lettre::error::Error),
    Smtp(lettre::transport::smtp::Error),
}

impl fmt::Display for MailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MailError::Address(error) => write!(f, "mail address: {error}"),
            MailError::Build(error) => write!(f, "mail message: {error}"),
            MailError::Smtp(error) => write!(f, "smtp: {error}"),
        }
    }
}

impl std::error::Error for MailError {}

/// The messages a capturing mailer received, for tests to read.
pub type Outbox = Arc<Mutex<Vec<Message>>>;

#[derive(Clone)]
pub enum Mailer {
    /// Writes every message to the log at info level.
    Log { from: String },
    Smtp {
        from: String,
        transport: AsyncSmtpTransport<Tokio1Executor>,
    },
    /// Keeps every message in memory instead of sending it.
    Capture { from: String, outbox: Outbox },
}

impl Mailer {
    /// SMTP when the configuration has a host, the log otherwise.
    pub fn from_config(config: &Config) -> Result<Self, MailError> {
        let from = config.mail_from.clone();
        let Some(smtp) = &config.smtp else {
            return Ok(Mailer::Log { from });
        };
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)
            .map_err(MailError::Smtp)?
            .port(smtp.port);
        if let (Some(username), Some(password)) = (&smtp.username, &smtp.password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        }
        Ok(Mailer::Smtp {
            from,
            transport: builder.build(),
        })
    }

    pub fn capture(from: &str) -> (Self, Outbox) {
        let outbox: Outbox = Arc::new(Mutex::new(Vec::new()));
        (
            Mailer::Capture {
                from: from.to_owned(),
                outbox: outbox.clone(),
            },
            outbox,
        )
    }

    pub async fn send(&self, message: Message) -> Result<(), MailError> {
        match self {
            Mailer::Log { from } => {
                tracing::info!(
                    from,
                    to = message.to,
                    subject = message.subject,
                    "mail:\n{}",
                    message.text
                );
                Ok(())
            }
            Mailer::Capture { outbox, .. } => {
                outbox.lock().expect("outbox lock").push(message);
                Ok(())
            }
            Mailer::Smtp { from, transport } => {
                let mail = lettre::Message::builder()
                    .from(from.parse().map_err(MailError::Address)?)
                    .to(message.to.parse().map_err(MailError::Address)?)
                    .subject(message.subject)
                    .header(ContentType::TEXT_PLAIN)
                    .body(message.text)
                    .map_err(MailError::Build)?;
                transport.send(mail).await.map_err(MailError::Smtp)?;
                Ok(())
            }
        }
    }
}
