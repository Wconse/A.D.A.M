//! Versioned content loading and validation for A.D.A.M.
//!
//! Content schema v8: a scenario is a TOML document (see `assets/demo.toml`)
//! loaded into a [`World`] through [`world_from_toml_str`]. The embedded
//! Stage 0 demo scenario keeps telling two contrasting stories under the 20%
//! final-sales tax, the only monetary sink of the closed economy:
//! - Northreach runs a thin bakery cash buffer and is expected to decay
//!   mid-chronicle (wage arrears, deprivation, mortality);
//! - Southvale carries buffers and wages sized to survive the full 50 years.
//!
//! Schema v8 replaces the single country with a countries list: the demo also
//! runs Borealia, a second country with its own region, so a single run
//! narrates more than one fiscal and demographic path.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, EducationLevel, EmploymentAgreement,
    EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId, HouseholdCohort,
    HouseholdType, LogisticsRoute, Money, NeedProfileId, NeedTier, OwnershipStake, Population,
    ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, RouteId, SimDate,
    TransportMode, World, WorldError, WorldSeed,
};

/// Content schema version understood by this crate.
pub const SCHEMA_VERSION: u32 = 8;

/// The embedded Stage 0 demo scenario in content schema v8.
pub const DEMO_SCENARIO_TOML: &str = include_str!("../assets/demo.toml");

/// Errors raised while loading scenario content.
#[derive(Debug)]
pub enum ContentError {
    /// The TOML document could not be parsed.
    Parse(toml::de::Error),
    /// The document parsed but violates the content schema.
    Schema(String),
    /// The world rejected an entity during registration.
    World(WorldError),
}

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "content parse error: {error}"),
            Self::Schema(message) => write!(f, "content schema error: {message}"),
            Self::World(error) => write!(f, "world rejected content: {error:?}"),
        }
    }
}

impl std::error::Error for ContentError {}

