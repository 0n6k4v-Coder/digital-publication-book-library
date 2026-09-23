use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use addr_spec::AddrSpec;
use argon2::Argon2;
use idna::domain_to_ascii_strict;
use secrecy::{ExposeSecret, SecretString};
use sha1::{Digest, Sha1};
use thiserror::Error;
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct NormalizedEmail {
    pub canonical: String,
    pub normalized: String,
}

pub fn normalize_email(input: &str) -> Result<NormalizedEmail, EmailValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(EmailValidationError::InvalidFormat);
    }

    if trimmed.chars().count() > 254 {
        return Err(EmailValidationError::TooLong);
    }

    let parsed = AddrSpec::from_str(trimmed).map_err(|_| EmailValidationError::InvalidFormat)?;

    if parsed.is_literal() {
        return Err(EmailValidationError::InvalidFormat);
    }

    let ascii_domain =
        domain_to_ascii_strict(parsed.domain()).map_err(|_| EmailValidationError::InvalidDomain)?;

    let canonical_address = AddrSpec::new(parsed.local_part(), ascii_domain)
        .map_err(|_| EmailValidationError::InvalidFormat)?;

    let canonical = canonical_address.to_string();

    if canonical.chars().count() > 254 {
        return Err(EmailValidationError::TooLong);
    }

    let local_nfd = parsed.local_part().nfd().collect::<String>();
    let local_casefolded = local_nfd.chars().case_fold().collect::<String>();
    let local_normalized = local_casefolded.nfd().collect::<String>();

    let normalized = format!("{local_normalized}@{ascii_domain}");

    Ok(NormalizedEmail {
        canonical,
        normalized,
    })
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EmailValidationError {
    #[error("invalid email format")]
    InvalidFormat,
    #[error("email address is too long")]
    TooLong,
    #[error("invalid email domain")]
    InvalidDomain,
}

#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    blocklist: Arc<PasswordBlocklist>,
}

impl PasswordPolicy {
    pub fn new(blocklist: Arc<PasswordBlocklist>) -> Self {
        Self { blocklist }
    }

