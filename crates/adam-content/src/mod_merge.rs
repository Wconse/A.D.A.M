use super::modding::{ModId, ModManifest, NamespacedKey, resolve_load_order};
use super::registry::{ContentRegistry, PatchOperation};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug)]
pub struct MergedModSet {
    pub load_order: Vec<ModId>,
    pub registry: ContentRegistry,
    pub content_fingerprint: u64,
}
/// Loads definitions and patch files from multiple mods in dependency order.
/// # Errors
/// Returns all manifest, dependency, file, namespace, duplicate, patch, and path diagnostics.
pub fn merge_mod_folders(paths: &[PathBuf]) -> Result<MergedModSet, Vec<String>> {
    let mut folders = BTreeMap::new();
    let mut manifests = Vec::new();
    let mut issues = Vec::new();
    for path in paths {
        match read_manifest(path) {
            Ok(manifest) => {
                folders.insert(manifest.id().clone(), path.clone());
                manifests.push(manifest);
            }
            Err(e) => issues.push(e),
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let order = resolve_load_order(manifests).map_err(|e| vec![e.to_string()])?;
    let mut registry = ContentRegistry::default();
    for id in &order {
        let path = &folders[id];
        for dir in ["goods", "recipes"] {
            for file in toml_files(&path.join(dir), &mut issues) {
                match definition_file(&file) {
                    Ok((key, value)) => {
                        if key.namespace() != id {
                            issues.push(format!(
                                "{}: definition is outside {}",
                                file.display(),
                                id.as_str()
                            ));
                        } else if let Err(e) = registry.add(id.clone(), key, value) {
                            issues.push(format!("{}: {e}", file.display()));
                        }
                    }
                    Err(e) => issues.push(format!("{}: {e}", file.display())),
                }
            }
        }
        for file in toml_files(&path.join("patches"), &mut issues) {
            match patch_file(&file) {
                Ok((target, operations)) => {
                    if let Err(e) = registry.patch(id, &target, &operations) {
                        issues.push(format!("{}: {e}", file.display()));
                    }
                }
                Err(e) => issues.push(format!("{}: {e}", file.display())),
            }
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let fingerprint = registry.content_fingerprint();
    Ok(MergedModSet {
        load_order: order,
        registry,
        content_fingerprint: fingerprint,
    })
}
fn read_manifest(path: &Path) -> Result<ModManifest, String> {
    let file = path.join("mod.toml");
    let source = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    ModManifest::parse_toml(&source).map_err(|e| format!("{}: {e}", file.display()))
}
fn toml_files(dir: &Path, issues: &mut Vec<String>) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut files = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect(),
        Err(e) => {
            issues.push(format!("{}: {e}", dir.display()));
            Vec::new()
        }
    };
    files.sort();
    files
}
fn definition_file(path: &Path) -> Result<(NamespacedKey, Value), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut value: Value = toml::from_str(&source).map_err(|e| e.to_string())?;
    let table = value.as_table_mut().ok_or("definition must be a table")?;
    let id = table
        .remove("id")
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or("definition requires string id")?;
    Ok((NamespacedKey::parse(&id).map_err(|e| e.to_string())?, value))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPatch {
    target: String,
    operations: Vec<RawOperation>,
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum RawOperation {
    Set { path: String, value: Value },
    Remove { path: String },
    Append { path: String, value: Value },
}
fn patch_file(path: &Path) -> Result<(NamespacedKey, Vec<PatchOperation>), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let raw: RawPatch = toml::from_str(&source).map_err(|e| e.to_string())?;
    let operations = raw
        .operations
        .into_iter()
        .map(|op| match op {
            RawOperation::Set { path, value } => PatchOperation::Set { path, value },
            RawOperation::Remove { path } => PatchOperation::Remove { path },
            RawOperation::Append { path, value } => PatchOperation::Append { path, value },
        })
        .collect();
    Ok((
        NamespacedKey::parse(&raw.target).map_err(|e| e.to_string())?,
        operations,
    ))
}
