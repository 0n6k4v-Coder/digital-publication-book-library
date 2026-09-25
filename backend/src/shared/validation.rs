use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

use addr_spec::AddrSpec;
use argon2::Argon2;
use idna::domain_to_ascii_strict;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use thiserror::Error;
use time::OffsetDateTime;
use tokio::{
    fs as async_fs,
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::Semaphore,
    task::JoinSet,
};
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

const SNAPSHOT_FORMAT_VERSION: u32 = 1;
const HIBP_SOURCE_IDENTIFIER: &str = "pwned-passwords-range-v3";
const TOTAL_HIBP_PREFIXES: u32 = 1_048_576;
const HIBP_PREFIXES_PER_SHARD: usize = 4_096;
const HIBP_SHARD_COUNT: usize = 256;
const HIBP_SUFFIX_HEX_LEN: usize = 35;
const HIBP_FULL_HASH_HEX_LEN: usize = 40;
const SHARD_MAGIC: [u8; 8] = *b"DPBLSH01";
const SHARD_HEADER_LEN: u64 = 16;
const SHARD_INDEX_ENTRIES: usize = HIBP_PREFIXES_PER_SHARD + 1;
const SHARD_INDEX_LEN: u64 = SHARD_INDEX_ENTRIES as u64 * 8;
const SHARD_DATA_OFFSET: u64 = SHARD_HEADER_LEN + SHARD_INDEX_LEN;
const SHARD_RECORD_LEN: u64 = HIBP_SUFFIX_HEX_LEN as u64;
const DEFAULT_HIBP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const HIBP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub const PASSWORD_BLOCKLIST_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

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

#[derive(Clone)]
pub struct PasswordBlocklist {
    state: Arc<RwLock<BlocklistState>>,
}

impl std::fmt::Debug for PasswordBlocklist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordBlocklist")
            .field("version", &self.version())
            .finish_non_exhaustive()
    }
}

enum BlocklistState {
    Memory {
        version: Arc<str>,
        hashes: Arc<HashSet<[u8; 20]>>,
    },
    Snapshot {
        root: PathBuf,
        manifest: SnapshotManifest,
    },
}

impl PasswordBlocklist {
    pub fn from_file(path: &Path) -> Result<Self, PasswordBlocklistLoadError> {
        Self::load_active(path)
    }