impl From<WorldError> for ContentError {
    fn from(error: WorldError) -> Self {
        Self::World(error)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioSpec {
    schema_version: u32,
    scenario: ScenarioMeta,
    countries: Vec<CountrySpec>,
    goods: Vec<GoodSpec>,
    consumption_profiles: Vec<ProfileSpec>,
    recipes: Vec<RecipeSpec>,
    regions: Vec<RegionSpec>,
    #[serde(default)]
    routes: Vec<RouteSpec>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioMeta {
    start_year: i32,
    start_day: u16,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CountrySpec {
    id: u32,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GoodSpec {
    id: u32,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileSpec {
    id: u32,
    name: String,
    targets: Vec<TargetSpec>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetSpec {
    good: u32,
    tier: String,
    basis: String,
    quantity_milli: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeSpec {
    id: u32,
    name: String,
    output_good: u32,
    output_per_batch_milli: u64,
    labor_milli: u64,
    /// Opts this recipe into competitive vacancy matching. Omission preserves
    /// legacy staffing behavior for existing scenarios and mods.
    minimum_education: Option<String>,
    #[serde(default)]
    inputs: Vec<RecipeInputSpec>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeInputSpec {
    good: u32,
    quantity_per_batch_milli: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionSpec {
    id: u32,
    name: String,
    population: u64,
    initial_annual_output: i64,
    country: u32,
    owner: OwnerSpec,
    #[serde(default)]
    financiers: Vec<FinancierSpec>,
    cohort: CohortSpec,
    prices: Vec<PriceSpec>,
    firms: Vec<FirmSpec>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteSpec {
    id: u32,
    origin: u32,
    destination: u32,
    mode: String,
    monthly_capacity_milli: u64,
    cost_per_unit_minor: i64,
    transit_days: u16,
    reliability_bps: u16,
    carrier: u32,
}
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnerSpec {
    id: u32,
    name: String,
    birth_year: i32,
    liquid_cash: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FinancierSpec {
    id: u32,
    name: String,
    birth_year: i32,
    liquid_cash: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CohortSpec {
    id: u32,
    need_profile: u32,
    people: u64,
    households: u64,
    age_band: String,
    household_type: String,
    education: String,
    employment: String,
    annual_income: i64,
    liquid_wealth: i64,
    debt: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PriceSpec {
    good: u32,
    minor_units: i64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FirmSpec {
    id: u32,
    name: String,
    recipe: u32,
    workers: u64,
    capacity_batches: u64,
    production_target: u64,
    cash: i64,
    wage: i64,
    #[serde(default)]
    inventory: Vec<InventorySpec>,
    governance: GovernanceSpec,
    policy: PolicySpec,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct InventorySpec {
    good: u32,
    quantity_milli: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GovernanceSpec {
    owner_voting_bp: u16,
    owner_economic_bp: u16,
    manager_role: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicySpec {
    inventory_buffer_days: u16,
    price_markup_bp: u16,
    allocations_bp: [u16; 3],
}

/// Builds the embedded Stage 0 demo scenario.
///
/// # Errors
/// Returns [`WorldError`] if the world rejects a demo entity.
///
/// # Panics
/// Panics only if the embedded demo document itself is invalid, which is a
/// programming error in this crate.
pub fn demo_world(seed: u64) -> Result<World, WorldError> {
    match world_from_toml_str(seed, DEMO_SCENARIO_TOML) {
        Ok(world) => Ok(world),
        Err(ContentError::World(error)) => Err(error),
        Err(error) => panic!("embedded demo scenario is invalid: {error}"),
    }
}

/// Builds a [`World`] from a content schema v8 TOML scenario document.
///
/// # Errors
/// Returns [`ContentError`] when the document cannot be parsed, violates the
/// content schema, or is rejected during world registration.
pub fn world_from_toml_str(seed: u64, document: &str) -> Result<World, ContentError> {
    let spec: ScenarioSpec = toml::from_str(document).map_err(ContentError::Parse)?;
    if spec.schema_version != SCHEMA_VERSION {
        let version = spec.schema_version;
        return Err(ContentError::Schema(format!(
            "unsupported schema_version {version} (expected {SCHEMA_VERSION})"
        )));
    }
    let start = SimDate::new(spec.scenario.start_year, spec.scenario.start_day)
        .map_err(|error| ContentError::Schema(format!("invalid start date: {error:?}")))?;
    let mut world = World::new(WorldSeed::new(seed), start);
    register_catalog(&mut world, &spec)?;

    for region in &spec.regions {
        register_region_economy(&mut world, CountryId::new(region.country), region)?;
    }
    for route in &spec.routes {
        world.register_logistics_route(
            LogisticsRoute::new(
                RouteId::new(route.id),
                RegionId::new(route.origin),
                RegionId::new(route.destination),
                parse_transport_mode(&route.mode)?,
                QuantityMilli::new(route.monthly_capacity_milli),
                Money::from_minor_units(route.cost_per_unit_minor),
                route.transit_days,
                route.reliability_bps,
            )
            .map_err(|error| ContentError::Schema(format!("invalid logistics route: {error:?}")))?
            .with_carrier(FirmId::new(route.carrier)),
        )?;
    }
    Ok(world)
}

fn register_catalog(world: &mut World, spec: &ScenarioSpec) -> Result<(), ContentError> {
    for country in &spec.countries {
        world.register_country(
            Country::new(CountryId::new(country.id), &country.name)
                .map_err(|error| ContentError::Schema(format!("invalid country: {error:?}")))?,
        )?;
    }
    for good in &spec.goods {
        world.register_good(
            Good::new(GoodId::new(good.id), &good.name)
                .map_err(|error| ContentError::Schema(format!("invalid good: {error:?}")))?,
        )?;
    }
    for profile in &spec.consumption_profiles {
        let mut targets = Vec::new();
        for target in &profile.targets {
            targets.push(ConsumptionTarget::new(
                GoodId::new(target.good),
                parse_need_tier(&target.tier)?,
                parse_demand_basis(&target.basis)?,
                QuantityMilli::new(target.quantity_milli),
            ));
        }
        world.register_consumption_profile(
            ConsumptionProfile::new(NeedProfileId::new(profile.id), &profile.name, targets)
                .map_err(|error| {
                    ContentError::Schema(format!("invalid consumption profile: {error:?}"))
                })?,
        )?;
    }
    for recipe in &spec.recipes {
        let inputs = recipe
            .inputs
            .iter()
            .map(|input| {
                ProductionInput::new(
                    GoodId::new(input.good),
                    QuantityMilli::new(input.quantity_per_batch_milli),
                )
            })
            .collect();
        let recipe_id = RecipeId::new(recipe.id);
        world.register_production_recipe(
            ProductionRecipe::new(
                recipe_id,
                &recipe.name,
                GoodId::new(recipe.output_good),
                QuantityMilli::new(recipe.output_per_batch_milli),
                recipe.labor_milli,
                inputs,
            )
            .map_err(|error| ContentError::Schema(format!("invalid recipe: {error:?}")))?,
        )?;
        if let Some(minimum_education) = recipe.minimum_education.as_deref() {
            world.set_recipe_minimum_education(recipe_id, parse_education(minimum_education)?)?;
        }
    }
    Ok(())
}

fn register_region_economy(
    world: &mut World,
    country: CountryId,
    spec: &RegionSpec,
) -> Result<(), ContentError> {
    let region = RegionId::new(spec.id);
    world.register_region(
        Region::new(
            region,
            country,
            &spec.name,
            Population::new(spec.population),
            Money::from_minor_units(spec.initial_annual_output),
        )
        .map_err(|error| ContentError::Schema(format!("invalid region: {error:?}")))?,
    )?;
    let owner = ActorId::new(spec.owner.id);
    world.register_actor(
        Actor::new(owner, &spec.owner.name, region, spec.owner.birth_year)
            .map_err(|error| ContentError::Schema(format!("invalid actor: {error:?}")))?,
    )?;
    world.register_actor_cash(owner, Money::from_minor_units(spec.owner.liquid_cash))?;
    for financier in &spec.financiers {
        let actor = ActorId::new(financier.id);
        world.register_actor(
            Actor::new(actor, &financier.name, region, financier.birth_year)
                .map_err(|error| ContentError::Schema(format!("invalid financier: {error:?}")))?,
        )?;
        world.register_actor_cash(actor, Money::from_minor_units(financier.liquid_cash))?;
    }
    world.register_household_cohort(
        HouseholdCohort::new(
            CohortId::new(spec.cohort.id),
            region,
            NeedProfileId::new(spec.cohort.need_profile),
            Population::new(spec.cohort.people),
            spec.cohort.households,
            parse_age_band(&spec.cohort.age_band)?,
            parse_household_type(&spec.cohort.household_type)?,
            parse_education(&spec.cohort.education)?,
            parse_employment(&spec.cohort.employment)?,
            Money::from_minor_units(spec.cohort.annual_income),
            Money::from_minor_units(spec.cohort.liquid_wealth),
            Money::from_minor_units(spec.cohort.debt),
        )
        .map_err(|error| ContentError::Schema(format!("invalid cohort: {error:?}")))?,
    )?;
    for price in &spec.prices {
        world.set_regional_price(
            region,
            GoodId::new(price.good),
            Money::from_minor_units(price.minor_units),
        )?;
    }
    for firm in &spec.firms {
        register_firm_definition(world, region, firm)?;
    }
    for firm in &spec.firms {
        register_firm_governance(world, spec, firm)?;
    }
    Ok(())
}

fn register_firm_definition(
    world: &mut World,
    region: RegionId,
    spec: &FirmSpec,
) -> Result<(), ContentError> {
    let mut inventories = BTreeMap::new();
    for row in &spec.inventory {
        inventories.insert(
            GoodId::new(row.good),
            QuantityMilli::new(row.quantity_milli),
        );
    }
    world.register_firm(
        Firm::new(
            FirmId::new(spec.id),
            &spec.name,
            region,
            RecipeId::new(spec.recipe),
            spec.workers,
            spec.capacity_batches,
            Money::from_minor_units(spec.cash),
            inventories,
        )
        .map_err(|error| ContentError::Schema(format!("invalid firm: {error:?}")))?,
    )?;
    Ok(())
}

fn register_firm_governance(
    world: &mut World,
    region: &RegionSpec,
    spec: &FirmSpec,
) -> Result<(), ContentError> {
    let firm = FirmId::new(spec.id);
    let owner = ActorId::new(region.owner.id);
    world.register_ownership_stake(OwnershipStake::new(
        firm,
        owner,
        parse_basis_points(spec.governance.owner_voting_bp)?,
        parse_basis_points(spec.governance.owner_economic_bp)?,
    ))?;
    world.register_firm_appointment(FirmAppointment::new(
        firm,
        owner,
        parse_role(&spec.governance.manager_role)?,
    ))?;
    world.set_firm_policy(
        owner,
        firm,
        FirmPolicy::new(
            spec.policy.inventory_buffer_days,
            parse_basis_points(spec.policy.price_markup_bp)?,
            parse_basis_points(spec.policy.allocations_bp[0])?,
            parse_basis_points(spec.policy.allocations_bp[1])?,
            parse_basis_points(spec.policy.allocations_bp[2])?,
        )
        .map_err(|error| ContentError::Schema(format!("invalid firm policy: {error:?}")))?,
    )?;
    world.set_firm_production_target(owner, firm, spec.production_target)?;
    world.register_employment_agreement(
        EmploymentAgreement::new(
            firm,
            CohortId::new(region.cohort.id),
            spec.workers,
            Money::from_minor_units(spec.wage),
        )
        .map_err(|error| {
            ContentError::Schema(format!("invalid employment agreement: {error:?}"))
        })?,
    )?;
    Ok(())
}

fn parse_basis_points(value: u16) -> Result<BasisPoints, ContentError> {
    BasisPoints::new(value)
        .map_err(|error| ContentError::Schema(format!("invalid basis points: {error:?}")))
}

fn unsupported(kind: &str, value: &str) -> ContentError {
    ContentError::Schema(format!("unsupported {kind} `{value}` in schema v8"))
}

fn parse_need_tier(value: &str) -> Result<NeedTier, ContentError> {
    match value {
        "survival" => Ok(NeedTier::Survival),
        other => Err(unsupported("need tier", other)),
    }
}

fn parse_demand_basis(value: &str) -> Result<DemandBasis, ContentError> {
    match value {
        "per_person" => Ok(DemandBasis::PerPerson),
        other => Err(unsupported("demand basis", other)),
    }
}

fn parse_age_band(value: &str) -> Result<AgeBand, ContentError> {
    match value {
        "adult" => Ok(AgeBand::Adult),
        other => Err(unsupported("age band", other)),
    }
}

fn parse_household_type(value: &str) -> Result<HouseholdType, ContentError> {
    match value {
        "working_age" => Ok(HouseholdType::WorkingAge),
        other => Err(unsupported("household type", other)),
    }
}

fn parse_education(value: &str) -> Result<EducationLevel, ContentError> {
    match value {
        "none" => Ok(EducationLevel::None),
        "basic" => Ok(EducationLevel::Basic),
        "secondary" => Ok(EducationLevel::Secondary),
        "vocational" => Ok(EducationLevel::Vocational),
        "tertiary" => Ok(EducationLevel::Tertiary),
        other => Err(unsupported("education level", other)),
    }
}

fn parse_employment(value: &str) -> Result<EmploymentStatus, ContentError> {
    match value {
        "employed" => Ok(EmploymentStatus::Employed),
        other => Err(unsupported("employment status", other)),
    }
}

fn parse_role(value: &str) -> Result<CorporateRole, ContentError> {
    match value {
        "operations_manager" => Ok(CorporateRole::OperationsManager),
        other => Err(unsupported("corporate role", other)),
    }
}

fn parse_transport_mode(value: &str) -> Result<TransportMode, ContentError> {
    match value {
        "road" => Ok(TransportMode::Road),
        "rail" => Ok(TransportMode::Rail),
        "sea" => Ok(TransportMode::Sea),
        "air" => Ok(TransportMode::Air),
        other => Err(unsupported("transport mode", other)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_world_builds_and_advances_one_economic_year() {
        let mut world = demo_world(1).expect("demo world");
        world.advance_economic_year().expect("first economic year");
        assert_eq!(world.logistics_routes().len(), 1);
    }

    #[test]
    fn demo_world_is_deterministic_for_equal_seeds() {
        let mut a = demo_world(7).expect("demo world");
        let mut b = demo_world(7).expect("demo world");
        a.advance_economic_years(2).expect("two years");
        b.advance_economic_years(2).expect("two years");
        assert_eq!(a.stable_fingerprint(), b.stable_fingerprint());
    }

    #[test]
    fn broken_document_is_a_parse_error() {
        assert!(matches!(
            world_from_toml_str(1, "not = ["),
            Err(ContentError::Parse(_))
        ));
    }

    #[test]
    fn wrong_schema_version_is_rejected() {
        let document = DEMO_SCENARIO_TOML.replace("schema_version = 8", "schema_version = 4");
        assert!(matches!(
            world_from_toml_str(1, &document),
            Err(ContentError::Schema(_))
        ));
    }

    #[test]
    fn unsupported_enum_values_are_rejected() {
        let document = DEMO_SCENARIO_TOML.replace("tier = \"survival\"", "tier = \"luxury\"");
        assert!(matches!(
            world_from_toml_str(1, &document),
            Err(ContentError::Schema(_))
        ));
    }

    #[test]
    fn recipe_labor_profile_is_explicit_and_legacy_recipes_remain_opted_out() {
        let legacy = demo_world(1).expect("legacy-compatible demo world");
        assert!(legacy.recipe_minimum_education().is_empty());

        let document = DEMO_SCENARIO_TOML.replacen(
            "labor_milli = 1000",
            "labor_milli = 1000\nminimum_education = \"vocational\"",
            1,
        );
        let configured = world_from_toml_str(1, &document).expect("configured labor profile");
        assert_eq!(
            configured.recipe_minimum_education()[&RecipeId::new(1)],
            EducationLevel::Vocational
        );
    }

    #[test]
    fn demo_world_narrates_two_countries() {
        let mut world = demo_world(1).expect("demo world");
        world.advance_economic_year().expect("first economic year");
        assert!(
            world
                .chronicle()
                .iter()
                .any(|entry| entry.text.contains("across 2 countries")),
            "chronicle should report fiscal closure across 2 countries"
        );
    }
    #[test]
    fn demo_credit_market_rescues_one_case_and_refuses_concentrated_second_case() {
        let mut world = demo_world(1).expect("demo world");
        let mut decisions = Vec::new();
        for _ in 0..3 {
            decisions = world
                .execute_monthly_economic_cycle()
                .expect("controlled credit month")
                .credit_decisions;
        }
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].firm, FirmId::new(13));
        assert_eq!(decisions[0].creditor, ActorId::new(4));
        assert_eq!(decisions[0].funding_gap, Money::from_minor_units(4));
        assert_eq!(decisions[0].principal, Money::from_minor_units(6));
        assert!(
            world
                .firm_creditor_claims()
                .keys()
                .any(|(firm, _, creditor)| {
                    *firm == FirmId::new(13) && *creditor == ActorId::new(4)
                })
        );
        assert!(
            !world
                .firm_creditor_claims()
                .keys()
                .any(|(firm, _, _)| *firm == FirmId::new(14))
        );
        assert!(world.firm_credit_offers().is_empty());
        assert_eq!(
            world.actor_cash()[&ActorId::new(4)],
            Money::from_minor_units(9)
        );

        for _ in 3..12 {
            world
                .execute_monthly_economic_cycle()
                .expect("remaining credit year");
        }
        assert!(
            !world
                .firm_creditor_claims()
                .keys()
                .any(|(firm, _, _)| *firm == FirmId::new(13))
        );
        let history = world.lender_credit_history()[&ActorId::new(4)];
        assert_eq!(history.principal_repaid(), Money::from_minor_units(6));
        assert_eq!(history.interest_income(), Money::from_minor_units(6));
        assert_eq!(history.realized_losses(), Money::default());
        assert_eq!(history.successful_loans(), 1);
        assert_eq!(history.defaulted_loans(), 0);
        let rescued = &world.employment_agreements()[&(FirmId::new(13), CohortId::new(1))];
        assert_eq!(rescued.workers(), 1);
        assert_eq!(rescued.arrears(), Money::default());
        let chronicle = world.chronicle();
        assert!(chronicle.iter().any(|entry| {
            entry
                .text
                .contains("accepted 1 observed credit offers providing 6 minor currency units")
                && entry.text.contains(
                    "1 of 2 viable monthly firm funding searches ended without an acceptable domestic credit offer",
                )
                && entry.text.contains(
                    "Private lenders recovered 6 principal and earned 6 interest; 1 loan completed successfully",
                )
        }));
    }
}
