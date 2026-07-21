#![doc = "Versioned TOML content loading for A.D.A.M."]

use std::fmt;

use adam_core::{
    Actor, ActorId, BasisPoints, Country, CountryId, Influence, Money, Population, PowerNode,
    PowerNodeId, PowerNodeKind, Region, RegionId, SimDate, TimeError, ValueError, World,
    WorldError, WorldSeed,
};
use serde::Deserialize;

pub const WORLD_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldBlueprint {
    name: String,
    start_year: i32,
    countries: Vec<Country>,
    regions: Vec<Region>,
    actors: Vec<Actor>,
    power_nodes: Vec<PowerNode>,
    influences: Vec<Influence>,
}

impl WorldBlueprint {
    /// Parses and validates a versioned world definition and all cross-references.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError`] for malformed TOML, unsupported schema versions, invalid
    /// canonical values, duplicate identities, or dangling references.
    pub fn parse_toml(source: &str) -> Result<Self, ContentError> {
        let raw: RawWorld = toml::from_str(source).map_err(ContentError::Parse)?;
        validate_header(&raw)?;
        let blueprint = Self {
            name: raw.world_name,
            start_year: raw.start_year,
            countries: parse_countries(raw.countries)?,
            regions: parse_regions(raw.regions)?,
            actors: parse_actors(raw.actors, raw.start_year)?,
            power_nodes: parse_power_nodes(raw.power_nodes)?,
            influences: parse_influences(raw.influences)?,
        };
        SimDate::new(blueprint.start_year, 1).map_err(ContentError::Time)?;
        blueprint.build_world(WorldSeed::new(0))?;
        Ok(blueprint)
    }