    pub fn load_active(root: &Path) -> Result<Self, PasswordBlocklistLoadError> {
        let state = load_active_state(root)?;

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
        })
    }

    pub fn from_hashes<I, H>(version: impl Into<String>, hashes: I) -> Self
    where
        I: IntoIterator<Item = H>,
        H: Into<[u8; 20]>,
    {
        Self {
            state: Arc::new(RwLock::new(BlocklistState::Memory {
                version: Arc::from(version.into()),
                hashes: Arc::new(hashes.into_iter().map(Into::into).collect()),
            })),
        }
    }

    pub fn version(&self) -> String {
        let state = self
            .state
            .read()
            .expect("password blocklist state lock poisoned");

        match &*state {
            BlocklistState::Memory { version, .. } => version.to_string(),
            BlocklistState::Snapshot { manifest, .. } => manifest.snapshot_version.clone(),
        }
    }

    pub fn contains(&self, password: &str) -> bool {
        let state = match self.state.read() {
            Ok(state) => state,
            Err(_) => return true,
        };

        match &*state {
            BlocklistState::Memory { hashes, .. } => {
                let digest = Sha1::digest(password.as_bytes());
                let digest: [u8; 20] = digest.into();
                hashes.contains(&digest)
            }
            BlocklistState::Snapshot { root, manifest } => {
                snapshot_contains(root, manifest, password).unwrap_or(true)
            }
        }
    }

    pub fn refresh_due(&self, max_age: Duration) -> bool {
        let state = match self.state.read() {
            Ok(state) => state,
            Err(_) => return true,
        };

        let BlocklistState::Snapshot { manifest, .. } = &*state else {
            return false;
        };

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let age = now.saturating_sub(manifest.source_retrieved_at_unix).max(0) as u64;

        age >= max_age.as_secs()
    }

    pub fn refresh_delay(&self, max_age: Duration) -> Duration {
        let state = match self.state.read() {
            Ok(state) => state,
            Err(_) => return Duration::ZERO,
        };

        let BlocklistState::Snapshot { manifest, .. } = &*state else {
            return max_age;
        };

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let age = now.saturating_sub(manifest.source_retrieved_at_unix).max(0) as u64;

        max_age.saturating_sub(Duration::from_secs(age))
    }

    pub fn reload_active(&self) -> Result<(), PasswordBlocklistLoadError> {
        let root = {
            let state = self
                .state
                .read()
                .expect("password blocklist state lock poisoned");

            match &*state {
                BlocklistState::Memory { .. } => {
                    return Err(PasswordBlocklistLoadError::NotReloadable)
                }
                BlocklistState::Snapshot { root, .. } => root.clone(),
            }
        };

        let new_state = load_active_state(&root)?;

        let mut state = self
            .state
            .write()
            .expect("password blocklist state lock poisoned");

        *state = new_state;

        Ok(())
    }

    pub async fn refresh_to_active(
        root: &Path,
        project_blocklist_path: &Path,
        refresh_concurrency: std::num::NonZeroUsize,
    ) -> Result<String, PasswordBlocklistRefreshError> {
        let project_blocklist = load_project_blocklist(project_blocklist_path)
            .map_err(PasswordBlocklistRefreshError::ProjectBlocklist)?;

        let snapshot_version = format!(
            "{}-{}",
            OffsetDateTime::now_utc().unix_timestamp(),
            Uuid::new_v4()
        );

        validate_snapshot_version(&snapshot_version)
            .map_err(|_| PasswordBlocklistRefreshError::InvalidVersion)?;

        let snapshot_root = root.join("snapshots");

        async_fs::create_dir_all(&snapshot_root)
            .await
            .map_err(|_| PasswordBlocklistRefreshError::Storage)?;

        let candidate_dir = snapshot_root.join(format!(".candidate-{snapshot_version}"));

        async_fs::create_dir_all(&candidate_dir)
            .await
            .map_err(|_| PasswordBlocklistRefreshError::Storage)?;

        let client = reqwest::Client::builder()
            .https_only(true)
            .user_agent("digital-publication-backend/0.1")
            .connect_timeout(HIBP_CONNECT_TIMEOUT)
            .timeout(DEFAULT_HIBP_REQUEST_TIMEOUT)
            .build()
            .map_err(|_| PasswordBlocklistRefreshError::HttpClient)?;

        let retrieval_time = OffsetDateTime::now_utc().unix_timestamp();

        let semaphore = Arc::new(Semaphore::new(refresh_concurrency.get()));

        let mut tasks = JoinSet::new();

        for shard_id in 0..HIBP_SHARD_COUNT {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .expect("semaphore cannot close");

            let client = client.clone();
            let project_blocklist = project_blocklist.clone();
            let candidate_dir = candidate_dir.clone();

            tasks.spawn(async move {
                let result = build_shard_file(
                    &client,
                    &candidate_dir,
                    shard_id,
                    &project_blocklist,
                    retrieval_time,
                )
                .await;

                drop(permit);
                result
            });
        }

        let mut refresh_failed = false;

        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(_)) | Err(_) => {
                    refresh_failed = true;
                    tasks.abort_all();
                    break;
                }
            }
        }

        if refresh_failed {
            let _ = async_fs::remove_dir_all(&candidate_dir).await;
            return Err(PasswordBlocklistRefreshError::SourceRefresh);
        }

        let manifest = SnapshotManifest {
            format_version: SNAPSHOT_FORMAT_VERSION,
            snapshot_version: snapshot_version.clone(),
            hibp_source: HIBP_SOURCE_IDENTIFIER.to_owned(),
            source_retrieved_at_unix: retrieval_time,
            generated_at_unix: OffsetDateTime::now_utc().unix_timestamp(),
            project_blocklist_version: project_blocklist.version,
            snapshot_sha256: compute_snapshot_digest_async(&candidate_dir)
                .await
                .map_err(|_| PasswordBlocklistRefreshError::Storage)?,
            hibp_prefixes_refreshed: TOTAL_HIBP_PREFIXES,
        };

        write_manifest_async(&candidate_dir, &manifest)
            .await
            .map_err(|_| PasswordBlocklistRefreshError::Storage)?;

        validate_snapshot_dir(&candidate_dir, &manifest.snapshot_version)
            .map_err(|_| PasswordBlocklistRefreshError::InvalidSnapshot)?;

        let final_dir = snapshot_root.join(&snapshot_version);

        async_fs::rename(&candidate_dir, &final_dir)
            .await
            .map_err(|_| PasswordBlocklistRefreshError::Storage)?;

        activate_snapshot_pointer(root, &snapshot_version).await?;

        Ok(snapshot_version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotManifest {
    format_version: u32,
    snapshot_version: String,
    hibp_source: String,
    source_retrieved_at_unix: i64,
    generated_at_unix: i64,
    project_blocklist_version: String,
    snapshot_sha256: String,
    hibp_prefixes_refreshed: u32,
}

#[derive(Debug, Clone)]
struct ProjectBlocklist {
    version: String,
    suffixes: Arc<Vec<Vec<[u8; HIBP_SUFFIX_HEX_LEN]>>>,
}

#[derive(Debug, Error)]
pub enum PasswordBlocklistLoadError {
    #[error("password blocklist active snapshot is missing")]
    MissingActiveSnapshot,

    #[error("password blocklist snapshot is not reloadable")]
    NotReloadable,

    #[error("password blocklist snapshot is invalid")]
    InvalidSnapshot,

    #[error("password blocklist snapshot could not be read")]
    Io(#[source] std::io::Error),

    #[error("password blocklist snapshot manifest is invalid")]
    Manifest(#[source] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum PasswordProjectBlocklistLoadError {
    #[error("failed to read project password blocklist")]
    Io(#[source] std::io::Error),

    #[error("invalid project password blocklist")]
    Invalid { line: usize },
}

#[derive(Debug, Error)]
pub enum PasswordBlocklistRefreshError {
    #[error("failed to load project password blocklist")]
    ProjectBlocklist(#[source] PasswordProjectBlocklistLoadError),

    #[error("password blocklist snapshot version is invalid")]
    InvalidVersion,

    #[error("failed to create password blocklist storage")]
    Storage,

    #[error("failed to create HTTPS client for password blocklist refresh")]
    HttpClient,

    #[error("password blocklist source refresh failed")]
    SourceRefresh,

    #[error("generated password blocklist snapshot is invalid")]
    InvalidSnapshot,

    #[error("failed to activate password blocklist snapshot")]
    Activation(#[source] std::io::Error),
}

fn load_active_state(root: &Path) -> Result<BlocklistState, PasswordBlocklistLoadError> {
    let active_path = root.join("active");

    let snapshot_version = std::fs::read_to_string(&active_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PasswordBlocklistLoadError::MissingActiveSnapshot
        } else {
            PasswordBlocklistLoadError::Io(error)
        }
    })?;

    let snapshot_version = snapshot_version.trim();

    validate_snapshot_version(snapshot_version)
        .map_err(|_| PasswordBlocklistLoadError::InvalidSnapshot)?;

    let snapshot_dir = root.join("snapshots").join(snapshot_version);

    validate_snapshot_dir(&snapshot_dir, snapshot_version)
        .map_err(|_| PasswordBlocklistLoadError::InvalidSnapshot)?;

    let manifest = read_manifest(&snapshot_dir)?;

    Ok(BlocklistState::Snapshot {
        root: root.to_path_buf(),
        manifest,
    })
}

fn read_manifest(snapshot_dir: &Path) -> Result<SnapshotManifest, PasswordBlocklistLoadError> {
    let manifest_path = snapshot_dir.join("manifest.json");

    let manifest =
        std::fs::read_to_string(manifest_path).map_err(PasswordBlocklistLoadError::Io)?;

    serde_json::from_str(&manifest).map_err(PasswordBlocklistLoadError::Manifest)
}

fn validate_snapshot_dir(snapshot_dir: &Path, expected_version: &str) -> Result<(), ()> {
    if !snapshot_dir.is_dir() || validate_snapshot_version(expected_version).is_err() {
        return Err(());
    }

    let manifest_path = snapshot_dir.join("manifest.json");

    let manifest_text = std::fs::read_to_string(manifest_path).map_err(|_| ())?;

    let manifest: SnapshotManifest = serde_json::from_str(&manifest_text).map_err(|_| ())?;

    if manifest.format_version != SNAPSHOT_FORMAT_VERSION
        || manifest.snapshot_version != expected_version
        || manifest.hibp_source != HIBP_SOURCE_IDENTIFIER
        || manifest.hibp_prefixes_refreshed != TOTAL_HIBP_PREFIXES
        || manifest.source_retrieved_at_unix <= 0
        || manifest.generated_at_unix < manifest.source_retrieved_at_unix
        || manifest.project_blocklist_version.is_empty()
        || manifest.snapshot_sha256.len() != 64
        || !manifest
            .snapshot_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(());
    }

    for shard_id in 0..HIBP_SHARD_COUNT {
        let path = shard_path(snapshot_dir, shard_id);
        let mut file = File::open(path).map_err(|_| ())?;

        validate_shard_file(&mut file, shard_id).map_err(|_| ())?;
    }

    let digest = compute_snapshot_digest(snapshot_dir).map_err(|_| ())?;

    if !digest.eq_ignore_ascii_case(&manifest.snapshot_sha256) {
        return Err(());
    }

    Ok(())
}

fn validate_shard_file(file: &mut File, shard_id: usize) -> Result<(), std::io::Error> {
    let mut header = [0u8; SHARD_HEADER_LEN as usize];
    file.read_exact(&mut header)?;

    if header[..8] != SHARD_MAGIC
        || u32::from_le_bytes(header[8..12].try_into().expect("header slice"))
            != SNAPSHOT_FORMAT_VERSION
        || u16::from_le_bytes(header[12..14].try_into().expect("header slice")) as usize != shard_id
        || u16::from_le_bytes(header[14..16].try_into().expect("header slice")) as usize
            != HIBP_PREFIXES_PER_SHARD
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid password blocklist shard header",
        ));
    }

    let mut offsets = vec![0u64; SHARD_INDEX_ENTRIES];
    let mut bytes = [0u8; 8];

    for offset in &mut offsets {
        file.read_exact(&mut bytes)?;
        *offset = u64::from_le_bytes(bytes);
    }

    let file_len = file.metadata()?.len();

    if offsets.first().copied() != Some(SHARD_DATA_OFFSET)
        || offsets.last().copied() != Some(file_len)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid password blocklist shard offsets",
        ));
    }

    for pair in offsets.windows(2) {
        let start = pair[0];
        let end = pair[1];

        if end < start || (end - start) % SHARD_RECORD_LEN != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid password blocklist shard record range",
            ));
        }
    }

    Ok(())
}

