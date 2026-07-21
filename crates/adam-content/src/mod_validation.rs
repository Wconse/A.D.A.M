use super::mod_schema::{GoodDefinition, RecipeDefinition, validate_goods_and_recipes};
use super::modding::{ModId, ModManifest, NamespacedKey, resolve_load_order};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ModValidationReport {
    pub manifest: ModManifest,
    pub goods: usize,
    pub recipes: usize,
}
#[derive(Clone, Debug)]
pub struct ModSetValidationReport {
    pub load_order: Vec<ModId>,
    pub goods: usize,
    pub recipes: usize,
    pub package_fingerprint: u64,
}
/// Validates multiple mod folders and resolves their dependency order.
/// # Errors
/// Returns manifest, dependency, file, schema, and reference diagnostics.
pub fn validate_mod_set(paths: &[PathBuf]) -> Result<ModSetValidationReport, Vec<String>> {
    let mut reports = Vec::new();
    let mut issues = Vec::new();
    for path in paths {
        match validate_mod_folder(path) {
            Ok(report) => reports.push(report),
            Err(found) => issues.extend(
                found
                    .into_iter()
                    .map(|issue| format!("{}: {issue}", path.display())),
            ),
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let load_order = resolve_load_order(reports.iter().map(|r| r.manifest.clone()).collect())
        .map_err(|e| vec![e.to_string()])?;
    Ok(ModSetValidationReport {
        package_fingerprint: fingerprint_paths(paths)?,
        load_order,
        goods: reports.iter().map(|r| r.goods).sum(),
        recipes: reports.iter().map(|r| r.recipes).sum(),
    })
}

/// Validates a mod folder containing `mod.toml`, `goods/*.toml`, and `recipes/*.toml`.
/// # Errors
/// Returns all readable diagnostics before any content is accepted.
pub fn validate_mod_folder(path: &Path) -> Result<ModValidationReport, Vec<String>> {
    let manifest_source =
        fs::read_to_string(path.join("mod.toml")).map_err(|e| vec![format!("mod.toml: {e}")])?;
    let manifest =
        ModManifest::parse_toml(&manifest_source).map_err(|e| vec![format!("mod.toml: {e}")])?;
    let mut issues = Vec::new();
    let goods = load_files::<GoodFile>(&path.join("goods"), &mut issues)
        .into_iter()
        .filter_map(|(file, item)| match NamespacedKey::parse(&item.id) {
            Ok(key) if key.namespace() == manifest.id() => Some((key, item.definition)),
            Ok(key) => {
                issues.push(format!(
                    "{}: definition {}:{} is outside mod namespace {}",
                    file.display(),
                    key.namespace().as_str(),
                    key.local(),
                    manifest.id().as_str()
                ));
                None
            }
            Err(e) => {
                issues.push(format!("{}: {e}", file.display()));
                None
            }
        })
        .collect::<BTreeMap<_, _>>();
    let recipes = load_files::<RecipeFile>(&path.join("recipes"), &mut issues)
        .into_iter()
        .filter_map(|(file, item)| match NamespacedKey::parse(&item.id) {
            Ok(key) if key.namespace() == manifest.id() => Some((key, item.definition)),
            Ok(key) => {
                issues.push(format!(
                    "{}: definition {}:{} is outside mod namespace {}",
                    file.display(),
                    key.namespace().as_str(),
                    key.local(),
                    manifest.id().as_str()
                ));
                None
            }
            Err(e) => {
                issues.push(format!("{}: {e}", file.display()));
                None
            }
        })
        .collect::<BTreeMap<_, _>>();
    if let Err(cross) = validate_goods_and_recipes(&goods, &recipes) {
        for issue in cross {
            issues.push(format!(
                "recipe {}:{} -> {}",
                issue.recipe.namespace().as_str(),
                issue.recipe.local(),
                issue.reference
            ));
        }
    }
    if issues.is_empty() {
        Ok(ModValidationReport {
            manifest,
            goods: goods.len(),
            recipes: recipes.len(),
        })
    } else {
        Err(issues)
    }
}
fn fingerprint_paths(paths: &[PathBuf]) -> Result<u64, Vec<String>> {
    let mut files = Vec::new();
    for root in paths {
        collect_toml(root, &mut files).map_err(|e| vec![e])?;
    }
    files.sort();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for file in files {
        let bytes = fs::read(&file).map_err(|e| vec![format!("{}: {e}", file.display())])?;
        for byte in file.to_string_lossy().bytes().chain(bytes) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    Ok(hash)
}
fn collect_toml(path: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().is_some_and(|e| e == "toml") {
            output.push(path.to_owned());
        }
        return Ok(());
    }
    let entries = fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    for entry in entries {
        let child = entry.map_err(|e| e.to_string())?.path();
        if child.is_dir() {
            collect_toml(&child, output)?;
        } else if child.extension().is_some_and(|e| e == "toml") {
            output.push(child);
        }
    }
    Ok(())
}

fn load_files<T: for<'de> Deserialize<'de>>(
    dir: &Path,
    issues: &mut Vec<String>,
) -> Vec<(PathBuf, T)> {
    if !dir.exists() {
        return Vec::new();
    }
    let mut paths = match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect::<Vec<_>>(),
        Err(e) => {
            issues.push(format!("{}: {e}", dir.display()));
            return Vec::new();
        }
    };
    paths.sort();
    paths
        .into_iter()
        .filter_map(|path| match fs::read_to_string(&path) {
            Ok(source) => match toml::from_str(&source) {
                Ok(value) => Some((path, value)),
                Err(e) => {
                    issues.push(format!("{}: {e}", path.display()));
                    None
                }
            },
            Err(e) => {
                issues.push(format!("{}: {e}", path.display()));
                None
            }
        })
        .collect()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GoodFile {
    id: String,
    #[serde(flatten)]
    definition: GoodDefinition,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    id: String,
    #[serde(flatten)]
    definition: RecipeDefinition,
}
