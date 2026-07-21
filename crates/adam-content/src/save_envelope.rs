use super::save_compat::{CompatibilityIssue, SaveCompatibilityMetadata, compatibility_issues};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;

pub const SAVE_FORMAT_VERSION: u32 = 1;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SaveEnvelope<T> {
    pub format_version: u32,
    pub compatibility: SaveCompatibilityMetadata,
    pub payload: T,
}
impl<T> SaveEnvelope<T> {
    #[must_use]
    pub const fn new(compatibility: SaveCompatibilityMetadata, payload: T) -> Self {
        Self {
            format_version: SAVE_FORMAT_VERSION,
            compatibility,
            payload,
        }
    }
}
/// Encodes a versioned save envelope as strict TOML.
/// # Errors
/// Returns [`SaveEnvelopeError::Encode`] when the payload cannot be represented.
pub fn encode_save<T: Serialize>(envelope: &SaveEnvelope<T>) -> Result<String, SaveEnvelopeError> {
    toml::to_string(envelope).map_err(|e| SaveEnvelopeError::Encode(e.to_string()))
}
/// Decodes a save and rejects format or mod-set incompatibility before exposing its payload.
/// # Errors
/// Returns [`SaveEnvelopeError`] for syntax, format-version, or compatibility mismatches.
pub fn decode_save<T: DeserializeOwned>(
    source: &str,
    current: &SaveCompatibilityMetadata,
) -> Result<SaveEnvelope<T>, SaveEnvelopeError> {
    let envelope: SaveEnvelope<T> =
        toml::from_str(source).map_err(|e| SaveEnvelopeError::Decode(e.to_string()))?;
    if envelope.format_version != SAVE_FORMAT_VERSION {
        return Err(SaveEnvelopeError::UnsupportedFormat {
            found: envelope.format_version,
            supported: SAVE_FORMAT_VERSION,
        });
    }
    let issues = compatibility_issues(&envelope.compatibility, current);
    if issues.is_empty() {
        Ok(envelope)
    } else {
        Err(SaveEnvelopeError::Incompatible(issues))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveEnvelopeError {
    Encode(String),
    Decode(String),
    UnsupportedFormat { found: u32, supported: u32 },
    Incompatible(Vec<CompatibilityIssue>),
}
impl fmt::Display for SaveEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(e) => write!(f, "save encode failed: {e}"),
            Self::Decode(e) => write!(f, "save decode failed: {e}"),
            Self::UnsupportedFormat { found, supported } => {
                write!(f, "unsupported save format {found}; supported {supported}")
            }
            Self::Incompatible(issues) => {
                write!(f, "save is incompatible for {} reason(s)", issues.len())
            }
        }
    }
}
impl std::error::Error for SaveEnvelopeError {}

pub trait SaveMigration {
    fn source_version(&self) -> u32;
    fn target_version(&self) -> u32;
    /// Migrates one save representation.
    /// # Errors
    /// Returns a diagnostic when this migration cannot transform the source.
    fn migrate(&self, source: &str) -> Result<String, String>;
}
/// Applies a contiguous migration chain until the current format is reached.
/// # Errors
/// Returns an error when no next migration exists, a step fails, or a cycle is detected.
pub fn migrate_save(
    mut source: String,
    mut version: u32,
    migrations: &[Box<dyn SaveMigration>],
) -> Result<String, String> {
    let mut steps = 0;
    while version < SAVE_FORMAT_VERSION {
        let migration = migrations
            .iter()
            .find(|m| m.source_version() == version)
            .ok_or_else(|| format!("missing save migration from version {version}"))?;
        if migration.target_version() <= version {
            return Err("save migration must advance version".into());
        }
        source = migration.migrate(&source)?;
        version = migration.target_version();
        steps += 1;
        if steps > migrations.len() {
            return Err("save migration cycle".into());
        }
    }
    if version == SAVE_FORMAT_VERSION {
        Ok(source)
    } else {
        Err(format!("save migration overshot to version {version}"))
    }
}

#[cfg(test)]
mod tests {
    use super::super::save_compat::SavedModReference;
    use super::*;
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Payload {
        year: i32,
    }
    fn metadata() -> SaveCompatibilityMetadata {
        SaveCompatibilityMetadata {
            simulation_version: 6,
            world_schema_version: 4,
            mods: vec![SavedModReference {
                id: "core.base".into(),
                version: "1".into(),
            }],
            package_fingerprint: 1,
            content_fingerprint: 2,
        }
    }
    #[test]
    fn envelope_round_trip_requires_exact_context() {
        let meta = metadata();
        let encoded =
            encode_save(&SaveEnvelope::new(meta.clone(), Payload { year: 2025 })).expect("encode");
        let decoded: SaveEnvelope<Payload> = decode_save(&encoded, &meta).expect("decode");
        assert_eq!(decoded.payload.year, 2025);
    }
    #[test]
    fn changed_mod_content_is_rejected_before_payload() {
        let saved = metadata();
        let encoded =
            encode_save(&SaveEnvelope::new(saved.clone(), Payload { year: 2025 })).expect("encode");
        let mut current = saved;
        current.content_fingerprint = 99;
        assert!(matches!(
            decode_save::<Payload>(&encoded, &current),
            Err(SaveEnvelopeError::Incompatible(_))
        ));
    }
}