fn snapshot_contains(
    root: &Path,
    manifest: &SnapshotManifest,
    password: &str,
) -> Result<bool, std::io::Error> {
    let digest = Sha1::digest(password.as_bytes());
    let digest: [u8; 20] = digest.into();

    let full_hash = encode_upper_hex(&digest);

    let prefix =
        parse_prefix(&full_hash[..5]).expect("encoded SHA-1 always has hexadecimal prefix");

    let shard_id = (prefix / HIBP_PREFIXES_PER_SHARD as u32) as usize;

    let local_prefix = (prefix % HIBP_PREFIXES_PER_SHARD as u32) as usize;

    let suffix = &full_hash[5..];

    debug_assert_eq!(suffix.len(), HIBP_SUFFIX_HEX_LEN);

    let snapshot_dir = root.join("snapshots").join(&manifest.snapshot_version);

    let path = shard_path(&snapshot_dir, shard_id);
    let mut file = File::open(path)?;

    let mut header = [0u8; SHARD_HEADER_LEN as usize];
    file.read_exact(&mut header)?;

    if header[..8] != SHARD_MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid password blocklist shard",
        ));
    }

    let index_offset = SHARD_HEADER_LEN + local_prefix as u64 * 8;

    file.seek(SeekFrom::Start(index_offset))?;

    let mut range = [0u8; 16];
    file.read_exact(&mut range)?;

    let start = u64::from_le_bytes(range[..8].try_into().expect("range slice"));

    let end = u64::from_le_bytes(range[8..].try_into().expect("range slice"));

    let file_len = file.metadata()?.len();

    if start < SHARD_DATA_OFFSET
        || end < start
        || end > file_len
        || (end - start) % SHARD_RECORD_LEN != 0
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid password blocklist lookup range",
        ));
    }

    let record_count = (end - start) / SHARD_RECORD_LEN;

    let mut low = 0u64;
    let mut high = record_count;

    let mut record = [0u8; HIBP_SUFFIX_HEX_LEN];

    while low < high {
        let mid = low + (high - low) / 2;

        file.seek(SeekFrom::Start(start + mid * SHARD_RECORD_LEN))?;

        file.read_exact(&mut record)?;

        match record.as_slice().cmp(suffix) {
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => high = mid,
            std::cmp::Ordering::Equal => return Ok(true),
        }
    }

    Ok(false)
}

