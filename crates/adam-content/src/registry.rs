use std::collections::BTreeMap;
use std::fmt;

use toml::Value;

use super::modding::{ModId, NamespacedKey};

#[derive(Clone, Debug)]
pub struct RegistryEntry {
    source: ModId,
    history: Vec<ModId>,
    value: Value,
}
impl RegistryEntry {
    #[must_use]
    pub const fn source(&self) -> &ModId {
        &self.source
    }
    #[must_use]
    pub fn history(&self) -> &[ModId] {
        &self.history
    }
    #[must_use]
    pub const fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Clone, Debug)]
pub enum PatchOperation {
    Set { path: String, value: Value },
    Remove { path: String },
    Append { path: String, value: Value },
}

#[derive(Clone, Debug, Default)]
pub struct ContentRegistry {
    entries: BTreeMap<NamespacedKey, RegistryEntry>,
}
impl ContentRegistry {
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<NamespacedKey, RegistryEntry> {
        &self.entries
    }
    /// Computes a canonical fingerprint independent of file and table insertion order.
    #[must_use]
    pub fn content_fingerprint(&self) -> u64 {
        let mut hash = StableHash::new();
        for (key, entry) in &self.entries {
            hash.text(key.namespace().as_str());
            hash.text(key.local());
            hash_value(&mut hash, entry.value());
        }
        hash.finish()
    }
    /// Adds a new namespaced definition with provenance.
    /// # Errors
    /// Returns [`RegistryError::DuplicateDefinition`] instead of silently overriding content.
    pub fn add(
        &mut self,
        source: ModId,
        key: NamespacedKey,
        value: Value,
    ) -> Result<(), RegistryError> {
        if self.entries.contains_key(&key) {
            return Err(RegistryError::DuplicateDefinition(key));
        }
        self.entries.insert(
            key,
            RegistryEntry {
                history: vec![source.clone()],
                source,
                value,
            },
        );
        Ok(())
    }
    /// Applies explicit patch operations in their declared order.
    /// # Errors
    /// Returns [`RegistryError`] for unknown targets, invalid paths, or type mismatches.
    pub fn patch(
        &mut self,
        source: &ModId,
        target: &NamespacedKey,
        operations: &[PatchOperation],
    ) -> Result<(), RegistryError> {
        let entry = self
            .entries
            .get_mut(target)
            .ok_or_else(|| RegistryError::UnknownTarget(target.clone()))?;
        for operation in operations {
            match operation {
                PatchOperation::Set { path, value } => {
                    let slot = resolve_parent_mut(&mut entry.value, path, true)?;
                    *slot = value.clone();
                }
                PatchOperation::Remove { path } => remove_path(&mut entry.value, path)?,
                PatchOperation::Append { path, value } => {
                    let slot = resolve_parent_mut(&mut entry.value, path, false)?;
                    let array = slot
                        .as_array_mut()
                        .ok_or_else(|| RegistryError::TypeMismatch(path.clone()))?;
                    array.push(value.clone());
                }
            }
        }
        entry.source = source.clone();
        entry.history.push(source.clone());
        Ok(())
    }
}

struct StableHash(u64);
impl StableHash {
    const fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
    fn text(&mut self, value: &str) {
        self.bytes(&(value.len() as u64).to_le_bytes());
        self.bytes(value.as_bytes());
    }
    const fn finish(self) -> u64 {
        self.0
    }
}
fn hash_value(hash: &mut StableHash, value: &Value) {
    match value {
        Value::String(v) => {
            hash.bytes(&[1]);
            hash.text(v);
        }
        Value::Integer(v) => {
            hash.bytes(&[2]);
            hash.bytes(&v.to_le_bytes());
        }
        Value::Float(v) => {
            hash.bytes(&[3]);
            hash.bytes(&v.to_bits().to_le_bytes());
        }
        Value::Boolean(v) => {
            hash.bytes(&[4, u8::from(*v)]);
        }
        Value::Datetime(v) => {
            hash.bytes(&[5]);
            hash.text(&v.to_string());
        }
        Value::Array(values) => {
            hash.bytes(&[6]);
            hash.bytes(&(values.len() as u64).to_le_bytes());
            for item in values {
                hash_value(hash, item);
            }
        }
        Value::Table(table) => {
            hash.bytes(&[7]);
            let mut keys: Vec<_> = table.keys().collect();
            keys.sort();
            for key in keys {
                hash.text(key);
                hash_value(hash, &table[key]);
            }
        }
    }
}

