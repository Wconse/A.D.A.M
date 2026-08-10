//! Stage 0 gate test: material bilateral grievance and emergent hostility.
//!
//! Invariants proven here:
//! 1. A country accrues bounded grievance toward a foreign country only on
//!    material evidence: one of its firms ends the month with unmet input
//!    demand while that country offered the good over a route that could
//!    deliver it in peace.
//! 2. Without a connecting route the same shortage creates no grievance:
//!    geography stays causal.
//! 3. Restored imports decay grievance toward zero and drop it at zero.
//! 4. Sustained material shortage escalates deterministically into the
//!    ordinary journaled bilateral hostility relation, and the grievance
//!    level stays bounded at the basis-point ceiling afterwards.
//! 5. Every path is replayable through the shared command boundary.
//!
//! World: a governed grain Farm in Farmland (country 1) and an ungoverned
//! Bakery in Bakerton (country 2), optionally connected by a road route. The
//! bakery is funded throughout; what decides whether imports settle is how
//! much grain exists against how many batches the bakery is built to bake.
//! Grievance must rest on goods that were not there, never on a till that
//! was empty or a price the buyer declined to pay.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, BasisPoints, CorporateRole, Country, CountryId, DomainEvent, Firm,
    FirmAppointment, FirmId, FirmPolicy, Good, GoodId, LogisticsRoute, Money, OwnershipStake,
    Population, ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId,
    RouteId, SimDate, TransportMode, World, WorldCommand, WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const GRAIN: u32 = 1;
const BREAD: u32 = 2;
const FARMLAND: u32 = 1;
const BAKERTON: u32 = 2;
const GRAIN_RECIPE: u32 = 1;
const BREAD_RECIPE: u32 = 2;

/// Two-country world without households: the farm is governed and offers
/// grain; the bakery is ungoverned, so observed management never adjusts its
/// default capacity target and its import order repeats every month.
#[allow(clippy::too_many_lines)]
fn grievance_world(bakery_cash_minor: i64, with_route: bool, bakery_capacity: u64) -> World {
    let mut world = World::new(WorldSeed::new(4_040), SimDate::new(2025, 1).expect("date"));
    world
        .register_country(Country::new(CountryId::new(1), "A").expect("country"))
        .expect("register country");
    world
        .register_country(Country::new(CountryId::new(2), "B").expect("country"))
        .expect("register country");
    world
        .register_good(Good::new(GoodId::new(GRAIN), "Grain").expect("good"))
        .expect("register grain");
    world
        .register_good(Good::new(GoodId::new(BREAD), "Bread").expect("good"))
        .expect("register bread");
    for (region, country, name) in [(FARMLAND, 1, "Farmland"), (BAKERTON, 2, "Bakerton")] {
        world
            .register_region(
                Region::new(
                    RegionId::new(region),
                    CountryId::new(country),
                    name,
                    Population::new(0),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("register region");
    }
    world
        .register_actor(
            Actor::new(ActorId::new(1), "Farm Owner", RegionId::new(FARMLAND), 1980)
                .expect("actor"),
        )
        .expect("register actor");
    world
        .register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(GRAIN_RECIPE),
                "Grain recipe",
                GoodId::new(GRAIN),
                QuantityMilli::new(1_000),
                1_000,
                vec![],
            )
            .expect("grain recipe"),
        )
        .expect("register grain recipe");
    world
        .register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(BREAD_RECIPE),
                "Bread recipe",
                GoodId::new(BREAD),
                QuantityMilli::new(2_000),
                1_000,
                vec![ProductionInput::new(
                    GoodId::new(GRAIN),
                    QuantityMilli::new(1_000),
                )],
            )
            .expect("bread recipe"),
        )
        .expect("register bread recipe");
    world
        .register_firm(
            Firm::new(
                FirmId::new(FARM),
                "Farm",
                RegionId::new(FARMLAND),
                RecipeId::new(GRAIN_RECIPE),
                1,
                1,
                Money::from_minor_units(1_000),
                BTreeMap::new(),
            )
            .expect("farm"),
        )
        .expect("register farm");
    world
        .register_firm(
            Firm::new(
                FirmId::new(BAKERY),
                "Bakery",
                RegionId::new(BAKERTON),
                RecipeId::new(BREAD_RECIPE),
                bakery_capacity,
                bakery_capacity,
                Money::from_minor_units(bakery_cash_minor),
                BTreeMap::new(),
            )
            .expect("bakery"),
        )
        .expect("register bakery");
    world
        .register_ownership_stake(OwnershipStake::new(
            FirmId::new(FARM),
            ActorId::new(1),
            BasisPoints::new(6_000).expect("rights"),
            BasisPoints::new(6_000).expect("rights"),
        ))
        .expect("ownership");
    world
        .register_firm_appointment(FirmAppointment::new(
            FirmId::new(FARM),
            ActorId::new(1),
            CorporateRole::OperationsManager,
        ))
        .expect("appointment");
    world
        .set_firm_policy(
            ActorId::new(1),
            FirmId::new(FARM),
            FirmPolicy::new(
                0,
                BasisPoints::new(0).expect("markup"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
            )
            .expect("policy"),
        )
        .expect("set farm policy");
    world
        .set_firm_production_target(ActorId::new(1), FirmId::new(FARM), 1)
        .expect("farm target");
    for (region, good, price) in [
        // Bread is dear enough that grain stays worth buying however far the
        // shortage pushes its price. A buyer that walks away on price has a
        // grievance against nobody, so a test about withheld supply must not
        // accidentally become a test about an unaffordable input.
        (FARMLAND, GRAIN, 5),
        (FARMLAND, BREAD, 10_000),
        (BAKERTON, GRAIN, 7),
        (BAKERTON, BREAD, 10_000),
    ] {
        world
            .set_regional_price(
                RegionId::new(region),
                GoodId::new(good),
                Money::from_minor_units(price),
            )
            .expect("regional price");
    }
    if with_route {
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(FARMLAND),
                    RegionId::new(BAKERTON),
                    TransportMode::Road,
                    QuantityMilli::new(1_000_000),
                    Money::from_minor_units(2),
                    7,
                    9_500,
                )
                .expect("route")
                .with_carrier(FirmId::new(FARM)),
            )
            .expect("register route");
    }
    world
}