fn parse_prefix(prefix: &[u8]) -> Option<u32> {
    if prefix.len() != 5 {
        return None;
    }

    let mut value = 0u32;

    for byte in prefix {
        let nibble = parse_hex_nibble(*byte)? as u32;
        value = (value << 4) | nibble;
    }

    Some(value)
}

fn parse_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn parse_project_hash(line: &str) -> Option<[u8; 20]> {
    if line.len() != HIBP_FULL_HASH_HEX_LEN {
        return None;
    }

    let bytes = line.as_bytes();
    let mut digest = [0u8; 20];

    for (index, slot) in digest.iter_mut().enumerate() {
        let high = parse_hex_nibble(bytes[index * 2])?;
        let low = parse_hex_nibble(bytes[index * 2 + 1])?;

        *slot = (high << 4) | low;
    }

    Some(digest)
}

fn load_project_blocklist(
    path: &Path,
) -> Result<ProjectBlocklist, PasswordProjectBlocklistLoadError> {
    let file = File::open(path).map_err(PasswordProjectBlocklistLoadError::Io)?;

    let reader = BufReader::new(file);

    let mut version = None;

    let mut suffixes = vec![Vec::<[u8; HIBP_SUFFIX_HEX_LEN]>::new(); TOTAL_HIBP_PREFIXES as usize];

    for (line_number, line) in reader.lines().enumerate() {
        let line_number = line_number + 1;

        let line = line.map_err(PasswordProjectBlocklistLoadError::Io)?;

        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(value) = line.strip_prefix("# version=") {
            if value.is_empty() || version.is_some() {
                return Err(PasswordProjectBlocklistLoadError::Invalid { line: line_number });
            }

            version = Some(value.to_owned());
            continue;
        }

        if line.starts_with('#') {
            continue;
        }

        let digest = parse_project_hash(line)
            .ok_or(PasswordProjectBlocklistLoadError::Invalid { line: line_number })?;

        let hex = encode_upper_hex(&digest);

        let prefix = parse_prefix(&hex[..5]).expect("SHA-1 hex prefix is valid");

        let suffix = hex[5..]
            .try_into()
            .expect("SHA-1 suffix has exactly 35 hexadecimal bytes");

        suffixes[prefix as usize].push(suffix);
    }

    let version = version.ok_or(PasswordProjectBlocklistLoadError::Invalid { line: 0 })?;

    for group in &mut suffixes {
        group.sort_unstable();
        group.dedup();
    }

    Ok(ProjectBlocklist {
        version,
        suffixes: Arc::new(suffixes),
    })
}

