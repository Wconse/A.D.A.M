use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Deserialize;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ModId(String);
impl ModId {
    /// Parses a stable lowercase dotted mod identity such as `author.project`.
    /// # Errors
    /// Returns [`ModError::InvalidIdentifier`] for malformed identities.
    pub fn parse(value: impl Into<String>) -> Result<Self, ModError> {
        let value = value.into();
        if valid_id(&value, true) {
            Ok(Self(value))
        } else {
            Err(ModError::InvalidIdentifier(value))
        }
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NamespacedKey {
    namespace: ModId,
    local: String,
}
impl NamespacedKey {
    /// Parses `namespace:local_name`.
    /// # Errors
    /// Returns [`ModError::InvalidIdentifier`] when either side is invalid.
    pub fn parse(value: &str) -> Result<Self, ModError> {
        let (namespace, local) = value
            .split_once(':')
            .ok_or_else(|| ModError::InvalidIdentifier(value.to_owned()))?;
        if !valid_id(local, false) {
            return Err(ModError::InvalidIdentifier(value.to_owned()));
        }
        Ok(Self {
            namespace: ModId::parse(namespace)?,
            local: local.to_owned(),
        })
    }
    #[must_use]
    pub const fn namespace(&self) -> &ModId {
        &self.namespace
    }
    #[must_use]
    pub fn local(&self) -> &str {
        &self.local
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModManifest {
    id: ModId,
    name: String,
    version: String,
    game_version: String,
    dependencies: Vec<ModId>,
    load_after: Vec<ModId>,
}
impl ModManifest {
    /// Parses and validates a strict TOML manifest.
    /// # Errors
    /// Returns [`ModError`] for malformed TOML, identifiers, empty metadata, or self-dependencies.
    pub fn parse_toml(source: &str) -> Result<Self, ModError> {
        let raw: RawManifest = toml::from_str(source).map_err(ModError::Toml)?;
        let id = ModId::parse(raw.id)?;
        if raw.name.trim().is_empty()
            || raw.version.trim().is_empty()
            || raw.game_version.trim().is_empty()
        {
            return Err(ModError::EmptyMetadata);
        }
        let dependencies = raw
            .dependencies
            .into_iter()
            .map(ModId::parse)
            .collect::<Result<Vec<_>, _>>()?;
        let load_after = raw
            .load_after
            .into_iter()
            .map(ModId::parse)
            .collect::<Result<Vec<_>, _>>()?;
        if dependencies
            .iter()
            .chain(&load_after)
            .any(|other| other == &id)
        {
            return Err(ModError::SelfDependency(id));
        }
        Ok(Self {
            id,
            name: raw.name,
            version: raw.version,
            game_version: raw.game_version,
            dependencies,
            load_after,
        })
    }
    #[must_use]
    pub const fn id(&self) -> &ModId {
        &self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
    #[must_use]
    pub fn dependencies(&self) -> &[ModId] {
        &self.dependencies
    }
}

/// Resolves dependencies and optional ordering hints with stable lexical tie-breaking.
/// # Errors
/// Returns [`ModError`] for duplicates, missing dependencies, or cycles.
pub fn resolve_load_order(manifests: Vec<ModManifest>) -> Result<Vec<ModId>, ModError> {
    let mut by_id = BTreeMap::new();
    for manifest in manifests {
        if by_id.insert(manifest.id.clone(), manifest).is_some() {
            return Err(ModError::DuplicateMod);
        }
    }
    for manifest in by_id.values() {
        for dependency in &manifest.dependencies {
            if !by_id.contains_key(dependency) {
                return Err(ModError::MissingDependency {
                    mod_id: manifest.id.clone(),
                    dependency: dependency.clone(),
                });
            }
        }
    }
    let mut resolved = Vec::new();
    let mut pending: BTreeSet<_> = by_id.keys().cloned().collect();
    while !pending.is_empty() {
        let next = pending
            .iter()
            .find(|id| {
                let manifest = &by_id[*id];
                manifest
                    .dependencies
                    .iter()
                    .chain(&manifest.load_after)
                    .all(|dependency| {
                        !by_id.contains_key(dependency) || resolved.contains(dependency)
                    })
            })
            .cloned();
        let Some(next) = next else {
            return Err(ModError::DependencyCycle(pending.into_iter().collect()));
        };
        pending.remove(&next);
        resolved.push(next);
    }
    Ok(resolved)
}

fn valid_id(value: &str, require_dot: bool) -> bool {
    !value.is_empty()
        && (!require_dot || value.contains('.'))
        && value.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'-')
        })
}

#[derive(Debug)]
pub enum ModError {
    Toml(toml::de::Error),
    InvalidIdentifier(String),
    EmptyMetadata,
    SelfDependency(ModId),
    DuplicateMod,
    MissingDependency { mod_id: ModId, dependency: ModId },
    DependencyCycle(Vec<ModId>),
}
impl fmt::Display for ModError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Toml(e) => write!(f, "invalid mod manifest: {e}"),
            Self::InvalidIdentifier(v) => write!(f, "invalid mod identifier: {v}"),
            Self::EmptyMetadata => f.write_str("mod metadata cannot be empty"),
            Self::SelfDependency(id) => write!(f, "mod {} depends on itself", id.as_str()),
            Self::DuplicateMod => f.write_str("duplicate mod ID"),
            Self::MissingDependency { mod_id, dependency } => write!(
                f,
                "mod {} requires missing {}",
                mod_id.as_str(),
                dependency.as_str()
            ),
            Self::DependencyCycle(_) => f.write_str("mod dependency cycle"),
        }
    }
}
impl std::error::Error for ModError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    id: String,
    name: String,
    version: String,
    game_version: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    load_after: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(id: &str, deps: &str) -> ModManifest {
        ModManifest::parse_toml(&format!(
            "id='{id}'\nname='Test'\nversion='1.0'\ngame_version='>=0.1'\ndependencies=[{deps}]\n"
        ))
        .expect("manifest")
    }
    #[test]
    fn namespace_is_strict() {
        assert!(NamespacedKey::parse("author.mod:item").is_ok());
        assert!(NamespacedKey::parse("item").is_err());
    }
    #[test]
    fn dependencies_load_first() {
        let order = resolve_load_order(vec![
            manifest("author.addon", "'core.base'"),
            manifest("core.base", ""),
        ])
        .expect("order");
        assert_eq!(order[0].as_str(), "core.base");
    }
    #[test]
    fn missing_dependency_fails() {
        assert!(matches!(
            resolve_load_order(vec![manifest("author.addon", "'missing.base'")]),
            Err(ModError::MissingDependency { .. })
        ));
    }
}
