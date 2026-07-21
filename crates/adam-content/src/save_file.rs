use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 8] = b"ADAMSAVE";
const CONTAINER_VERSION: u32 = 1;
const HEADER_LEN: usize = 28;
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveFileError {
    Io(String),
    InvalidMagic,
    UnsupportedContainer(u32),
    LengthMismatch,
    ChecksumMismatch { expected: u64, actual: u64 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveSource {
    Primary,
    Backup,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveredSave {
    pub payload: Vec<u8>,
    pub source: SaveSource,
    pub primary_error: Option<SaveFileError>,
}
impl fmt::Display for SaveFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "save I/O failed: {e}"),
            Self::InvalidMagic => f.write_str("invalid save magic"),
            Self::UnsupportedContainer(v) => write!(f, "unsupported save container {v}"),
            Self::LengthMismatch => f.write_str("save payload length mismatch"),
            Self::ChecksumMismatch { expected, actual } => write!(
                f,
                "save checksum mismatch: expected {expected:016x}, got {actual:016x}"
            ),
        }
    }
}
impl std::error::Error for SaveFileError {}
/// Wraps payload bytes in a checksummed container.
#[must_use]
pub fn build_save_container(payload: &[u8]) -> Vec<u8> {
    let checksum = checksum(payload);
    let mut bytes = Vec::with_capacity(HEADER_LEN + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&checksum.to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
/// Verifies and extracts a save payload.
/// # Errors
/// Returns [`SaveFileError`] for corrupt, truncated, or unsupported containers.
pub fn parse_save_container(bytes: &[u8]) -> Result<&[u8], SaveFileError> {
    if bytes.len() < HEADER_LEN {
        return Err(SaveFileError::LengthMismatch);
    }
    if &bytes[..8] != MAGIC {
        return Err(SaveFileError::InvalidMagic);
    }
    let version_bytes: [u8; 4] = bytes
        .get(8..12)
        .ok_or(SaveFileError::LengthMismatch)?
        .try_into()
        .map_err(|_| SaveFileError::LengthMismatch)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != CONTAINER_VERSION {
        return Err(SaveFileError::UnsupportedContainer(version));
    }
    let length_bytes: [u8; 8] = bytes
        .get(12..20)
        .ok_or(SaveFileError::LengthMismatch)?
        .try_into()
        .map_err(|_| SaveFileError::LengthMismatch)?;
    let length = u64::from_le_bytes(length_bytes);
    let checksum_bytes: [u8; 8] = bytes
        .get(20..28)
        .ok_or(SaveFileError::LengthMismatch)?
        .try_into()
        .map_err(|_| SaveFileError::LengthMismatch)?;
    let expected = u64::from_le_bytes(checksum_bytes);
    let payload = &bytes[HEADER_LEN..];
    if usize::try_from(length).ok() != Some(payload.len()) {
        return Err(SaveFileError::LengthMismatch);
    }
    let actual = checksum(payload);
    if actual != expected {
        return Err(SaveFileError::ChecksumMismatch { expected, actual });
    }
    Ok(payload)
}
/// Writes through a synced temporary file, preserving the previous save as `.bak`.
/// # Errors
/// Returns [`SaveFileError::Io`] and attempts to restore the backup on final rename failure.
pub fn write_save_atomic(path: &Path, payload: &[u8]) -> Result<(), SaveFileError> {
    let temporary = with_suffix(path, "tmp");
    let backup = with_suffix(path, "bak");
    let bytes = build_save_container(payload);
    let mut file = File::create(&temporary).map_err(|error| io(&error))?;
    file.write_all(&bytes).map_err(|error| io(&error))?;
    file.sync_all().map_err(|error| io(&error))?;
    if backup.exists() {
        fs::remove_file(&backup).map_err(|error| io(&error))?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|error| io(&error))?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(io(&error));
    }
    Ok(())
}
/// Reads and validates an atomic save container.
/// # Errors
/// Returns [`SaveFileError`] for I/O or container-integrity failures.
pub fn read_save_file(path: &Path) -> Result<Vec<u8>, SaveFileError> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|error| io(&error))?
        .read_to_end(&mut bytes)
        .map_err(|error| io(&error))?;
    Ok(parse_save_container(&bytes)?.to_vec())
}
/// Reads the primary save and falls back to the validated `.bak` file when necessary.
/// # Errors
/// Returns both primary and backup diagnostics when neither file is valid.
pub fn read_save_with_backup(path: &Path) -> Result<RecoveredSave, Vec<SaveFileError>> {
    match read_save_file(path) {
        Ok(payload) => Ok(RecoveredSave {
            payload,
            source: SaveSource::Primary,
            primary_error: None,
        }),
        Err(primary) => {
            let backup = with_suffix(path, "bak");
            match read_save_file(&backup) {
                Ok(payload) => Ok(RecoveredSave {
                    payload,
                    source: SaveSource::Backup,
                    primary_error: Some(primary),
                }),
                Err(backup_error) => Err(vec![primary, backup_error]),
            }
        }
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_owned();
    value.push(format!(".{suffix}"));
    PathBuf::from(value)
}
fn io(error: &std::io::Error) -> SaveFileError {
    SaveFileError::Io(error.to_string())
}
fn checksum(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corruption_is_detected() {
        let mut bytes = build_save_container(b"history");
        *bytes.last_mut().expect("byte") ^= 1;
        assert!(matches!(
            parse_save_container(&bytes),
            Err(SaveFileError::ChecksumMismatch { .. })
        ));
    }
    #[test]
    fn atomic_write_keeps_previous_backup() {
        let dir = std::env::temp_dir().join(format!("adam-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("world.adam");
        write_save_atomic(&path, b"first").expect("write");
        write_save_atomic(&path, b"second").expect("write");
        assert_eq!(read_save_file(&path).expect("read"), b"second");
        assert_eq!(
            read_save_file(&with_suffix(&path, "bak")).expect("backup"),
            b"first"
        );
        fs::remove_dir_all(dir).expect("cleanup");
    }
    #[test]
    fn corrupt_primary_recovers_from_valid_backup() {
        let dir = std::env::temp_dir().join(format!("adam-recovery-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("world.adam");
        write_save_atomic(&path, b"first").expect("first");
        write_save_atomic(&path, b"second").expect("second");
        fs::write(&path, b"corrupt").expect("corrupt");
        let recovered = read_save_with_backup(&path).expect("recover");
        assert_eq!(recovered.source, SaveSource::Backup);
        assert_eq!(recovered.payload, b"first");
        assert!(recovered.primary_error.is_some());
        fs::remove_dir_all(dir).expect("cleanup");
    }
}
