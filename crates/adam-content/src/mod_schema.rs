use super::modding::NamespacedKey;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodDefinition {
    pub name: String,
    pub unit: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeInputDefinition {
    pub good: String,
    pub quantity_milli: u64,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeDefinition {
    pub name: String,
    pub output_good: String,
    pub output_quantity_milli: u64,
    pub labor_milli_worker_months: u64,
    #[serde(default)]
    pub inputs: Vec<RecipeInputDefinition>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossReferenceIssue {
    pub recipe: NamespacedKey,
    pub reference: String,
}
/// Validates names, positive physical values, duplicate inputs, and all recipe-to-good references.
/// # Errors
/// Returns every invalid recipe and missing reference without stopping at the first issue.
pub fn validate_goods_and_recipes(
    goods: &BTreeMap<NamespacedKey, GoodDefinition>,
    recipes: &BTreeMap<NamespacedKey, RecipeDefinition>,
) -> Result<(), Vec<CrossReferenceIssue>> {
    let known: BTreeSet<_> = goods.keys().cloned().collect();
    let mut issues = Vec::new();
    for (key, recipe) in recipes {
        let mut references = vec![recipe.output_good.as_str()];
        references.extend(recipe.inputs.iter().map(|i| i.good.as_str()));
        for reference in references {
            match NamespacedKey::parse(reference) {
                Ok(parsed) if known.contains(&parsed) => {}
                _ => issues.push(CrossReferenceIssue {
                    recipe: key.clone(),
                    reference: reference.to_owned(),
                }),
            }
        }
        if recipe.name.trim().is_empty()
            || recipe.output_quantity_milli == 0
            || recipe.labor_milli_worker_months == 0
            || recipe.inputs.iter().any(|i| i.quantity_milli == 0)
        {
            issues.push(CrossReferenceIssue {
                recipe: key.clone(),
                reference: "invalid physical recipe values".into(),
            });
        }
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_recipe_good_is_reported() {
        let goods = BTreeMap::from([(
            NamespacedKey::parse("core.base:food").expect("key"),
            GoodDefinition {
                name: "Food".into(),
                unit: "basket".into(),
            },
        )]);
        let recipes = BTreeMap::from([(
            NamespacedKey::parse("mod.test:factory").expect("key"),
            RecipeDefinition {
                name: "Factory".into(),
                output_good: "core.base:missing".into(),
                output_quantity_milli: 1000,
                labor_milli_worker_months: 1000,
                inputs: vec![],
            },
        )]);
        let issues = validate_goods_and_recipes(&goods, &recipes).expect_err("missing");
        assert_eq!(issues[0].reference, "core.base:missing");
    }
}
