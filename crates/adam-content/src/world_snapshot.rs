use super::save_compat::{SaveCompatibilityMetadata, compatibility_issues};
use super::save_envelope::{SAVE_FORMAT_VERSION, SaveEnvelope, SaveEnvelopeError};
use adam_core::World;

/// Encodes the complete authoritative world into a versioned binary envelope.
/// # Errors
/// Returns [`SaveEnvelopeError::Encode`] if any world field cannot be serialized.
pub fn encode_world_save(
    world: &World,
    compatibility: SaveCompatibilityMetadata,
) -> Result<Vec<u8>, SaveEnvelopeError> {
    bincode::serialize(&SaveEnvelope::new(compatibility, world.clone()))
        .map_err(|e| SaveEnvelopeError::Encode(e.to_string()))
}
/// Decodes a complete world only after format and exact mod compatibility checks.
/// # Errors
/// Returns [`SaveEnvelopeError`] for corrupt data, unsupported format, or incompatible content.
pub fn decode_world_save(
    bytes: &[u8],
    current: &SaveCompatibilityMetadata,
) -> Result<World, SaveEnvelopeError> {
    let envelope: SaveEnvelope<World> =
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
    use crate::save_compat::SavedModReference;
    use adam_core::{
        Country, CountryId, Firm, FirmExpectationSource, FirmExpectations, FirmId, Good, GoodId,
        Money, Population, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate,
        WorldSeed,
    };
    use std::collections::BTreeMap;
    fn metadata() -> SaveCompatibilityMetadata {
        SaveCompatibilityMetadata {
            simulation_version: adam_core::SIMULATION_VERSION,
            world_schema_version: crate::WORLD_SCHEMA_VERSION,
            mods: vec![SavedModReference {
                id: "core.base".into(),
                version: "1".into(),
            }],
            package_fingerprint: 1,
            content_fingerprint: 2,
        }
    }
    fn world() -> World {
        let mut world = World::new(WorldSeed::new(47), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register");
        world.advance_years(20).expect("years");
        world
    }
    fn world_with_firm_expectations() -> World {
        let mut world = World::new(WorldSeed::new(47), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Output").expect("good"))
            .expect("good");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(1),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Output recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Firm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(5_000),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .update_firm_expectations(
                FirmId::new(1),
                FirmExpectations::new(
                    Money::from_minor_units(8_000),
                    Money::from_minor_units(2_000),
                    Money::from_minor_units(1_000),
                    2,
                    FirmExpectationSource::Management,
                )
                .expect("expectations"),
            )
            .expect("expectation update");
        world
    }
    #[test]
    fn full_world_round_trip_and_continuation_are_identical() {
        let original = world();
        let bytes = encode_world_save(&original, metadata()).expect("save");
        let mut loaded = decode_world_save(&bytes, &metadata()).expect("load");
        let mut uninterrupted = original;
        loaded.advance_years(30).expect("continue");
        uninterrupted.advance_years(30).expect("continue");
        assert_eq!(loaded, uninterrupted);
        assert_eq!(
            loaded.stable_fingerprint(),
            uninterrupted.stable_fingerprint()
        );
    }
    #[test]
    fn firm_expectations_survive_world_save_round_trip() {
        let original = world_with_firm_expectations();
        let bytes = encode_world_save(&original, metadata()).expect("save");
        let loaded = decode_world_save(&bytes, &metadata()).expect("load");
        assert_eq!(loaded, original);
        assert_eq!(loaded.stable_fingerprint(), original.stable_fingerprint());
        assert_eq!(
            loaded.firm_expectations()[&FirmId::new(1)].horizon_months(),
            2
        );
    }
    #[test]
    fn incompatible_mod_content_blocks_world_decode() {
        let bytes = encode_world_save(&world(), metadata()).expect("save");
        let mut current = metadata();
        current.content_fingerprint = 99;
        assert!(matches!(
            decode_world_save(&bytes, &current),
            Err(SaveEnvelopeError::Incompatible(_))
        ));
    }
}
