//! Versioned content loading and validation for A.D.A.M.
//!
//! The TOML content pipeline described by earlier roadmap entries has to be
//! rebuilt because its sources were never committed. Until then this crate
//! provides the deterministic embedded demo scenario used by the Stage 0
//! console chronicle runner.
//!
//! The demo deliberately tells two contrasting stories under the 20% sales
//! tax, which is the only monetary sink of the closed economy:
//! - Northreach runs a thin bakery cash buffer and is expected to decay
//!   mid-chronicle (wage arrears, deprivation, mortality);
//! - Southvale carries buffers and wages sized to survive the full 50 years.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, EducationLevel, EmploymentAgreement,
    EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId, HouseholdCohort,
    HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake, Population, ProductionInput,
    ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, World, WorldError,
    WorldSeed,
};

const GRAIN: u32 = 1;
const BREAD: u32 = 2;
const FARM_RECIPE: u32 = 1;
const BAKERY_RECIPE: u32 = 2;
const NEED_PROFILE: u32 = 1;
const COUNTRY: u32 = 1;

struct RegionSpec {
    region: u32,
    name: &'static str,
    owner_name: &'static str,
    farm_name: &'static str,
    bakery_name: &'static str,
    farm_wage: i64,
    bakery_wage: i64,
    farm_cash: i64,
    bakery_cash: i64,
    grain_price: i64,
    bread_price: i64,
}

fn region_specs() -> [RegionSpec; 2] {
    [
        RegionSpec {
            region: 1,
            name: "Northreach",
            owner_name: "Mara Voss",
            farm_name: "Northreach Grain Collective",
            bakery_name: "Northreach Bakery",
            farm_wage: 16,
            bakery_wage: 16,
            farm_cash: 200_000,
            bakery_cash: 400_000,
            grain_price: 5,
            bread_price: 10,
        },
        RegionSpec {
            region: 2,
            name: "Southvale",
            owner_name: "Ilya Roden",
            farm_name: "Southvale Grain Collective",
            bakery_name: "Southvale Bakery",
            farm_wage: 24,
            bakery_wage: 18,
            farm_cash: 400_000,
            bakery_cash: 1_500_000,
            grain_price: 6,
            bread_price: 12,
        },
    ]
}

/// Builds the embedded Stage 0 demo scenario.
///
/// # Errors
/// Returns [`WorldError`] if any demo entity fails registration.
///
/// # Panics
/// Panics only if an embedded demo constant is itself invalid, which is a
/// programming error in this crate.
pub fn demo_world(seed: u64) -> Result<World, WorldError> {
    let mut world = World::new(
        WorldSeed::new(seed),
        SimDate::new(2026, 1).expect("demo start date"),
    );
    register_catalog(&mut world)?;
    for spec in &region_specs() {
        register_region_economy(&mut world, spec)?;
    }
    Ok(world)
}

fn register_catalog(world: &mut World) -> Result<(), WorldError> {
    world.register_country(
        Country::new(CountryId::new(COUNTRY), "Arcadia").expect("demo country"),
    )?;
    world.register_good(Good::new(GoodId::new(GRAIN), "Grain").expect("demo grain"))?;
    world.register_good(Good::new(GoodId::new(BREAD), "Bread").expect("demo bread"))?;
    world.register_consumption_profile(
        ConsumptionProfile::new(
            NeedProfileId::new(NEED_PROFILE),
            "Households",
            vec![ConsumptionTarget::new(
                GoodId::new(BREAD),
                NeedTier::Survival,
                DemandBasis::PerPerson,
                QuantityMilli::new(1_000),
            )],
        )
        .expect("demo consumption profile"),
    )?;
    world.register_production_recipe(
        ProductionRecipe::new(
            RecipeId::new(FARM_RECIPE),
            "Grain farming",
            GoodId::new(GRAIN),
            QuantityMilli::new(4_000),
            1_000,
            vec![],
        )
        .expect("demo farm recipe"),
    )?;
    world.register_production_recipe(
        ProductionRecipe::new(
            RecipeId::new(BAKERY_RECIPE),
            "Bread baking",
            GoodId::new(BREAD),
            QuantityMilli::new(2_000),
            1_000,
            vec![ProductionInput::new(
                GoodId::new(GRAIN),
                QuantityMilli::new(1_000),
            )],
        )
        .expect("demo bakery recipe"),
    )?;
    Ok(())
}

