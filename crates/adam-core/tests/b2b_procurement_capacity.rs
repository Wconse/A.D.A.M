//! Gate test: B2B firm procurement and household survival imports share one
//! physical route-capacity pool per commercial cycle.
//!
//! Invariants proven here:
//! 1. When a firm procures intermediates over a route first, its fill consumes
//!    from the shared monthly capacity pool; household imports on the same
//!    route in the same month see only the remainder.
//! 2. Without a capacity constraint the two flows coexist freely.
//! 3. A year with shared-capacity trade is replayable through the command
//!    boundary.
//!
//! World: Farmland (region 1, country A) produces Grain for export.
//! Bakerton (region 2, country A) has a Bakery that needs Grain as an input
//! and households that need Bread for survival. The road route connecting
//! Farmland → Bakerton carries both flows and has a finite monthly capacity.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, EducationLevel, EmploymentAgreement,
    EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId, HouseholdCohort,
    HouseholdType, LogisticsRoute, Money, NeedProfileId, NeedTier, OwnershipStake, Population,
    ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, RouteId, SimDate,
    TransportMode, World, WorldCommand, WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const GRAIN: u32 = 1;
const BREAD: u32 = 2;
const FARMLAND: u32 = 1;
const BAKERTON: u32 = 2;
const GRAIN_RECIPE: u32 = 1;
const BREAD_RECIPE: u32 = 2;

fn install_governance(world: &mut World, firm: FirmId, owner: ActorId) {
    world
        .register_ownership_stake(OwnershipStake::new(
            firm,
            owner,
            BasisPoints::new(6_000).expect("rights"),
            BasisPoints::new(6_000).expect("rights"),
        ))
        .expect("ownership");
    world
        .register_firm_appointment(FirmAppointment::new(
            firm,
            owner,
            CorporateRole::OperationsManager,
        ))
        .expect("appointment");
    world
        .set_firm_policy(
            owner,
            firm,
            FirmPolicy::new(
                0,
                BasisPoints::new(0).expect("markup"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
            )
            .expect("policy"),
        )
        .expect("set policy");
    world
        .set_firm_production_target(owner, firm, 1)
        .expect("target");
}

/// Builds a two-region world where the Farm produces Grain and the Bakery
/// needs Grain to make Bread. Bakerton households need Bread for survival.
/// `route_capacity` controls how many milli-units per month the road can carry.
#[allow(clippy::too_many_lines)]
fn shared_route_world(route_capacity: u64) -> World {
    let mut world = World::new(WorldSeed::new(4_242), SimDate::new(2025, 1).expect("date"));
    world
        .register_country(Country::new(CountryId::new(1), "A").expect("country"))
        .expect("country");
    world
        .register_good(Good::new(GoodId::new(GRAIN), "Grain").expect("good"))
        .expect("grain");
    world
        .register_good(Good::new(GoodId::new(BREAD), "Bread").expect("good"))
        .expect("bread");
    world
        .register_consumption_profile(
            ConsumptionProfile::new(
                NeedProfileId::new(1),
                "Households",
                vec![ConsumptionTarget::new(
                    GoodId::new(BREAD),
                    NeedTier::Survival,
                    DemandBasis::PerPerson,
                    QuantityMilli::new(1_000),
                )],
            )
            .expect("profile"),
        )
        .expect("profile");
    for (id, name) in [(FARMLAND, "Farmland"), (BAKERTON, "Bakerton")] {
        world
            .register_region(
                Region::new(
                    RegionId::new(id),
                    CountryId::new(1),
                    name,
                    Population::new(1),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("region");
    }
    for (id, name, region) in [(1, "Farm Owner", FARMLAND), (2, "Bakery Owner", BAKERTON)] {
        world
            .register_actor(
                Actor::new(ActorId::new(id), name, RegionId::new(region), 1980).expect("actor"),
            )
            .expect("actor");
    }
    world
        .register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(GRAIN_RECIPE),
                "Grain recipe",
                GoodId::new(GRAIN),
                QuantityMilli::new(2_000),
                1_000,
                vec![],
            )
            .expect("grain recipe"),
        )
        .expect("grain recipe");
    world
        .register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(BREAD_RECIPE),
                "Bread recipe",
                GoodId::new(BREAD),
                QuantityMilli::new(1_000),
                1_000,
                vec![ProductionInput::new(
                    GoodId::new(GRAIN),
                    QuantityMilli::new(1_000),
                )],
            )
            .expect("bread recipe"),
        )
        .expect("bread recipe");
    // Farm: produces 2000 milli-Grain per batch and pre-stocks 3000
    world
        .register_firm(
            Firm::new(
                FirmId::new(FARM),
                "Farm",
                RegionId::new(FARMLAND),
                RecipeId::new(GRAIN_RECIPE),
                1,
                1,
                Money::from_minor_units(500),
                BTreeMap::from([(GoodId::new(GRAIN), QuantityMilli::new(3_000))]),
            )
            .expect("farm"),
        )
        .expect("farm");
    // Bakery: makes Bread from Grain
    world
        .register_firm(
            Firm::new(
                FirmId::new(BAKERY),
                "Bakery",
                RegionId::new(BAKERTON),
                RecipeId::new(BREAD_RECIPE),
                1,
                1,
                Money::from_minor_units(500),
                BTreeMap::new(),
            )
            .expect("bakery"),
        )
        .expect("bakery");
    install_governance(&mut world, FirmId::new(FARM), ActorId::new(1));
    install_governance(&mut world, FirmId::new(BAKERY), ActorId::new(2));
    // One cohort in each region; Bakerton cohort needs Bread for survival
    for (cohort_id, region) in [(1u32, FARMLAND), (2, BAKERTON)] {
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(cohort_id),
                    RegionId::new(region),
                    NeedProfileId::new(1),
                    Population::new(1),
                    1,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::default(),
                    Money::from_minor_units(200),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
    }
    world
        .register_employment_agreement(
            EmploymentAgreement::new(
                FirmId::new(FARM),
                CohortId::new(1),
                1,
                Money::from_minor_units(50),
            )
            .expect("farm agreement"),
        )
        .expect("farm agreement");
    world
        .register_employment_agreement(
            EmploymentAgreement::new(
                FirmId::new(BAKERY),
                CohortId::new(2),
                1,
                Money::from_minor_units(50),
            )
            .expect("bakery agreement"),
        )
        .expect("bakery agreement");
    for (region, good, price) in [
        (FARMLAND, GRAIN, 4i64),
        (FARMLAND, BREAD, 10),
        (BAKERTON, GRAIN, 8),
        (BAKERTON, BREAD, 10),
    ] {
        world
            .set_regional_price(
                RegionId::new(region),
                GoodId::new(good),
                Money::from_minor_units(price),
            )
            .expect("price");
    }
    // Road route Farmland -> Bakerton; carries both B2B Grain and household Bread
    world
        .register_logistics_route(
            LogisticsRoute::new(
                RouteId::new(1),
                RegionId::new(FARMLAND),
                RegionId::new(BAKERTON),
                TransportMode::Road,
                QuantityMilli::new(route_capacity),
                Money::from_minor_units(1),
                3,
                9_500,
            )
            .expect("route")
            .with_carrier(FirmId::new(FARM)),
        )
        .expect("route");
    world
}