fn validate_snapshot_version(version: &str) -> Result<(), ()> {
    if version.is_empty() || version.len() > 200 {
        return Err(());
    }

    if !version
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(());
    }

    Ok(())
}

fn shard_path(snapshot_dir: &Path, shard_id: usize) -> PathBuf {
    snapshot_dir.join(format!("shard-{shard_id:02X}.bin"))
}

async fn build_shard_file(
    client: &reqwest::Client,
    candidate_dir: &Path,
    shard_id: usize,
    project_blocklist: &ProjectBlocklist,
    _retrieval_time: i64,
) -> Result<(), ()> {
    let path = shard_path(candidate_dir, shard_id);

    let mut file = async_fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .await
        .map_err(|_| ())?;

    file.write_all(&SHARD_MAGIC).await.map_err(|_| ())?;

    file.write_all(&SNAPSHOT_FORMAT_VERSION.to_le_bytes())
        .await
        .map_err(|_| ())?;

    file.write_all(&(shard_id as u16).to_le_bytes())
        .await
        .map_err(|_| ())?;

    file.write_all(&(HIBP_PREFIXES_PER_SHARD as u16).to_le_bytes())
        .await
        .map_err(|_| ())?;

    let empty_offsets = vec![0u8; SHARD_INDEX_LEN as usize];

    file.write_all(&empty_offsets).await.map_err(|_| ())?;

    let mut offsets = vec![SHARD_DATA_OFFSET; SHARD_INDEX_ENTRIES];
    let mut data_cursor = SHARD_DATA_OFFSET;

    let prefix_start = shard_id * HIBP_PREFIXES_PER_SHARD;

    for (local_prefix, offset) in offsets.iter_mut().enumerate().take(HIBP_PREFIXES_PER_SHARD) {
        let prefix_number = prefix_start + local_prefix;
        let prefix = format!("{prefix_number:05X}");

        let hibp_suffixes = fetch_hibp_range(client, &prefix).await?;

        let mut suffixes = hibp_suffixes;

        suffixes.extend(project_blocklist.suffixes[prefix_number].iter().copied());

        suffixes.sort_unstable();
        suffixes.dedup();

        *offset = data_cursor;

        for suffix in suffixes {
            file.write_all(&suffix).await.map_err(|_| ())?;

            data_cursor += SHARD_RECORD_LEN;
        }
    }

    offsets[HIBP_PREFIXES_PER_SHARD] = data_cursor;

    file.seek(SeekFrom::Start(SHARD_HEADER_LEN))
        .await
        .map_err(|_| ())?;

    for offset in offsets {
        file.write_all(&offset.to_le_bytes())
            .await
            .map_err(|_| ())?;
    }

    file.sync_all().await.map_err(|_| ())?;

    Ok(())
}

