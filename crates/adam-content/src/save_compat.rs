use super::modding::ModId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SavedModReference {
    pub id: String,
    pub version: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SaveCompatibilityMetadata {
    pub simulation_version: u32,
    pub world_schema_version: u32,
    pub mods: Vec<SavedModReference>,
    pub package_fingerprint: u64,
    pub content_fingerprint: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompatibilityIssue {
    SimulationVersion { expected: u32, actual: u32 },
    WorldSchemaVersion { expected: u32, actual: u32 },
    ModList,
    PackageFingerprint,
    ContentFingerprint,
}
/// Compares save metadata with the exact currently loaded simulation and mod set.
#[must_use]
pub fn compatibility_issues(
    saved: &SaveCompatibilityMetadata,
    current: &SaveCompatibilityMetadata,
) -> Vec<CompatibilityIssue> {
    let mut issues = Vec::new();
    if saved.simulation_version != current.simulation_version {
        issues.push(CompatibilityIssue::SimulationVersion {
            expected: saved.simulation_version,
            actual: current.simulation_version,
        });
    }
    if saved.world_schema_version != current.world_schema_version {
        issues.push(CompatibilityIssue::WorldSchemaVersion {
            expected: saved.world_schema_version,
            actual: current.world_schema_version,
        });
    }
    if saved.mods != current.mods {
        issues.push(CompatibilityIssue::ModList);
    }
    if saved.package_fingerprint != current.package_fingerprint {
        issues.push(CompatibilityIssue::PackageFingerprint);
    }
    if saved.content_fingerprint != current.content_fingerprint {
        issues.push(CompatibilityIssue::ContentFingerprint);
    }
    issues
}
#[must_use]
pub fn saved_mod(id: &ModId, version: &str) -> SavedModReference {
    SavedModReference {
        id: id.as_str().to_owned(),
        version: version.to_owned(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn metadata() -> SaveCompatibilityMetadata {
        SaveCompatibilityMetadata {
            simulation_version: 6,
            world_schema_version: 4,
            mods: vec![SavedModReference {
                id: "core.base".into(),
                version: "1.0".into(),
            }],
            package_fingerprint: 1,
            content_fingerprint: 2,
        }
    }
    #[test]
    fn exact_metadata_is_compatible() {
        let value = metadata();
        assert!(compatibility_issues(&value, &value).is_empty());
    }
    #[test]
    fn changed_content_is_reported_separately() {
        let saved = metadata();
        let mut current = metadata();
        current.content_fingerprint = 3;
        assert_eq!(
            compatibility_issues(&saved, &current),
            vec![CompatibilityIssue::ContentFingerprint]
        );
    }
}
