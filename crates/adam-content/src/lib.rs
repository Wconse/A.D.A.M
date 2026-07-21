#![doc = "Versioned TOML content loading for A.D.A.M."]

use std::collections::BTreeSet;
use std::fmt;

use adam_core::{Country, CountryId, SimDate, TimeError, World, WorldError, WorldSeed};
use serde::Deserialize;

pub const WORLD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CountryBlueprint {
    id: CountryId,
    name: String,
}

impl CountryBlueprint {
    #[must_use]
    pub const fn id(&self) -> CountryId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldBlueprint {
    name: String,
    start_year: i32,
    countries: Vec<CountryBlueprint>,
}

impl WorldBlueprint {
    /// Parses and validates a versioned world definition.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] for malformed TOML, unsupported schema versions, empty names,
    /// missing countries, or duplicate country IDs.
    pub fn parse_toml(source: &str) -> Result<Self, ContentError> {
        let raw: RawWorld = toml::from_str(source).map_err(ContentError::Parse)?;
        if raw.schema_version != WORLD_SCHEMA_VERSION {
            return Err(ContentError::UnsupportedSchema {
                expected: WORLD_SCHEMA_VERSION,
                actual: raw.schema_version,
            });
        }
        if raw.world_name.trim().is_empty() {
            return Err(ContentError::EmptyWorldName);
        }
        if raw.countries.is_empty() {
            return Err(ContentError::NoCountries);
        }

        let mut ids = BTreeSet::new();
        let mut countries = Vec::with_capacity(raw.countries.len());
        for raw_country in raw.countries {
            let id = CountryId::new(raw_country.id);
            if !ids.insert(id) {
                return Err(ContentError::DuplicateCountry(id));
            }
            let country = Country::new(id, raw_country.name).map_err(ContentError::Domain)?;
            countries.push(CountryBlueprint {
                id: country.id(),
                name: country.name().to_owned(),
            });
        }

        SimDate::new(raw.start_year, 1).map_err(ContentError::Time)?;
        Ok(Self {
            name: raw.world_name,
            start_year: raw.start_year,
            countries,
        })
    }

    /// Creates authoritative domain state from validated content.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError::Domain`] if a validated country cannot be registered.
    pub fn build_world(&self, seed: WorldSeed) -> Result<World, ContentError> {
        let start_date = SimDate::new(self.start_year, 1).map_err(ContentError::Time)?;
        let mut world = World::new(seed, start_date);
        for blueprint in &self.countries {
            let country =
                Country::new(blueprint.id, blueprint.name.clone()).map_err(ContentError::Domain)?;
            world
                .register_country(country)
                .map_err(ContentError::Domain)?;
        }
        Ok(world)
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn start_year(&self) -> i32 {
        self.start_year
    }

    #[must_use]
    pub fn countries(&self) -> &[CountryBlueprint] {
        &self.countries
    }
}

#[derive(Debug)]
pub enum ContentError {
    Parse(toml::de::Error),
    UnsupportedSchema { expected: u32, actual: u32 },
    EmptyWorldName,
    NoCountries,
    DuplicateCountry(CountryId),
    Domain(WorldError),
    Time(TimeError),
}

impl fmt::Display for ContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(formatter, "invalid world TOML: {error}"),
            Self::UnsupportedSchema { expected, actual } => {
                write!(
                    formatter,
                    "unsupported world schema {actual}; expected {expected}"
                )
            }
            Self::EmptyWorldName => formatter.write_str("world name cannot be empty"),
            Self::NoCountries => formatter.write_str("world must contain at least one country"),
            Self::DuplicateCountry(id) => write!(formatter, "duplicate country ID {id}"),
            Self::Domain(error) => write!(formatter, "invalid domain content: {error}"),
            Self::Time(error) => write!(formatter, "invalid world start date: {error}"),
        }
    }
}

impl std::error::Error for ContentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Domain(error) => Some(error),
            Self::Time(error) => Some(error),
            Self::UnsupportedSchema { .. }
            | Self::EmptyWorldName
            | Self::NoCountries
            | Self::DuplicateCountry(_) => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorld {
    schema_version: u32,
    world_name: String,
    start_year: i32,
    countries: Vec<RawCountry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCountry {
    id: u64,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
schema_version = 1
world_name = "Test World"
start_year = 2025

[[countries]]
id = 1
name = "Aster"

[[countries]]
id = 2
name = "Boreal"
"#;

    #[test]
    fn valid_content_builds_deterministic_world() {
        let blueprint = WorldBlueprint::parse_toml(VALID).expect("valid content");
        let first = blueprint
            .build_world(WorldSeed::new(47))
            .expect("world builds");
        let second = blueprint
            .build_world(WorldSeed::new(47))
            .expect("world builds");
        assert_eq!(first, second);
        assert_eq!(blueprint.name(), "Test World");
        assert_eq!(blueprint.countries().len(), 2);
    }

    #[test]
    fn unsupported_schema_is_explicit() {
        let source = VALID.replace("schema_version = 1", "schema_version = 2");
        let error = WorldBlueprint::parse_toml(&source).expect_err("schema must fail");
        assert!(matches!(
            error,
            ContentError::UnsupportedSchema {
                expected: 1,
                actual: 2
            }
        ));
    }

    #[test]
    fn duplicate_country_ids_are_rejected() {
        let source = VALID.replace("id = 2", "id = 1");
        let error = WorldBlueprint::parse_toml(&source).expect_err("duplicate must fail");
        assert!(matches!(
            error,
            ContentError::DuplicateCountry(id) if id == CountryId::new(1)
        ));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let source = format!("{VALID}\nunexpected = true\n");
        let error = WorldBlueprint::parse_toml(&source).expect_err("unknown field must fail");
        assert!(matches!(error, ContentError::Parse(_)));
    }
}