async fn fetch_hibp_range(
    client: &reqwest::Client,
    prefix: &str,
) -> Result<Vec<[u8; HIBP_SUFFIX_HEX_LEN]>, ()> {
    let url = format!("https://api.pwnedpasswords.com/range/{prefix}");

    let response = client.get(url).send().await.map_err(|_| ())?;

    if !response.status().is_success() {
        return Err(());
    }

    if response.status() != reqwest::StatusCode::OK {
        return Err(());
    }

    let body = response.text().await.map_err(|_| ())?;

    let mut suffixes = Vec::new();

    for raw_line in body.lines() {
        let line = raw_line.trim_end_matches('\r');

        if line.is_empty() {
            continue;
        }

        let (suffix_text, count_text) = line.split_once(':').ok_or(())?;

        if suffix_text.len() != HIBP_SUFFIX_HEX_LEN || count_text.is_empty() {
            return Err(());
        }

        if count_text.parse::<u64>().is_err() {
            return Err(());
        }

        let suffix_bytes = suffix_text.as_bytes();

        let mut suffix = [b'0'; HIBP_SUFFIX_HEX_LEN];

        for (index, slot) in suffix.iter_mut().enumerate() {
            let nibble = parse_hex_nibble(suffix_bytes[index]).ok_or(())?;

            *slot = match nibble {
                0..=9 => b'0' + nibble,
                10..=15 => b'A' + (nibble - 10),
                _ => return Err(()),
            };
        }

        suffixes.push(suffix);
    }

    suffixes.sort_unstable();
    suffixes.dedup();

    Ok(suffixes)
}

async fn compute_snapshot_digest_async(snapshot_dir: &Path) -> Result<String, std::io::Error> {
    let snapshot_dir = snapshot_dir.to_path_buf();

    tokio::task::spawn_blocking(move || compute_snapshot_digest(&snapshot_dir))
        .await
        .map_err(std::io::Error::other)?
}

fn compute_snapshot_digest(snapshot_dir: &Path) -> Result<String, std::io::Error> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    for shard_id in 0..HIBP_SHARD_COUNT {
        let mut file = File::open(shard_path(snapshot_dir, shard_id))?;

        loop {
            let read = file.read(&mut buffer)?;

            if read == 0 {
                break;
            }

            hasher.update(&buffer[..read]);
        }
    }

    let digest: [u8; 32] = hasher.finalize().into();

    Ok(encode_lower_hex(&digest))
}