    /// Creates authoritative domain state from fully validated content.
    ///
    /// # Errors
    ///
    /// Returns [`ContentError::Domain`] if a content invariant no longer matches the domain.
    pub fn build_world(&self, seed: WorldSeed) -> Result<World, ContentError> {
        let start_date = SimDate::new(self.start_year, 1).map_err(ContentError::Time)?;
        let mut world = World::new(seed, start_date);
        for country in &self.countries {
            world
                .register_country(country.clone())
                .map_err(ContentError::Domain)?;
        }
        for region in &self.regions {
            world
                .register_region(region.clone())
                .map_err(ContentError::Domain)?;
        }
        for actor in &self.actors {
            world
                .register_actor(actor.clone())
                .map_err(ContentError::Domain)?;
        }
        for node in &self.power_nodes {
            world
                .register_power_node(node.clone())
                .map_err(ContentError::Domain)?;
        }
        for influence in &self.influences {
            world
                .establish_influence(*influence)
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
    pub fn countries(&self) -> &[Country] {
        &self.countries
    }

    #[must_use]
    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    #[must_use]
    pub fn actors(&self) -> &[Actor] {
        &self.actors
    }

    #[must_use]
    pub fn power_nodes(&self) -> &[PowerNode] {
        &self.power_nodes
    }

    #[must_use]
    pub fn influences(&self) -> &[Influence] {
        &self.influences
    }
}

fn validate_header(raw: &RawWorld) -> Result<(), ContentError> {
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
    Ok(())
}

fn parse_countries(raw: Vec<RawCountry>) -> Result<Vec<Country>, ContentError> {
    raw.into_iter()
        .map(|country| {
            let id = CountryId::new(non_zero_id("country", country.id)?);
            Country::new(id, country.name).map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_regions(raw: Vec<RawRegion>) -> Result<Vec<Region>, ContentError> {
    raw.into_iter()
        .map(|region| {
            if region.annual_output_minor < 0 {
                return Err(ContentError::NegativeAnnualOutput {
                    region: region.id,
                    value: region.annual_output_minor,
                });
            }
            Region::new(
                RegionId::new(non_zero_id("region", region.id)?),
                CountryId::new(non_zero_id("region country", region.country_id)?),
                region.name,
                Population::new(region.population),
                Money::from_minor_units(region.annual_output_minor),
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_actors(raw: Vec<RawActor>, start_year: i32) -> Result<Vec<Actor>, ContentError> {
    raw.into_iter()
        .map(|actor| {
            if actor.born_year > start_year {
                return Err(ContentError::FutureActorBirth {
                    actor: actor.id,
                    born_year: actor.born_year,
                    start_year,
                });
            }
            Actor::new(
                ActorId::new(non_zero_id("actor", actor.id)?),
                actor.name,
                RegionId::new(non_zero_id("actor home region", actor.home_region_id)?),
                actor.born_year,
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_power_nodes(raw: Vec<RawPowerNode>) -> Result<Vec<PowerNode>, ContentError> {
    raw.into_iter()
        .map(|node| {
            PowerNode::new(
                PowerNodeId::new(non_zero_id("power node", node.id)?),
                CountryId::new(non_zero_id("power node country", node.country_id)?),
                node.name,
                node.kind.into(),
                node.holder_actor_id
                    .map(|id| non_zero_id("power node holder", id).map(ActorId::new))
                    .transpose()?,
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_influences(raw: Vec<RawInfluence>) -> Result<Vec<Influence>, ContentError> {
    raw.into_iter()
        .map(|influence| {
            let weight = BasisPoints::new(influence.weight_bps).map_err(ContentError::Value)?;
            if weight.get() == 0 {
                return Err(ContentError::ZeroInfluence {
                    actor: influence.actor_id,
                    node: influence.node_id,
                });
            }
            Ok(Influence::new(
                ActorId::new(non_zero_id("influence actor", influence.actor_id)?),
                PowerNodeId::new(non_zero_id("influence node", influence.node_id)?),
                weight,
            ))
        })
        .collect()
}

fn non_zero_id(kind: &'static str, value: u32) -> Result<u32, ContentError> {
    if value == 0 {
        Err(ContentError::ZeroId(kind))
    } else {
        Ok(value)
    }
}

#[derive(Debug)]
pub enum ContentError {
    Parse(toml::de::Error),
    UnsupportedSchema {
        expected: u32,
        actual: u32,
    },
    EmptyWorldName,
    NoCountries,
    ZeroId(&'static str),
    NegativeAnnualOutput {
        region: u32,
        value: i64,
    },
    FutureActorBirth {
        actor: u32,
        born_year: i32,
        start_year: i32,
    },
    ZeroInfluence {
        actor: u32,
        node: u32,
    },
    Domain(WorldError),
    Time(TimeError),
    Value(ValueError),
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
            Self::ZeroId(kind) => write!(formatter, "{kind} ID zero is reserved"),
            Self::NegativeAnnualOutput { region, value } => {
                write!(
                    formatter,
                    "region {region} has negative annual output {value}"
                )
            }
            Self::FutureActorBirth {
                actor,
                born_year,
                start_year,
            } => write!(
                formatter,
                "actor {actor} birth year {born_year} is after world start {start_year}"
            ),
            Self::ZeroInfluence { actor, node } => {
                write!(
                    formatter,
                    "influence from actor {actor} to node {node} is zero"
                )
            }
            Self::Domain(error) => write!(formatter, "invalid domain content: {error}"),
            Self::Time(error) => write!(formatter, "invalid world start date: {error}"),
            Self::Value(error) => write!(formatter, "invalid canonical value: {error}"),
        }
    }
}

impl std::error::Error for ContentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Domain(error) => Some(error),
            Self::Time(error) => Some(error),
            Self::Value(error) => Some(error),
            Self::UnsupportedSchema { .. }
            | Self::EmptyWorldName
            | Self::NoCountries
            | Self::ZeroId(_)
            | Self::NegativeAnnualOutput { .. }
            | Self::FutureActorBirth { .. }
            | Self::ZeroInfluence { .. } => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorld {
    schema_version: u32,
    world_name: String,
    start_year: i32,
    #[serde(default)]
    countries: Vec<RawCountry>,
    #[serde(default)]
    regions: Vec<RawRegion>,
    #[serde(default)]
    actors: Vec<RawActor>,
    #[serde(default)]
    power_nodes: Vec<RawPowerNode>,
    #[serde(default)]
    influences: Vec<RawInfluence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCountry {
    id: u32,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegion {
    id: u32,
    country_id: u32,
    name: String,
    population: u64,
    annual_output_minor: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawActor {
    id: u32,
    name: String,
    home_region_id: u32,
    born_year: i32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawPowerNodeKind {
    PoliticalOffice,
    Capital,
    MilitaryCommand,
    MediaPlatform,
    CivicOrganization,
}

impl From<RawPowerNodeKind> for PowerNodeKind {
    fn from(value: RawPowerNodeKind) -> Self {
        match value {
            RawPowerNodeKind::PoliticalOffice => Self::PoliticalOffice,
            RawPowerNodeKind::Capital => Self::Capital,
            RawPowerNodeKind::MilitaryCommand => Self::MilitaryCommand,
            RawPowerNodeKind::MediaPlatform => Self::MediaPlatform,
            RawPowerNodeKind::CivicOrganization => Self::CivicOrganization,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPowerNode {
    id: u32,
    country_id: u32,
    name: String,
    kind: RawPowerNodeKind,
    holder_actor_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInfluence {
    actor_id: u32,
    node_id: u32,
    weight_bps: u16,
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

[[regions]]
id = 10
country_id = 1
name = "Aster Capital"
population = 1000000
annual_output_minor = 5000000000

[[actors]]
id = 100
name = "Ada Vale"
home_region_id = 10
born_year = 1980

[[power_nodes]]
id = 1000
country_id = 1
name = "Presidency"
kind = "political_office"
holder_actor_id = 100

[[influences]]
actor_id = 100
node_id = 1000
weight_bps = 8000
"#;

    #[test]
    fn valid_content_builds_complete_deterministic_world() {
        let blueprint = WorldBlueprint::parse_toml(VALID).expect("valid content");
        let first = blueprint
            .build_world(WorldSeed::new(47))
            .expect("world builds");
        let second = blueprint
            .build_world(WorldSeed::new(47))
            .expect("world builds");
        assert_eq!(first, second);
        assert_eq!(blueprint.regions().len(), 1);
        assert_eq!(blueprint.actors().len(), 1);
        assert_eq!(blueprint.power_nodes().len(), 1);
        assert_eq!(blueprint.influences().len(), 1);
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
    fn dangling_references_are_rejected() {
        let source = VALID.replace("home_region_id = 10", "home_region_id = 999");
        let error = WorldBlueprint::parse_toml(&source).expect_err("reference must fail");
        assert!(matches!(
            error,
            ContentError::Domain(WorldError::UnknownRegion(id)) if id == RegionId::new(999)
        ));
    }

    #[test]
    fn out_of_range_influence_is_rejected() {
        let source = VALID.replace("weight_bps = 8000", "weight_bps = 10001");
        let error = WorldBlueprint::parse_toml(&source).expect_err("weight must fail");
        assert!(matches!(
            error,
            ContentError::Value(ValueError::BasisPointsOutOfRange(10_001))
        ));
    }

    #[test]
    fn zero_ids_are_reserved() {
        let source = VALID.replace("id = 1\nname = \"Aster\"", "id = 0\nname = \"Aster\"");
        let error = WorldBlueprint::parse_toml(&source).expect_err("zero ID must fail");
        assert!(matches!(error, ContentError::ZeroId("country")));
    }
}
