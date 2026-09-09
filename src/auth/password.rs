//! Password hashing with Argon2id (the OWASP-recommended parameters the
//! crate defaults to), and the password rules.

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand::RngExt;

use super::FieldError;

pub const MIN_LENGTH: usize = 8;

/// Argon2 accepts longer inputs; the cap keeps a hostile client from
/// making the server hash megabytes.
pub const MAX_LENGTH: usize = 512;

/// A hash of an unguessable password, verified against when the email of
/// a login attempt is unknown, so both outcomes take the same time.
pub const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c3RhcnRlci1kdW1teS1zYWx0$o1p9Q3Z0aIhWQpSXVwKb0y2n9hR2yTtmqQ8dJqNUM5c";

/// The recommended salt size for Argon2.
const SALT_BYTES: usize = 16;

pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let mut bytes = [0u8; SALT_BYTES];
    rand::rng().fill(&mut bytes[..]);
    let salt = SaltString::encode_b64(&bytes)?;
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// Whether the password produced the stored hash; a malformed hash is a
/// mismatch rather than an error.
pub fn verify(hash: &str, password: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

/// The rules a new password must meet, as field errors.
pub fn validate(password: &str, confirmation: Option<&str>) -> Vec<FieldError> {
    let mut errors = Vec::new();
    let length = password.chars().count();
    if length < MIN_LENGTH {
        errors.push((
            "password".to_owned(),
            format!("The password must be at least {MIN_LENGTH} characters."),
        ));
    } else if length > MAX_LENGTH {
        errors.push((
            "password".to_owned(),
            format!("The password may not be longer than {MAX_LENGTH} characters."),
        ));
    }
    if let Some(confirmation) = confirmation
        && confirmation != password
    {
        errors.push((
            "password_confirmation".to_owned(),
            "The password confirmation does not match.".to_owned(),
        ));
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_verify_their_own_password_only() {
        let hashed = hash("correct horse battery staple").expect("hash");
        assert!(hashed.starts_with("$argon2id$"));
        assert!(verify(&hashed, "correct horse battery staple"));
        assert!(!verify(&hashed, "Correct horse battery staple"));
        assert!(!verify("not a hash", "anything"));
        assert_ne!(hashed, hash("correct horse battery staple").expect("hash"));
    }

    #[test]
    fn the_dummy_hash_is_well_formed_and_matches_nothing_useful() {
        assert!(PasswordHash::new(DUMMY_HASH).is_ok());
        assert!(!verify(DUMMY_HASH, ""));
        assert!(!verify(DUMMY_HASH, "password"));
    }

    #[test]
    fn the_rules_report_each_failure() {
        assert!(validate("long enough", None).is_empty());
        assert!(validate("long enough", Some("long enough")).is_empty());
        assert_eq!(
            validate("short", None),
            vec![(
                "password".to_owned(),
                "The password must be at least 8 characters.".to_owned()
            )]
        );
        assert_eq!(
            validate("long enough", Some("different")),
            vec![(
                "password_confirmation".to_owned(),
                "The password confirmation does not match.".to_owned()
            )]
        );
        assert_eq!(validate(&"x".repeat(MAX_LENGTH + 1), None).len(), 1);
        assert_eq!(validate("short", Some("other")).len(), 2);
    }
}