fn register_region_economy(world: &mut World, spec: &RegionSpec) -> Result<(), WorldError> {
    let region = RegionId::new(spec.region);
    world.register_region(
        Region::new(
            region,
            CountryId::new(COUNTRY),
            spec.name,
            Population::new(1_000),
            Money::from_minor_units(1),
        )
        .expect("demo region"),
    )?;
    world.register_actor(
        Actor::new(ActorId::new(spec.region), spec.owner_name, region, 1980).expect("demo actor"),
    )?;
    world.register_household_cohort(
        HouseholdCohort::new(
            CohortId::new(spec.region),
            region,
            NeedProfileId::new(NEED_PROFILE),
            Population::new(1_000),
            400,
            AgeBand::Adult,
            HouseholdType::WorkingAge,
            EducationLevel::Secondary,
            EmploymentStatus::Employed,
            Money::default(),
            Money::from_minor_units(20_000),
            Money::default(),
        )
        .expect("demo cohort"),
    )?;
    world.set_regional_price(
        region,
        GoodId::new(GRAIN),
        Money::from_minor_units(spec.grain_price),
    )?;
    world.set_regional_price(
        region,
        GoodId::new(BREAD),
        Money::from_minor_units(spec.bread_price),
    )?;
    register_region_firms(world, spec)
}

fn register_region_firms(world: &mut World, spec: &RegionSpec) -> Result<(), WorldError> {
    let region = RegionId::new(spec.region);
    let owner = ActorId::new(spec.region);
    let farm = FirmId::new(spec.region * 10 + 1);
    let bakery = FirmId::new(spec.region * 10 + 2);
    world.register_firm(
        Firm::new(
            farm,
            spec.farm_name,
            region,
            RecipeId::new(FARM_RECIPE),
            125,
            125,
            Money::from_minor_units(spec.farm_cash),
            BTreeMap::new(),
        )
        .expect("demo farm"),
    )?;
    world.register_firm(
        Firm::new(
            bakery,
            spec.bakery_name,
            region,
            RecipeId::new(BAKERY_RECIPE),
            500,
            500,
            Money::from_minor_units(spec.bakery_cash),
            BTreeMap::from([
                (GoodId::new(GRAIN), QuantityMilli::new(500_000)),
                (GoodId::new(BREAD), QuantityMilli::new(1_000_000)),
            ]),
        )
        .expect("demo bakery"),
    )?;
    for (firm, workers, target, wage) in [
        (farm, 125_u64, 125_u64, spec.farm_wage),
        (bakery, 500, 500, spec.bakery_wage),
    ] {
        world.register_ownership_stake(OwnershipStake::new(
            firm,
            owner,
            BasisPoints::new(6_000).expect("demo ownership"),
            BasisPoints::new(6_000).expect("demo ownership"),
        ))?;
        world.register_firm_appointment(FirmAppointment::new(
            firm,
            owner,
            CorporateRole::OperationsManager,
        ))?;
        world.set_firm_policy(
            owner,
            firm,
            FirmPolicy::new(
                0,
                BasisPoints::new(0).expect("demo markup"),
                BasisPoints::new(0).expect("demo allocation"),
                BasisPoints::new(0).expect("demo allocation"),
                BasisPoints::new(0).expect("demo allocation"),
            )
            .expect("demo policy"),
        )?;
        world.set_firm_production_target(owner, firm, target)?;
        world.register_employment_agreement(
            EmploymentAgreement::new(
                firm,
                CohortId::new(spec.region),
                workers,
                Money::from_minor_units(wage),
            )
            .expect("demo agreement"),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_world_builds_and_advances_one_economic_year() {
        let mut world = demo_world(1).expect("demo world");
        world.advance_economic_year().expect("first economic year");
    }

    #[test]
    fn demo_world_is_deterministic_for_equal_seeds() {
        let mut a = demo_world(7).expect("demo world");
        let mut b = demo_world(7).expect("demo world");
        a.advance_economic_years(2).expect("two years");
        b.advance_economic_years(2).expect("two years");
        assert_eq!(a.stable_fingerprint(), b.stable_fingerprint());
    }
}