    pub fn validate(&self, password: &SecretString) -> Result<(), PasswordValidationError> {
        if password.expose_secret().chars().count() < 15 {
            return Err(PasswordValidationError::TooShort);
        }

        if self.blocklist.contains(password.expose_secret()) {
            return Err(PasswordValidationError::Blocklisted);
        }

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PasswordValidationError {
    #[error("password must contain at least 15 characters")]
    TooShort,
    #[error("password is not allowed")]
    Blocklisted,
}

#[derive(Debug, Clone)]
pub struct PasswordBlocklist {
    version: Arc<str>,
    hashes: Arc<HashSet<[u8; 20]>>,
}

impl PasswordBlocklist {
    pub fn from_file(path: &Path) -> Result<Self, PasswordBlocklistLoadError> {
        let file = File::open(path).map_err(|source| PasswordBlocklistLoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let mut version = None;
        let mut hashes = HashSet::new();
        let reader = BufReader::new(file);

        for (line_number, line) in reader.lines().enumerate() {
            let line_number = line_number + 1;

            let line = line.map_err(|source| PasswordBlocklistLoadError::Io {
                path: path.to_path_buf(),
                source,
            })?;

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(value) = line.strip_prefix("# version=") {
                if value.is_empty() || version.is_some() {
                    return Err(PasswordBlocklistLoadError::Invalid {
                        line: line_number,
                        reason: "exactly one non-empty version header is required",
                    });
                }

                version = Some(value.to_owned());
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            let hash_text = line.split_once(':').map_or(line, |(hash, _count)| hash);

            if hash_text.len() != 40 {
                return Err(PasswordBlocklistLoadError::Invalid {
                    line: line_number,
                    reason: "password hash must contain exactly 40 hexadecimal characters",
                });
            }

            let mut bytes = [0u8; 20];

            for (index, slot) in bytes.iter_mut().enumerate() {
                let start = index * 2;

                *slot = u8::from_str_radix(&hash_text[start..start + 2], 16).map_err(|_| {
                    PasswordBlocklistLoadError::Invalid {
                        line: line_number,
                        reason: "password hash must be hexadecimal",
                    }
                })?;
            }

            hashes.insert(bytes);
        }

        let version = version.ok_or(PasswordBlocklistLoadError::Invalid {
            line: 0,
            reason: "a # version=<value> header is required",
        })?;

        if hashes.is_empty() {
            return Err(PasswordBlocklistLoadError::Invalid {
                line: 0,
                reason: "password blocklist must contain at least one hash",
            });
        }

        Ok(Self {
            version: Arc::from(version),
            hashes: Arc::new(hashes),
        })
    }

    pub fn from_hashes<I, H>(version: impl Into<String>, hashes: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: Into<[u8; 20]>,
    {
        Self {
            version: Arc::from(version.into()),
            hashes: Arc::new(hashes.into_iter().map(Into::into).collect()),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn contains(&self, password: &str) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());

        let digest: [u8; 20] = hasher.finalize().into();

        self.hashes.contains(&digest)
    }
}

#[derive(Debug, Error)]
pub enum PasswordBlocklistLoadError {
    #[error("failed to read password blocklist {path}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("invalid password blocklist at line {line}: {reason}")]
    Invalid { line: usize, reason: &'static str },
}

pub fn hash_password(password: SecretString) -> Result<String, argon2::password_hash::Error> {
    use argon2::password_hash::PasswordHasher;

    Argon2::default()
        .hash_password(password.expose_secret().as_bytes())
        .map(|hash| hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(password: &str) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        hasher.finalize().into()
    }

    #[test]
    fn normalizes_email_without_provider_specific_changes() {
        let normalized = normalize_email("  Admin+Tag@BÜCHER.DE  ").unwrap();

        assert_eq!(normalized.canonical, "Admin+Tag@xn--bcher-kva.de");
        assert_eq!(normalized.normalized, "admin+tag@xn--bcher-kva.de");
    }

    #[test]
    fn casefolds_unicode_local_part_for_identity_comparison() {
        let first = normalize_email("Straße@example.com").unwrap();
        let second = normalize_email("STRASSE@example.com").unwrap();

        assert_eq!(first.normalized, second.normalized);
    }

    #[test]
    fn rejects_display_names_and_domain_literals() {
        assert!(normalize_email("Alice <alice@example.com>").is_err());
        assert!(normalize_email("alice@[127.0.0.1]").is_err());
    }

    #[test]
    fn password_policy_supports_64_characters() {
        let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

        let policy = PasswordPolicy::new(Arc::new(blocklist));
        let password = SecretString::from("a".repeat(64));

        assert!(policy.validate(&password).is_ok());
    }

    #[test]
    fn password_policy_blocks_exact_hash_matches() {
        let blocklist =
            PasswordBlocklist::from_hashes("test", [hash("correct horse battery staple")]);

        let policy = PasswordPolicy::new(Arc::new(blocklist));
        let password = SecretString::from("correct horse battery staple".to_owned());

        assert_eq!(
            policy.validate(&password),
            Err(PasswordValidationError::Blocklisted)
        );
    }

    #[test]
    fn hashes_password_with_argon2id() {
        use argon2::password_hash::{PasswordHash, PasswordVerifier};

        let password = SecretString::from("a secure password with enough length".to_owned());

        let encoded = hash_password(password.clone()).unwrap();

        assert!(encoded.starts_with("$argon2id$v=19$"));

        Argon2::default()
            .verify_password(
                password.expose_secret().as_bytes(),
                &PasswordHash::new(&encoded).unwrap(),
            )
            .unwrap();
    }
}
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use addr_spec::AddrSpec;
use argon2::Argon2;
use idna::domain_to_ascii_strict;
use secrecy::{ExposeSecret, SecretString};
use sha1::{Digest, Sha1};
use thiserror::Error;
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct NormalizedEmail {
    pub canonical: String,
    pub normalized: String,
}

pub fn normalize_email(input: &str) -> Result<NormalizedEmail, EmailValidationError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(EmailValidationError::InvalidFormat);
    }

    if trimmed.chars().count() > 254 {
        return Err(EmailValidationError::TooLong);
    }

    let parsed = AddrSpec::from_str(trimmed).map_err(|_| EmailValidationError::InvalidFormat)?;

    if parsed.is_literal() {
        return Err(EmailValidationError::InvalidFormat);
    }

    let ascii_domain =
        domain_to_ascii_strict(parsed.domain()).map_err(|_| EmailValidationError::InvalidDomain)?;

    let canonical_address = AddrSpec::new(parsed.local_part(), &ascii_domain)
        .map_err(|_| EmailValidationError::InvalidFormat)?;

    let canonical = canonical_address.to_string();

    if canonical.chars().count() > 254 {
        return Err(EmailValidationError::TooLong);
    }

    let local_nfd = parsed.local_part().nfd().collect::<String>();
    let local_casefolded = local_nfd.chars().case_fold().collect::<String>();
    let local_normalized = local_casefolded.nfd().collect::<String>();

    let normalized = format!("{local_normalized}@{ascii_domain}");

    Ok(NormalizedEmail {
        canonical,
        normalized,
    })
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EmailValidationError {
    #[error("invalid email format")]
    InvalidFormat,
    #[error("email address is too long")]
    TooLong,
    #[error("invalid email domain")]
    InvalidDomain,
}

#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    blocklist: Arc<PasswordBlocklist>,
}

impl PasswordPolicy {
    pub fn new(blocklist: Arc<PasswordBlocklist>) -> Self {
        Self { blocklist }
    }

    pub fn validate(&self, password: &SecretString) -> Result<(), PasswordValidationError> {
        if password.expose_secret().chars().count() < 15 {
            return Err(PasswordValidationError::TooShort);
        }

        if self.blocklist.contains(password.expose_secret()) {
            return Err(PasswordValidationError::Blocklisted);
        }

        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PasswordValidationError {
    #[error("password must contain at least 15 characters")]
    TooShort,
    #[error("password is not allowed")]
    Blocklisted,
}

#[derive(Debug, Clone)]
pub struct PasswordBlocklist {
    version: Arc<str>,
    hashes: Arc<HashSet<[u8; 20]>>,
}

impl PasswordBlocklist {
    pub fn from_file(path: &Path) -> Result<Self, PasswordBlocklistLoadError> {
        let file = File::open(path).map_err(|source| PasswordBlocklistLoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let mut version = None;
        let mut hashes = HashSet::new();
        let reader = BufReader::new(file);

        for (line_number, line) in reader.lines().enumerate() {
            let line_number = line_number + 1;

            let line = line.map_err(|source| PasswordBlocklistLoadError::Io {
                path: path.to_path_buf(),
                source,
            })?;

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(value) = line.strip_prefix("# version=") {
                if value.is_empty() || version.is_some() {
                    return Err(PasswordBlocklistLoadError::Invalid {
                        line: line_number,
                        reason: "exactly one non-empty version header is required",
                    });
                }

                version = Some(value.to_owned());
                continue;
            }

            if line.starts_with('#') {
                continue;
            }

            let hash_text = line.split_once(':').map_or(line, |(hash, _count)| hash);

            if hash_text.len() != 40 {
                return Err(PasswordBlocklistLoadError::Invalid {
                    line: line_number,
                    reason: "password hash must contain exactly 40 hexadecimal characters",
                });
            }

            let mut bytes = [0u8; 20];

            for (index, slot) in bytes.iter_mut().enumerate() {
                let start = index * 2;

                *slot = u8::from_str_radix(&hash_text[start..start + 2], 16).map_err(|_| {
                    PasswordBlocklistLoadError::Invalid {
                        line: line_number,
                        reason: "password hash must be hexadecimal",
                    }
                })?;
            }

            hashes.insert(bytes);
        }

        let version = version.ok_or(PasswordBlocklistLoadError::Invalid {
            line: 0,
            reason: "a # version=<value> header is required",
        })?;

        if hashes.is_empty() {
            return Err(PasswordBlocklistLoadError::Invalid {
                line: 0,
                reason: "password blocklist must contain at least one hash",
            });
        }

        Ok(Self {
            version: Arc::from(version),
            hashes: Arc::new(hashes),
        })
    }

    pub fn from_hashes<I, H>(version: impl Into<String>, hashes: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: Into<[u8; 20]>,
    {
        Self {
            version: Arc::from(version.into()),
            hashes: Arc::new(hashes.into_iter().map(Into::into).collect()),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn contains(&self, password: &str) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());

        let digest: [u8; 20] = hasher.finalize().into();

        self.hashes.contains(&digest)
    }
}

#[derive(Debug, Error)]
pub enum PasswordBlocklistLoadError {
    #[error("failed to read password blocklist {path}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("invalid password blocklist at line {line}: {reason}")]
    Invalid { line: usize, reason: &'static str },
}

pub fn hash_password(password: SecretString) -> Result<String, argon2::password_hash::Error> {
    use argon2::password_hash::PasswordHasher;

    Argon2::default()
        .hash_password(password.expose_secret().as_bytes())
        .map(|hash| hash.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(password: &str) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        hasher.finalize().into()
    }

    #[test]
    fn normalizes_email_without_provider_specific_changes() {
        let normalized = normalize_email("  Admin+Tag@BÜCHER.DE  ").unwrap();

        assert_eq!(normalized.canonical, "Admin+Tag@xn--bcher-kva.de");
        assert_eq!(normalized.normalized, "admin+tag@xn--bcher-kva.de");
    }

    #[test]
    fn casefolds_unicode_local_part_for_identity_comparison() {
        let first = normalize_email("Straße@example.com").unwrap();
        let second = normalize_email("STRASSE@example.com").unwrap();

        assert_eq!(first.normalized, second.normalized);
    }

    #[test]
    fn rejects_display_names_and_domain_literals() {
        assert!(normalize_email("Alice <alice@example.com>").is_err());
        assert!(normalize_email("alice@[127.0.0.1]").is_err());
    }

    #[test]
    fn password_policy_supports_64_characters() {
        let blocklist = PasswordBlocklist::from_hashes("test", Vec::<[u8; 20]>::new());

        let policy = PasswordPolicy::new(Arc::new(blocklist));
        let password = SecretString::from("a".repeat(64));

        assert!(policy.validate(&password).is_ok());
    }

    #[test]
    fn password_policy_blocks_exact_hash_matches() {
        let blocklist =
            PasswordBlocklist::from_hashes("test", [hash("correct horse battery staple")]);

        let policy = PasswordPolicy::new(Arc::new(blocklist));
        let password = SecretString::from("correct horse battery staple".to_owned());

        assert_eq!(
            policy.validate(&password),
            Err(PasswordValidationError::Blocklisted)
        );
    }

    #[test]
    fn hashes_password_with_argon2id() {
        use argon2::password_hash::{phc::PasswordHash, PasswordVerifier};

        let password = SecretString::from("a secure password with enough length".to_owned());

        let encoded = hash_password(password.clone()).unwrap();

        assert!(encoded.starts_with("$argon2id$v=19$"));

        Argon2::default()
            .verify_password(
                password.expose_secret().as_bytes(),
                &PasswordHash::new(&encoded).unwrap(),
            )
            .unwrap();
    }
}