fn path_parts(path: &str) -> Result<Vec<&str>, RegistryError> {
    let parts: Vec<_> = path.split('.').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        Err(RegistryError::InvalidPath(path.to_owned()))
    } else {
        Ok(parts)
    }
}
fn resolve_parent_mut<'a>(
    root: &'a mut Value,
    path: &str,
    create_leaf: bool,
) -> Result<&'a mut Value, RegistryError> {
    let parts = path_parts(path)?;
    let mut current = root;
    for (index, part) in parts.iter().enumerate() {
        let table = current
            .as_table_mut()
            .ok_or_else(|| RegistryError::TypeMismatch(path.to_owned()))?;
        let last = index + 1 == parts.len();
        if last && create_leaf {
            return Ok(table
                .entry((*part).to_owned())
                .or_insert(Value::String(String::new())));
        }
        current = table
            .get_mut(*part)
            .ok_or_else(|| RegistryError::MissingPath(path.to_owned()))?;
    }
    Ok(current)
}
fn remove_path(root: &mut Value, path: &str) -> Result<(), RegistryError> {
    let parts = path_parts(path)?;
    let (leaf, parents) = parts.split_last().expect("non-empty path");
    let mut current = root;
    for part in parents {
        current = current
            .as_table_mut()
            .ok_or_else(|| RegistryError::TypeMismatch(path.to_owned()))?
            .get_mut(*part)
            .ok_or_else(|| RegistryError::MissingPath(path.to_owned()))?;
    }
    current
        .as_table_mut()
        .ok_or_else(|| RegistryError::TypeMismatch(path.to_owned()))?
        .remove(*leaf)
        .ok_or_else(|| RegistryError::MissingPath(path.to_owned()))?;
    Ok(())
}

#[derive(Clone, Debug)]
pub enum RegistryError {
    DuplicateDefinition(NamespacedKey),
    UnknownTarget(NamespacedKey),
    InvalidPath(String),
    MissingPath(String),
    TypeMismatch(String),
}
impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateDefinition(_) => f.write_str("duplicate content definition"),
            Self::UnknownTarget(_) => f.write_str("unknown patch target"),
            Self::InvalidPath(p) => write!(f, "invalid patch path: {p}"),
            Self::MissingPath(p) => write!(f, "missing patch path: {p}"),
            Self::TypeMismatch(p) => write!(f, "patch type mismatch at: {p}"),
        }
    }
}
impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(v: &str) -> ModId {
        ModId::parse(v).expect("id")
    }
    fn key(v: &str) -> NamespacedKey {
        NamespacedKey::parse(v).expect("key")
    }
    #[test]
    fn duplicate_add_is_explicit() {
        let mut r = ContentRegistry::default();
        r.add(
            id("core.base"),
            key("core.base:item"),
            Value::Table(toml::map::Map::default()),
        )
        .expect("add");
        assert!(matches!(
            r.add(
                id("mod.test"),
                key("core.base:item"),
                Value::Table(toml::map::Map::default())
            ),
            Err(RegistryError::DuplicateDefinition(_))
        ));
    }
    #[test]
    fn patches_are_explicit_and_ordered() {
        let mut r = ContentRegistry::default();
        let value: Value = "price = 10\ntags=['base']".parse().expect("toml");
        r.add(id("core.base"), key("core.base:item"), value)
            .expect("add");
        r.patch(
            &id("mod.test"),
            &key("core.base:item"),
            &[
                PatchOperation::Set {
                    path: "price".into(),
                    value: Value::Integer(20),
                },
                PatchOperation::Append {
                    path: "tags".into(),
                    value: Value::String("modded".into()),
                },
            ],
        )
        .expect("patch");
        let entry = &r.entries()[&key("core.base:item")];
        assert_eq!(entry.value()["price"].as_integer(), Some(20));
        assert_eq!(entry.value()["tags"].as_array().expect("array").len(), 2);
        assert_eq!(entry.source().as_str(), "mod.test");
        assert_eq!(entry.history().len(), 2);
        assert_ne!(r.content_fingerprint(), 0);
    }
}