fn run_month(direct: &mut World, replayed: &mut World) {
    direct
        .execute_monthly_economic_cycle()
        .expect("economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(replayed)
        .expect("replayed economic month");
}

fn grievance_level(world: &World) -> Option<u16> {
    world
        .bilateral_grievances()
        .get(&(CountryId::new(2), CountryId::new(1)))
        .map(|level| level.get())
}

#[test]
fn unmet_import_demand_with_reachable_foreign_supply_accrues_grievance() {
    let mut world = grievance_world(1_000_000, true, 2);
    let mut replayed = world.clone();
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed economic month");

    // The bakery has money and a price it is glad to pay; what it cannot get
    // is grain, because the foreign farm grows one batch where two are
    // needed. That is material evidence, not an injected flag. An empty till
    // would prove nothing here: a buyer who cannot pay has been denied
    // nothing by anyone abroad.
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(1_000)
    );
    assert_eq!(grievance_level(&world), Some(500));
    assert_eq!(world.bilateral_grievances().len(), 1);
    assert!(world.events().events().iter().any(|envelope| matches!(
        envelope.event(),
        DomainEvent::BilateralGrievanceChanged {
            aggrieved,
            target,
            level,
        } if *aggrieved == CountryId::new(2)
            && *target == CountryId::new(1)
            && level.get() == 500
    )));
    assert!(!world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn missing_route_shortage_creates_no_grievance() {
    let mut world = grievance_world(1_000_000, false, 2);
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");

    // The same shortage without a connecting route blames nobody: the
    // foreign offer was never deliverable, so no material grievance exists.
    // A funded buyer needing two batches goes wholly unserved here, where the
    // reachable world above left one batch of the two unmet.
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(2_000)
    );
    assert!(world.bilateral_grievances().is_empty());
    assert!(!world.events().events().iter().any(|envelope| matches!(
        envelope.event(),
        DomainEvent::BilateralGrievanceChanged { .. }
    )));
}

#[test]
fn restored_imports_decay_grievance_to_zero() {
    let mut world = grievance_world(1_000, true, 1);
    let mut replayed = world.clone();

    // Month 1: funded imports settle, so no grievance exists.
    run_month(&mut world, &mut replayed);
    assert!(world.bilateral_grievances().is_empty());

    // Month 2 under commanded hostility: the blocked import accrues.
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: true,
    }
    .apply(&mut world)
    .expect("hostility on");
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: true,
    }
    .apply(&mut replayed)
    .expect("replayed hostility on");
    run_month(&mut world, &mut replayed);
    assert_eq!(grievance_level(&world), Some(500));

    // Months 3-4 in restored peace: imports settle again and the grievance
    // decays back to zero and is dropped.
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: false,
    }
    .apply(&mut world)
    .expect("hostility off");
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: false,
    }
    .apply(&mut replayed)
    .expect("replayed hostility off");
    run_month(&mut world, &mut replayed);
    assert_eq!(grievance_level(&world), Some(250));
    run_month(&mut world, &mut replayed);
    assert_eq!(grievance_level(&world), None);
    assert!(world.bilateral_grievances().is_empty());
    assert!(world.events().events().iter().any(|envelope| matches!(
        envelope.event(),
        DomainEvent::BilateralGrievanceChanged { level, .. } if level.get() == 0
    )));
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn sustained_material_shortage_escalates_to_hostility_and_stays_bounded() {
    let mut world = grievance_world(1_000_000, true, 2);
    let mut replayed = world.clone();

    // Fifteen months of unmet import demand against available foreign supply
    // reach the escalation threshold exactly.
    for _ in 0..15 {
        run_month(&mut world, &mut replayed);
    }
    assert_eq!(grievance_level(&world), Some(7_500));
    assert!(world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert!(world.events().events().iter().any(|envelope| matches!(
        envelope.event(),
        DomainEvent::BilateralHostilityChanged {
            first,
            second,
            active: true,
        } if *first == CountryId::new(1) && *second == CountryId::new(2)
    )));

    // The embargo entrenches the grievance, but the level stays bounded at
    // the basis-point ceiling.
    for _ in 0..7 {
        run_month(&mut world, &mut replayed);
    }
    assert_eq!(grievance_level(&world), Some(BasisPoints::MAX));
    assert!(world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}