async fn write_manifest_async(
    snapshot_dir: &Path,
    manifest: &SnapshotManifest,
) -> Result<(), std::io::Error> {
    let temp_path = snapshot_dir.join("manifest.json.tmp");
    let final_path = snapshot_dir.join("manifest.json");

    let bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;

    let mut file = async_fs::File::create(&temp_path).await?;

    file.write_all(&bytes).await?;
    file.sync_all().await?;
    drop(file);

    async_fs::rename(temp_path, final_path).await?;

    Ok(())
}

async fn activate_snapshot_pointer(
    root: &Path,
    snapshot_version: &str,
) -> Result<(), PasswordBlocklistRefreshError> {
    async_fs::create_dir_all(root.join("snapshots"))
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)?;

    let pointer_tmp = root.join(format!("active.tmp-{snapshot_version}"));

    let pointer_path = root.join("active");

    let mut file = async_fs::File::create(&pointer_tmp)
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)?;

    file.write_all(snapshot_version.as_bytes())
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)?;

    file.write_all(b"\n")
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)?;

    file.sync_all()
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)?;

    drop(file);

    async_fs::rename(pointer_tmp, pointer_path)
        .await
        .map_err(PasswordBlocklistRefreshError::Activation)
}

fn encode_upper_hex(bytes: &[u8; 20]) -> [u8; 40] {
    let mut output = [0u8; 40];

    for (index, byte) in bytes.iter().copied().enumerate() {
        output[index * 2] = hex_digit(byte >> 4, true);
        output[index * 2 + 1] = hex_digit(byte & 0x0f, true);
    }

    output
}

fn encode_lower_hex(bytes: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);

    for byte in bytes.iter().copied() {
        output.push(hex_digit(byte >> 4, false) as char);
        output.push(hex_digit(byte & 0x0f, false) as char);
    }

    output
}

fn hex_digit(value: u8, uppercase: bool) -> u8 {
    match value {
        0..=9 => b'0' + value,
        10..=15 if uppercase => b'A' + (value - 10),
        10..=15 => b'a' + (value - 10),
        _ => unreachable!("nibble is always less than 16"),
    }
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
    fn password_policy_does_not_block_partial_or_substring_matches() {
        let blocklist =
            PasswordBlocklist::from_hashes("test", [hash("correct horse battery staple")]);

        let policy = PasswordPolicy::new(Arc::new(blocklist));

        let prefix = SecretString::from("correct horse battery staple plus context".to_owned());

        let suffix = SecretString::from("context correct horse battery staple".to_owned());

        assert!(policy.validate(&prefix).is_ok());
        assert!(policy.validate(&suffix).is_ok());
    }

    #[test]
    fn password_blocklist_fails_closed_when_no_validated_snapshot_exists() {
        let path = std::env::temp_dir().join(format!("dpbl-missing-{}", Uuid::new_v4()));

        let result = PasswordBlocklist::load_active(&path);

        assert!(matches!(
            result,
            Err(PasswordBlocklistLoadError::MissingActiveSnapshot)
        ));
    }

    #[test]
    fn hibp_prefix_space_is_complete() {
        let first = 0u32;
        let last = TOTAL_HIBP_PREFIXES - 1;

        assert_eq!(first, 0x00000);
        assert_eq!(last, 0xFFFFF);
        assert_eq!(last - first + 1, TOTAL_HIBP_PREFIXES);
    }

    #[test]
    fn password_blocklist_debug_does_not_include_a_password() {
        let password = "a secure password with enough length";

        let blocklist = PasswordBlocklist::from_hashes("test", [hash(password)]);

        let debug = format!("{blocklist:?}");

        assert!(!debug.contains(password));
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

    #[test]
    fn uppercase_hash_encoding_preserves_complete_sha1() {
        let digest = hash("password-password");
        let encoded = encode_upper_hex(&digest);

        assert_eq!(encoded.len(), 40);

        assert!(encoded
            .iter()
            .all(|byte| { byte.is_ascii_hexdigit() && !byte.is_ascii_lowercase() }));
    }
}