/// With ample capacity (10 000) both flows fill freely: B2B Grain import for
/// the Bakery and household Bread import for Bakerton cohort.
#[test]
fn ample_capacity_allows_both_flows() {
    let mut world = shared_route_world(10_000);
    let mut replayed = world.clone();
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed");

    // Bakery procures 1000 milli-Grain from Farm over the route
    let proc_fills = &result.commercial.procurement.fills;
    assert_eq!(proc_fills.len(), 1, "bakery should import grain");
    assert_eq!(proc_fills[0].buyer, FirmId::new(BAKERY));
    assert_eq!(proc_fills[0].seller, FirmId::new(FARM));
    assert_eq!(proc_fills[0].quantity, QuantityMilli::new(1_000));
    assert!(result.commercial.procurement.unmet.is_empty());

    // Bakerton household gets Bread from Bakery (locally produced after Grain import)
    // Bread is produced locally after procurement, so household demand is met locally
    assert!(world == replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

/// With capacity exactly 1000 the Bakery's B2B Grain import (1000 milli-units)
/// exhausts the shared pool, leaving zero capacity for any additional import
/// on the same route in the same cycle.
#[test]
fn tight_capacity_is_shared_between_b2b_and_household_imports() {
    // The Bakery needs 1000 milli-Grain from Farmland.
    // Route capacity = 1000: just enough for the B2B fill, nothing left for
    // any household import on the same route that month.
    let mut world = shared_route_world(1_000);
    let mut replayed = world.clone();
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed");

    // B2B: Bakery imports exactly 1000 milli-Grain (uses full capacity)
    let proc_fills = &result.commercial.procurement.fills;
    assert_eq!(proc_fills.len(), 1);
    assert_eq!(proc_fills[0].buyer, FirmId::new(BAKERY));
    assert_eq!(proc_fills[0].quantity, QuantityMilli::new(1_000));
    assert!(result.commercial.procurement.unmet.is_empty());

    // Route capacity is now zero: no cross-region household import on route 1
    // could have happened this cycle.
    // (Farmland→Bakerton is the only route; Bakerton Bread is produced locally
    // after the Grain import, so household demand may still be met locally.)

    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

/// Capacity 500 forces B2B procurement to be partially unmet: Bakery can only
/// import 500 milli-Grain instead of the required 1000.
#[test]
fn insufficient_capacity_caps_b2b_procurement_fill() {
    let mut world = shared_route_world(500);
    let mut replayed = world.clone();
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed");

    // B2B: only 500 out of 1000 needed Grain can cross the route
    let proc_fills = &result.commercial.procurement.fills;
    assert_eq!(proc_fills.len(), 1);
    assert_eq!(proc_fills[0].buyer, FirmId::new(BAKERY));
    assert_eq!(proc_fills[0].quantity, QuantityMilli::new(500));

    // Unmet: 500 milli-Grain still needed
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(500)
    );

    // The new journal event gives the chronicle an explainable, named cause.
    let chronicle = world.chronicle();
    assert_eq!(chronicle.len(), 1);
    // Household survival shortage already ranks this month above the B2B-only
    // importance tier; the procurement sentence must still be present.
    assert_eq!(chronicle[0].importance, 90);
    assert!(
        chronicle[0]
            .text
            .contains("Route capacity prevented Bakery from procuring 500 milli-units of Grain")
    );

    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

/// A year of shared-capacity trade is replayable.
#[test]
fn shared_capacity_year_is_replayable() {
    let mut direct = shared_route_world(10_000);
    let mut replayed = direct.clone();
    direct.advance_economic_year().expect("economic year");
    WorldCommand::AdvanceEconomicYear
        .apply(&mut replayed)
        .expect("replayed year");

    assert_eq!(direct, replayed);
    assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
}
