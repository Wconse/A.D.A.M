use super::save_compat::{SaveCompatibilityMetadata, compatibility_issues};
use super::save_envelope::{SAVE_FORMAT_VERSION, SaveEnvelope, SaveEnvelopeError};
use adam_core::{World, WorldCommand, WorldError, replay_commands};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReplayCheckpoint {
    pub snapshot: World,
    pub command_tail: Vec<WorldCommand>,
}
impl ReplayCheckpoint {
    #[must_use]
    pub const fn new(snapshot: World, command_tail: Vec<WorldCommand>) -> Self {
        Self {
            snapshot,
            command_tail,
        }
    }
    /// Restores the snapshot and replays every recorded command.
    /// # Errors
    /// Returns [`WorldError`] when a replayed transition fails.
    pub fn restore(&self) -> Result<World, WorldError> {
        let mut world = self.snapshot.clone();
        replay_commands(&mut world, &self.command_tail)?;
        Ok(world)
    }
}
/// Encodes a snapshot plus ordered command tail in a compatibility envelope.
/// # Errors
/// Returns [`SaveEnvelopeError::Encode`] on serialization failure.
pub fn encode_replay_checkpoint(
    value: &ReplayCheckpoint,
    compatibility: SaveCompatibilityMetadata,
) -> Result<Vec<u8>, SaveEnvelopeError> {
    bincode::serialize(&SaveEnvelope::new(compatibility, value.clone()))
        .map_err(|e| SaveEnvelopeError::Encode(e.to_string()))
}
/// Decodes only when format and exact mod context match.
/// # Errors
/// Returns [`SaveEnvelopeError`] for corrupt, unsupported, or incompatible data.
pub fn decode_replay_checkpoint(
    bytes: &[u8],
    current: &SaveCompatibilityMetadata,
) -> Result<ReplayCheckpoint, SaveEnvelopeError> {
    let envelope: SaveEnvelope<ReplayCheckpoint> =
        bincode::deserialize(bytes).map_err(|e| SaveEnvelopeError::Decode(e.to_string()))?;
    if envelope.format_version != SAVE_FORMAT_VERSION {
        return Err(SaveEnvelopeError::UnsupportedFormat {
            found: envelope.format_version,
            supported: SAVE_FORMAT_VERSION,
        });
    }
    let issues = compatibility_issues(&envelope.compatibility, current);
    if issues.is_empty() {
        Ok(envelope.payload)
    } else {
        Err(SaveEnvelopeError::Incompatible(issues))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_compat::SaveCompatibilityMetadata;
    use adam_core::{Country, CountryId, SimDate, WorldSeed};
    fn metadata() -> SaveCompatibilityMetadata {
        SaveCompatibilityMetadata {
            simulation_version: adam_core::SIMULATION_VERSION,
            world_schema_version: crate::WORLD_SCHEMA_VERSION,
            mods: vec![],
            package_fingerprint: 1,
            content_fingerprint: 2,
        }
    }
    #[test]
    fn snapshot_plus_tail_matches_uninterrupted_history() {
        let mut snapshot = World::new(WorldSeed::new(47), SimDate::new(2025, 1).expect("date"));
        snapshot
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register");
        snapshot.advance_years(20).expect("years");
        let checkpoint =
            ReplayCheckpoint::new(snapshot.clone(), vec![WorldCommand::AdvanceYears(30)]);
        let encoded = encode_replay_checkpoint(&checkpoint, metadata()).expect("encode");
        let decoded = decode_replay_checkpoint(&encoded, &metadata()).expect("decode");
        let restored = decoded.restore().expect("restore");
        snapshot.advance_years(30).expect("years");
        assert_eq!(restored, snapshot);
        assert_eq!(restored.stable_fingerprint(), snapshot.stable_fingerprint());
    }
}
