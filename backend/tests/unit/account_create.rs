use std::sync::Arc;

use secrecy::SecretString;
use sha1::{Digest, Sha1};

use digital_publication_backend::shared::validation::{
    hash_password, normalize_email, PasswordBlocklist, PasswordPolicy,
    PasswordValidationError,
};

fn sha1_hash(password: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

#[test]
fn normalize_email_trims_whitespace_and_normalizes_domain() {
    let email = normalize_email("  Admin@BÜCHER.DE  ").unwrap();

    assert_eq!(
        email.canonical,
        "Admin@xn--bcher-kva.de"
    );

    assert_eq!(
        email.normalized,
        "admin@xn--bcher-kva.de"
    );
}

#[test]
fn normalize_email_preserves_provider_specific_local_part_semantics() {
    let email =
        normalize_email("Admin+Books@Example.COM").unwrap();

    assert_eq!(
        email.canonical,
        "Admin+Books@example.com"
    );

    assert_eq!(
        email.normalized,
        "admin+books@example.com"
    );
}

#[test]
fn normalize_email_is_case_insensitive_for_identity() {
    let first =
        normalize_email("Admin@Example.com").unwrap();

    let second =
        normalize_email("ADMIN@EXAMPLE.COM").unwrap();

    assert_eq!(first.normalized, second.normalized);
}

#[test]
fn normalize_email_supports_unicode_local_part() {
    let first =
        normalize_email("Straße@example.com").unwrap();

    let second =
        normalize_email("STRASSE@example.com").unwrap();

    assert_eq!(first.normalized, second.normalized);
}

#[test]
fn normalize_email_rejects_display_names() {
    assert!(
        normalize_email("Alice <alice@example.com>").is_err()
    );
}

#[test]
fn normalize_email_rejects_domain_literals() {
    assert!(
        normalize_email("alice@[127.0.0.1]").is_err()
    );
}

#[test]
fn normalize_email_rejects_email_longer_than_254_characters() {
    let local_part = "a".repeat(245);
    let email = format!("{local_part}@example.com");

    assert!(normalize_email(&email).is_err());
}

#[test]
fn password_policy_requires_15_characters() {
    let blocklist =
        PasswordBlocklist::from_hashes("test", std::iter::empty());

    let policy =
        PasswordPolicy::new(Arc::new(blocklist));

    let password = SecretString::from("12345678901234");

    assert_eq!(
        policy.validate(&password),
        Err(PasswordValidationError::TooShort)
    );
}

#[test]
fn password_policy_accepts_at_least_64_characters() {
    let blocklist =
        PasswordBlocklist::from_hashes("test", std::iter::empty());

    let policy =
        PasswordPolicy::new(Arc::new(blocklist));

    let password =
        SecretString::from("a".repeat(64));

    assert!(policy.validate(&password).is_ok());
}

#[test]
fn password_policy_rejects_exact_blocklist_match() {
    let blocked_password =
        "correct horse battery staple";

    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        [sha1_hash(blocked_password)],
    );

    let policy =
        PasswordPolicy::new(Arc::new(blocklist));

    let password =
        SecretString::from(blocked_password.to_owned());

    assert_eq!(
        policy.validate(&password),
        Err(PasswordValidationError::Blocklisted)
    );
}

#[test]
fn password_policy_does_not_reject_substrings_of_blocklisted_passwords() {
    let blocklisted_password =
        "correct horse battery staple";

    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        [sha1_hash(blocklisted_password)],
    );

    let policy =
        PasswordPolicy::new(Arc::new(blocklist));

    let password =
        SecretString::from("horse battery".to_owned());

    assert!(policy.validate(&password).is_ok());
}

#[test]
fn password_hash_is_argon2id() {
    let password =
        SecretString::from("a secure password with enough length");

    let encoded =
        hash_password(password.clone()).unwrap();

    assert!(
        encoded.starts_with("$argon2id$v=19$")
    );
}

#[test]
fn password_hash_is_not_deterministic() {
    let password =
        SecretString::from("a secure password with enough length");

    let first =
        hash_password(password.clone()).unwrap();

    let second =
        hash_password(password).unwrap();

    assert_ne!(first, second);
}