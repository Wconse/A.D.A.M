#![doc = "Versioned TOML content loading for A.D.A.M."]

pub mod mod_merge;
pub mod mod_schema;
pub mod mod_validation;
pub mod modding;
pub mod registry;
pub mod replay_save;
pub mod save_compat;
pub mod save_envelope;
pub mod save_file;
pub mod world_snapshot;

use std::collections::BTreeMap;
use std::fmt;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, CountryIndicators, DemandBasis, EducationLevel,
    EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId,
    HouseholdCohort, HouseholdType, Influence, Money, NeedProfileId, NeedTier, OwnershipStake,
    Population, PowerNode, PowerNodeId, PowerNodeKind, ProductionInput, ProductionRecipe,
    QuantityMilli, RecipeId, Region, RegionId, SimDate, TimeError, ValueError, World, WorldError,
    WorldSeed,
};
use serde::Deserialize;

pub const WORLD_SCHEMA_VERSION: u32 = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldBlueprint {
    name: String,
    start_year: i32,
    countries: Vec<Country>,
    goods: Vec<Good>,
    production_recipes: Vec<ProductionRecipe>,
    firms: Vec<Firm>,
    employment_agreements: Vec<EmploymentAgreement>,
    ownership_stakes: Vec<OwnershipStake>,
    firm_appointments: Vec<FirmAppointment>,
    firm_policies: Vec<(ActorId, FirmId, FirmPolicy)>,
    production_targets: Vec<(ActorId, FirmId, u64)>,
    consumption_profiles: Vec<ConsumptionProfile>,
    regional_prices: Vec<(RegionId, GoodId, Money)>,
    regions: Vec<Region>,
    cohorts: Vec<HouseholdCohort>,
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
            goods: parse_goods(raw.goods)?,
            production_recipes: parse_production_recipes(raw.production_recipes)?,
            firms: parse_firms(raw.firms)?,
            employment_agreements: parse_employment_agreements(raw.employment_agreements)?,
            ownership_stakes: parse_ownership_stakes(raw.ownership_stakes)?,
            firm_appointments: parse_firm_appointments(raw.firm_appointments)?,
            firm_policies: parse_firm_policies(raw.firm_policies)?,
            production_targets: parse_production_targets(raw.production_targets)?,
            consumption_profiles: parse_profiles(raw.consumption_profiles)?,
            regional_prices: parse_prices(raw.regional_prices)?,
            regions: parse_regions(raw.regions)?,
            cohorts: parse_cohorts(raw.cohorts)?,
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
        for good in &self.goods {
            world
                .register_good(good.clone())
                .map_err(ContentError::Domain)?;
        }
        for recipe in &self.production_recipes {
            world
                .register_production_recipe(recipe.clone())
                .map_err(ContentError::Domain)?;
        }
        for profile in &self.consumption_profiles {
            world
                .register_consumption_profile(profile.clone())
                .map_err(ContentError::Domain)?;
        }
        for region in &self.regions {
            world
                .register_region(region.clone())
                .map_err(ContentError::Domain)?;
        }
        for firm in &self.firms {
            world
                .register_firm(firm.clone())
                .map_err(ContentError::Domain)?;
        }
        for (region, good, price) in &self.regional_prices {
            world
                .set_regional_price(*region, *good, *price)
                .map_err(ContentError::Domain)?;
        }
        for cohort in &self.cohorts {
            world
                .register_household_cohort(cohort.clone())
                .map_err(ContentError::Domain)?;
        }
        world
            .validate_population_accounting()
            .map_err(ContentError::Domain)?;
        for agreement in &self.employment_agreements {
            world
                .register_employment_agreement(agreement.clone())
                .map_err(ContentError::Domain)?;
        }
        for actor in &self.actors {
            world
                .register_actor(actor.clone())
                .map_err(ContentError::Domain)?;
        }
        for stake in &self.ownership_stakes {
            world
                .register_ownership_stake(*stake)
                .map_err(ContentError::Domain)?;
        }
        for appointment in &self.firm_appointments {
            world
                .register_firm_appointment(*appointment)
                .map_err(ContentError::Domain)?;
        }
        for (actor, firm, policy) in &self.firm_policies {
            world
                .set_firm_policy(*actor, *firm, *policy)
                .map_err(ContentError::Domain)?;
        }
        for (actor, firm, batches) in &self.production_targets {
            world
                .set_firm_production_target(*actor, *firm, *batches)
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
    pub fn cohorts(&self) -> &[HouseholdCohort] {
        &self.cohorts
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
            if country.treasury_minor < 0 {
                return Err(ContentError::NegativeCountryStock {
                    country: country.id,
                    field: "treasury",
                    value: country.treasury_minor,
                });
            }
            if country.public_debt_minor < 0 {
                return Err(ContentError::NegativeCountryStock {
                    country: country.id,
                    field: "public debt",
                    value: country.public_debt_minor,
                });
            }
            let indicators = CountryIndicators::new(
                Money::from_minor_units(country.treasury_minor),
                Money::from_minor_units(country.public_debt_minor),
                BasisPoints::new(country.legitimacy_bps).map_err(ContentError::Value)?,
                BasisPoints::new(country.elite_cohesion_bps).map_err(ContentError::Value)?,
            );
            Country::new(id, country.name)
                .map(|value| value.with_indicators(indicators))
                .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_goods(raw: Vec<RawGood>) -> Result<Vec<Good>, ContentError> {
    raw.into_iter()
        .map(|good| {
            Good::new(GoodId::new(non_zero_id("good", good.id)?), good.name)
                .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_profiles(
    raw: Vec<RawConsumptionProfile>,
) -> Result<Vec<ConsumptionProfile>, ContentError> {
    raw.into_iter()
        .map(|profile| {
            let targets = profile
                .targets
                .into_iter()
                .map(|target| {
                    Ok(ConsumptionTarget::new(
                        GoodId::new(non_zero_id("consumption target good", target.good_id)?),
                        target.tier.into(),
                        target.basis.into(),
                        QuantityMilli::new(target.monthly_quantity_milli),
                    ))
                })
                .collect::<Result<Vec<_>, ContentError>>()?;
            ConsumptionProfile::new(
                NeedProfileId::new(non_zero_id("need profile", profile.id)?),
                profile.name,
                targets,
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_prices(
    raw: Vec<RawRegionalPrice>,
) -> Result<Vec<(RegionId, GoodId, Money)>, ContentError> {
    raw.into_iter()
        .map(|price| {
            Ok((
                RegionId::new(non_zero_id("price region", price.region_id)?),
                GoodId::new(non_zero_id("price good", price.good_id)?),
                Money::from_minor_units(price.price_minor),
            ))
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

fn parse_cohorts(raw: Vec<RawCohort>) -> Result<Vec<HouseholdCohort>, ContentError> {
    raw.into_iter()
        .map(|cohort| {
            HouseholdCohort::new(
                CohortId::new(non_zero_id("cohort", cohort.id)?),
                RegionId::new(non_zero_id("cohort region", cohort.region_id)?),
                NeedProfileId::new(non_zero_id("cohort need profile", cohort.need_profile_id)?),
                Population::new(cohort.people),
                cohort.households,
                cohort.age_band.into(),
                cohort.household_type.into(),
                cohort.education.into(),
                cohort.employment.into(),
                Money::from_minor_units(cohort.annual_income_minor),
                Money::from_minor_units(cohort.liquid_wealth_minor),
                Money::from_minor_units(cohort.debt_minor),
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_production_recipes(
    raw: Vec<RawProductionRecipe>,
) -> Result<Vec<ProductionRecipe>, ContentError> {
    raw.into_iter()
        .map(|recipe| {
            let inputs = recipe
                .inputs
                .into_iter()
                .map(|input| {
                    Ok(ProductionInput::new(
                        GoodId::new(non_zero_id("production input good", input.good_id)?),
                        QuantityMilli::new(input.quantity_per_batch_milli),
                    ))
                })
                .collect::<Result<Vec<_>, ContentError>>()?;
            ProductionRecipe::new(
                RecipeId::new(non_zero_id("recipe", recipe.id)?),
                recipe.name,
                GoodId::new(non_zero_id("recipe output good", recipe.output_good_id)?),
                QuantityMilli::new(recipe.output_per_batch_milli),
                recipe.labor_milli_worker_months,
                inputs,
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_firms(raw: Vec<RawFirm>) -> Result<Vec<Firm>, ContentError> {
    raw.into_iter()
        .map(|firm| {
            let inventories = firm
                .inventories
                .into_iter()
                .map(|row| {
                    Ok((
                        GoodId::new(non_zero_id("firm inventory good", row.good_id)?),
                        QuantityMilli::new(row.quantity_milli),
                    ))
                })
                .collect::<Result<BTreeMap<_, _>, ContentError>>()?;
            Firm::new(
                FirmId::new(non_zero_id("firm", firm.id)?),
                firm.name,
                RegionId::new(non_zero_id("firm region", firm.region_id)?),
                RecipeId::new(non_zero_id("firm recipe", firm.recipe_id)?),
                firm.workers,
                firm.capacity_batches,
                Money::from_minor_units(firm.cash_minor),
                inventories,
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_employment_agreements(
    raw: Vec<RawEmploymentAgreement>,
) -> Result<Vec<EmploymentAgreement>, ContentError> {
    raw.into_iter()
        .map(|row| {
            EmploymentAgreement::new(
                FirmId::new(non_zero_id("employment firm", row.firm_id)?),
                CohortId::new(non_zero_id("employment cohort", row.cohort_id)?),
                row.workers,
                Money::from_minor_units(row.monthly_wage_per_worker_minor),
            )
            .map_err(ContentError::Domain)
        })
        .collect()
}

fn parse_ownership_stakes(
    raw: Vec<RawOwnershipStake>,
) -> Result<Vec<OwnershipStake>, ContentError> {
    raw.into_iter()
        .map(|row| {
            Ok(OwnershipStake::new(
                FirmId::new(non_zero_id("ownership firm", row.firm_id)?),
                ActorId::new(non_zero_id("ownership actor", row.actor_id)?),
                BasisPoints::new(row.economic_rights_bps).map_err(ContentError::Value)?,
                BasisPoints::new(row.voting_rights_bps).map_err(ContentError::Value)?,
            ))
        })
        .collect()
}

fn parse_firm_appointments(
    raw: Vec<RawFirmAppointment>,
) -> Result<Vec<FirmAppointment>, ContentError> {
    raw.into_iter()
        .map(|row| {
            Ok(FirmAppointment::new(
                FirmId::new(non_zero_id("appointment firm", row.firm_id)?),
                ActorId::new(non_zero_id("appointment actor", row.actor_id)?),
                row.role.into(),
            ))
        })
        .collect()
}

fn parse_firm_policies(
    raw: Vec<RawFirmPolicy>,
) -> Result<Vec<(ActorId, FirmId, FirmPolicy)>, ContentError> {
    raw.into_iter()
        .map(|row| {
            let policy = FirmPolicy::new(
                row.inventory_buffer_days,
                BasisPoints::new(row.price_markup_bps).map_err(ContentError::Value)?,
                BasisPoints::new(row.marketing_budget_bps).map_err(ContentError::Value)?,
                BasisPoints::new(row.reinvestment_bps).map_err(ContentError::Value)?,
                BasisPoints::new(row.dividend_bps).map_err(ContentError::Value)?,
            )
            .map_err(ContentError::Domain)?;
            Ok((
                ActorId::new(non_zero_id("policy actor", row.actor_id)?),
                FirmId::new(non_zero_id("policy firm", row.firm_id)?),
                policy,
            ))
        })
        .collect()
}

fn parse_production_targets(
    raw: Vec<RawProductionTarget>,
) -> Result<Vec<(ActorId, FirmId, u64)>, ContentError> {
    raw.into_iter()
        .map(|row| {
            Ok((
                ActorId::new(non_zero_id("target actor", row.actor_id)?),
                FirmId::new(non_zero_id("target firm", row.firm_id)?),
                row.batches,
            ))
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
    NegativeCountryStock {
        country: u32,
        field: &'static str,
        value: i64,
    },
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
            Self::NegativeCountryStock {
                country,
                field,
                value,
            } => {
                write!(formatter, "country {country} has negative {field}: {value}")
            }
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
            | Self::NegativeCountryStock { .. }
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
    goods: Vec<RawGood>,
    #[serde(default)]
    production_recipes: Vec<RawProductionRecipe>,
    #[serde(default)]
    firms: Vec<RawFirm>,
    #[serde(default)]
    employment_agreements: Vec<RawEmploymentAgreement>,
    #[serde(default)]
    ownership_stakes: Vec<RawOwnershipStake>,
    #[serde(default)]
    firm_appointments: Vec<RawFirmAppointment>,
    #[serde(default)]
    firm_policies: Vec<RawFirmPolicy>,
    #[serde(default)]
    production_targets: Vec<RawProductionTarget>,
    #[serde(default)]
    consumption_profiles: Vec<RawConsumptionProfile>,
    #[serde(default)]
    regional_prices: Vec<RawRegionalPrice>,
    #[serde(default)]
    regions: Vec<RawRegion>,
    #[serde(default)]
    cohorts: Vec<RawCohort>,
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
    treasury_minor: i64,
    public_debt_minor: i64,
    legitimacy_bps: u16,
    elite_cohesion_bps: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGood {
    id: u32,
    name: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawNeedTier {
    Survival,
    Participation,
    Development,
    Discretionary,
}
impl From<RawNeedTier> for NeedTier {
    fn from(value: RawNeedTier) -> Self {
        match value {
            RawNeedTier::Survival => Self::Survival,
            RawNeedTier::Participation => Self::Participation,
            RawNeedTier::Development => Self::Development,
            RawNeedTier::Discretionary => Self::Discretionary,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawDemandBasis {
    PerPerson,
    PerHousehold,
}
impl From<RawDemandBasis> for DemandBasis {
    fn from(value: RawDemandBasis) -> Self {
        match value {
            RawDemandBasis::PerPerson => Self::PerPerson,
            RawDemandBasis::PerHousehold => Self::PerHousehold,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConsumptionTarget {
    good_id: u32,
    tier: RawNeedTier,
    basis: RawDemandBasis,
    monthly_quantity_milli: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConsumptionProfile {
    id: u32,
    name: String,
    targets: Vec<RawConsumptionTarget>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRegionalPrice {
    region_id: u32,
    good_id: u32,
    price_minor: i64,
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

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawAgeBand {
    Child,
    Youth,
    Adult,
    Mature,
    Senior,
}
impl From<RawAgeBand> for AgeBand {
    fn from(value: RawAgeBand) -> Self {
        match value {
            RawAgeBand::Child => Self::Child,
            RawAgeBand::Youth => Self::Youth,
            RawAgeBand::Adult => Self::Adult,
            RawAgeBand::Mature => Self::Mature,
            RawAgeBand::Senior => Self::Senior,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawHouseholdType {
    FamilyWithChildren,
    WorkingAge,
    Multigenerational,
    Retired,
}
impl From<RawHouseholdType> for HouseholdType {
    fn from(value: RawHouseholdType) -> Self {
        match value {
            RawHouseholdType::FamilyWithChildren => Self::FamilyWithChildren,
            RawHouseholdType::WorkingAge => Self::WorkingAge,
            RawHouseholdType::Multigenerational => Self::Multigenerational,
            RawHouseholdType::Retired => Self::Retired,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawEducationLevel {
    None,
    Basic,
    Secondary,
    Vocational,
    Tertiary,
}
impl From<RawEducationLevel> for EducationLevel {
    fn from(value: RawEducationLevel) -> Self {
        match value {
            RawEducationLevel::None => Self::None,
            RawEducationLevel::Basic => Self::Basic,
            RawEducationLevel::Secondary => Self::Secondary,
            RawEducationLevel::Vocational => Self::Vocational,
            RawEducationLevel::Tertiary => Self::Tertiary,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawEmploymentStatus {
    Dependent,
    Employed,
    Unemployed,
    Inactive,
    Retired,
}
impl From<RawEmploymentStatus> for EmploymentStatus {
    fn from(value: RawEmploymentStatus) -> Self {
        match value {
            RawEmploymentStatus::Dependent => Self::Dependent,
            RawEmploymentStatus::Employed => Self::Employed,
            RawEmploymentStatus::Unemployed => Self::Unemployed,
            RawEmploymentStatus::Inactive => Self::Inactive,
            RawEmploymentStatus::Retired => Self::Retired,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCohort {
    id: u32,
    region_id: u32,
    need_profile_id: u32,
    people: u64,
    households: u64,
    age_band: RawAgeBand,
    household_type: RawHouseholdType,
    education: RawEducationLevel,
    employment: RawEmploymentStatus,
    annual_income_minor: i64,
    liquid_wealth_minor: i64,
    debt_minor: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProductionInput {
    good_id: u32,
    quantity_per_batch_milli: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProductionRecipe {
    id: u32,
    name: String,
    output_good_id: u32,
    output_per_batch_milli: u64,
    labor_milli_worker_months: u64,
    #[serde(default)]
    inputs: Vec<RawProductionInput>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFirmInventory {
    good_id: u32,
    quantity_milli: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFirm {
    id: u32,
    name: String,
    region_id: u32,
    recipe_id: u32,
    workers: u64,
    capacity_batches: u64,
    cash_minor: i64,
    #[serde(default)]
    inventories: Vec<RawFirmInventory>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEmploymentAgreement {
    firm_id: u32,
    cohort_id: u32,
    workers: u64,
    monthly_wage_per_worker_minor: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOwnershipStake {
    firm_id: u32,
    actor_id: u32,
    economic_rights_bps: u16,
    voting_rights_bps: u16,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RawCorporateRole {
    BoardDirector,
    ChiefExecutive,
    OperationsManager,
    MarketingManager,
}
impl From<RawCorporateRole> for CorporateRole {
    fn from(value: RawCorporateRole) -> Self {
        match value {
            RawCorporateRole::BoardDirector => Self::BoardDirector,
            RawCorporateRole::ChiefExecutive => Self::ChiefExecutive,
            RawCorporateRole::OperationsManager => Self::OperationsManager,
            RawCorporateRole::MarketingManager => Self::MarketingManager,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFirmAppointment {
    firm_id: u32,
    actor_id: u32,
    role: RawCorporateRole,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFirmPolicy {
    firm_id: u32,
    actor_id: u32,
    inventory_buffer_days: u16,
    price_markup_bps: u16,
    marketing_budget_bps: u16,
    reinvestment_bps: u16,
    dividend_bps: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProductionTarget {
    firm_id: u32,
    actor_id: u32,
    batches: u64,
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
schema_version = 5
world_name = "Test World"
start_year = 2025

[[countries]]
id = 1
name = "Aster"
treasury_minor = 1000000000
public_debt_minor = 500000000
legitimacy_bps = 6000
elite_cohesion_bps = 5500

[[goods]]
id = 1
name = "Staple food"

[[consumption_profiles]]
id = 1
name = "Test profile"
targets = [
  { good_id = 1, tier = "survival", basis = "per_person", monthly_quantity_milli = 1000 },
]

[[regional_prices]]
region_id = 10
good_id = 1
price_minor = 100

[[regions]]
id = 10
country_id = 1
name = "Aster Capital"
population = 1000000
annual_output_minor = 5000000000

[[cohorts]]
id = 10000
region_id = 10
need_profile_id = 1
people = 1000000
households = 400000
age_band = "adult"
household_type = "working_age"
education = "secondary"
employment = "employed"
annual_income_minor = 300000000000
liquid_wealth_minor = 100000000000
debt_minor = 50000000000

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
        assert_eq!(blueprint.cohorts().len(), 1);
        assert_eq!(blueprint.actors().len(), 1);
        assert_eq!(blueprint.power_nodes().len(), 1);
        assert_eq!(blueprint.influences().len(), 1);
    }

    #[test]
    fn production_content_builds_authorized_supply_chain() {
        let source = format!(
            "{}{}",
            VALID,
            r#"

[[production_recipes]]
id = 1
name = "Food production"
output_good_id = 1
output_per_batch_milli = 1000
labor_milli_worker_months = 1000

[[firms]]
id = 1
name = "Aster Foods"
region_id = 10
recipe_id = 1
workers = 1
capacity_batches = 1
cash_minor = 1000

[[employment_agreements]]
firm_id = 1
cohort_id = 10000
workers = 1
monthly_wage_per_worker_minor = 1

[[ownership_stakes]]
firm_id = 1
actor_id = 100
economic_rights_bps = 10000
voting_rights_bps = 10000

[[firm_appointments]]
firm_id = 1
actor_id = 100
role = "operations_manager"

[[firm_policies]]
firm_id = 1
actor_id = 100
inventory_buffer_days = 0
price_markup_bps = 0
marketing_budget_bps = 0
reinvestment_bps = 5000
dividend_bps = 0

[[production_targets]]
firm_id = 1
actor_id = 100
batches = 1
"#
        );
        let blueprint = WorldBlueprint::parse_toml(&source).expect("production content");
        let world = blueprint
            .build_world(WorldSeed::new(47))
            .expect("producing world");

        assert_eq!(world.production_recipes().len(), 1);
        assert_eq!(world.firms().len(), 1);
        assert_eq!(world.employment_agreements().len(), 1);
        assert_eq!(world.ownership_stakes().len(), 1);
        assert_eq!(world.firm_appointments().len(), 1);
        assert_eq!(world.firm_policies().len(), 1);
        assert_eq!(world.firm_production_targets()[&FirmId::new(1)], 1);
        assert_eq!(
            world.plan_monthly_production().expect("production plan")[0].batches(),
            1
        );
    }

    #[test]
    fn unsupported_schema_is_explicit() {
        let source = VALID.replace("schema_version = 5", "schema_version = 6");
        let error = WorldBlueprint::parse_toml(&source).expect_err("schema must fail");
        assert!(matches!(
            error,
            ContentError::UnsupportedSchema {
                expected: 5,
                actual: 6
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
