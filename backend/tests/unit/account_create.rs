use std::sync::Arc;

use axum::http::{header, HeaderMap, HeaderValue};
use secrecy::SecretString;
use sha1::{Digest, Sha1};

use digital_publication_backend::{
    domains::authentication::extractor::{
        parse_bearer_token, sha256_token_verifier, BearerAuthError,
    },
    shared::validation::{
        hash_password, normalize_email, PasswordBlocklist, PasswordPolicy,
        PasswordValidationError,
    },
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
    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        Vec::<[u8; 20]>::new(),
    );

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
    let blocklist = PasswordBlocklist::from_hashes(
        "test",
        Vec::<[u8; 20]>::new(),
    );

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
        SecretString::from("horse battery staple".to_owned());

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

#[test]
fn bearer_auth_extracts_authorization_header_token() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer opaque-token"),
    );

    assert_eq!(
        parse_bearer_token(&headers),
        Ok("opaque-token")
    );
}

#[test]
fn bearer_auth_scheme_is_case_insensitive() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("bEaReR opaque-token"),
    );

    assert_eq!(
        parse_bearer_token(&headers),
        Ok("opaque-token")
    );
}

#[test]
fn bearer_auth_distinguishes_missing_and_invalid_credentials() {
    let headers = HeaderMap::new();
    assert_eq!(
        parse_bearer_token(&headers),
        Err(BearerAuthError::Missing)
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Basic abc"),
    );

    assert_eq!(
        parse_bearer_token(&headers),
        Err(BearerAuthError::Invalid)
    );
}

#[test]
fn bearer_auth_rejects_malformed_or_multiple_credentials() {
    for value in [
        "",
        "Bearer",
        "Bearer ",
        "Bearer token extra",
        "Bearer\ttoken",
        "Basic token",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(value).unwrap(),
        );

        assert_eq!(
            parse_bearer_token(&headers),
            Err(BearerAuthError::Invalid),
            "value={value:?}"
        );
    }
}

#[test]
fn bearer_auth_uses_sha256_of_the_exact_token() {
    assert_eq!(
        sha256_token_verifier("hello world"),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}