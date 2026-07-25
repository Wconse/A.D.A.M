//! Stage 0 gate test: automatic de-escalation of emergent hostility.
//!
//! Invariants proven here:
//! 1. Hostility activated by grievance escalation is remembered as emergent
//!    and de-escalates automatically once both directed grievances have
//!    decayed away, through the same journaled hostility transition.
//! 2. Commanded hostility is never emergent: without a material grievance it
//!    stays active until commanded away, and commanded peace clears the
//!    emergent marker so it does not linger in authoritative state.
//! 3. Every path is replayable through the shared command boundary with a
//!    bit-identical fingerprint.
//!
//! World: a governed grain Farm in Farmland (country 1) and a funded,
//! ungoverned Bakery in Bakerton (country 2) whose capacity of two batches
//! needs twice the grain the farm can offer. The structural import shortfall
//! accrues grievance in peace and escalates into emergent hostility; a
//! domestic grain producer registered afterwards ends the material cause and
//! lets the world find its own way back to peace.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, BasisPoints, CorporateRole, Country, CountryId, DomainEvent, Firm,
    FirmAppointment, FirmId, FirmPolicy, Good, GoodId, LogisticsRoute, Money, OwnershipStake,
    Population, ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId,
    RouteId, SimDate, TransportMode, World, WorldCommand, WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const DOMESTIC_FARM: u32 = 3;
const GRAIN: u32 = 1;
const BREAD: u32 = 2;
const FARMLAND: u32 = 1;
const BAKERTON: u32 = 2;
const GRAIN_RECIPE: u32 = 1;
const BREAD_RECIPE: u32 = 2;

/// Two-country world without households: the governed farm offers one batch
/// of grain per month while the funded, ungoverned bakery needs two, so the
/// import shortfall is structural even though every settled trade clears.
#[allow(clippy::too_many_lines)]
fn deescalation_world() -> World {
    let mut world = World::new(WorldSeed::new(4_041), SimDate::new(2025, 1).expect("date"));
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
                2,
                2,
                Money::from_minor_units(1_000),
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
        (FARMLAND, GRAIN, 5),
        (FARMLAND, BREAD, 10),
        (BAKERTON, GRAIN, 7),
        (BAKERTON, BREAD, 10),
    ] {
        world
            .set_regional_price(
                RegionId::new(region),
                GoodId::new(good),
                Money::from_minor_units(price),
            )
            .expect("regional price");
    }
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
    world
}

/// Registers a governed domestic grain producer in Bakerton on both worlds.
/// The firm itself enters through the replayable `RegisterFirm` command;
/// governance registrations are applied identically to both timelines. The
/// firm is governed because only firms with an enacted policy plan market
/// offers.
fn register_domestic_grain_farm(world: &mut World, replayed: &mut World) {
    for target in [world, replayed] {
        target
            .register_actor(
                Actor::new(
                    ActorId::new(2),
                    "Bakerton Farmer",
                    RegionId::new(BAKERTON),
                    1980,
                )
                .expect("actor"),
            )
            .expect("register domestic actor");
        WorldCommand::RegisterFirm(
            Firm::new(
                FirmId::new(DOMESTIC_FARM),
                "Bakerton Farm",
                RegionId::new(BAKERTON),
                RecipeId::new(GRAIN_RECIPE),
                2,
                2,
                Money::from_minor_units(1_000),
                BTreeMap::new(),
            )
            .expect("domestic farm"),
        )
        .apply(target)
        .expect("register domestic farm");
        target
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(DOMESTIC_FARM),
                ActorId::new(2),
                BasisPoints::new(6_000).expect("rights"),
                BasisPoints::new(6_000).expect("rights"),
            ))
            .expect("domestic ownership");
        target
            .register_firm_appointment(FirmAppointment::new(
                FirmId::new(DOMESTIC_FARM),
                ActorId::new(2),
                CorporateRole::OperationsManager,
            ))
            .expect("domestic appointment");
        target
            .set_firm_policy(
                ActorId::new(2),
                FirmId::new(DOMESTIC_FARM),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                )
                .expect("policy"),
            )
            .expect("domestic policy");
        target
            .set_firm_production_target(ActorId::new(2), FirmId::new(DOMESTIC_FARM), 2)
            .expect("domestic target");
    }
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
fn emergent_hostility_deescalates_after_material_cause_ends() {
    let mut world = deescalation_world();
    let mut replayed = world.clone();

    // Fifteen months of structural import shortfall: the bakery buys the one
    // batch the farm offers but still ends every month one batch short, so
    // grievance accrues in peace and escalates at the threshold exactly.
    for _ in 0..15 {
        run_month(&mut world, &mut replayed);
    }
    assert_eq!(grievance_level(&world), Some(7_500));
    assert!(world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert_eq!(
        world
            .emergent_hostilities()
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![(CountryId::new(1), CountryId::new(2))]
    );

    // A domestic producer ends the material dependence: demand is met, the
    // grievance decays month by month, and when it is gone the emergent
    // hostility lifts itself through the same journaled transition.
    register_domestic_grain_farm(&mut world, &mut replayed);
    for _ in 0..30 {
        assert!(world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
        run_month(&mut world, &mut replayed);
    }
    assert_eq!(grievance_level(&world), None);
    assert!(world.bilateral_grievances().is_empty());
    assert!(!world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert!(world.emergent_hostilities().is_empty());
    assert!(world.events().events().iter().any(|envelope| matches!(
        envelope.event(),
        DomainEvent::BilateralHostilityChanged {
            first,
            second,
            active: false,
        } if *first == CountryId::new(1) && *second == CountryId::new(2)
    )));
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn commanded_hostility_without_grievance_never_deescalates() {
    let mut world = deescalation_world();
    let mut replayed = world.clone();

    // Commanded hostility between countries with no tracked grievance: the
    // monthly cycle must not treat it as emergent and lift it.
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
    assert!(world.emergent_hostilities().is_empty());

    // The embargo blocks the import, so grievance accrues and later decays
    // once peace is commanded; none of that may auto-lift commanded
    // hostility while it stays below the escalation threshold.
    for _ in 0..3 {
        run_month(&mut world, &mut replayed);
    }
    assert_eq!(grievance_level(&world), Some(1_500));
    assert!(world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert!(world.emergent_hostilities().is_empty());
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn commanded_peace_clears_the_emergent_marker() {
    let mut world = deescalation_world();
    let mut replayed = world.clone();

    // Escalate into emergent hostility, then command peace while grievance
    // is still high: the marker must clear so the pair behaves like any
    // commanded relation afterwards.
    for _ in 0..15 {
        run_month(&mut world, &mut replayed);
    }
    assert!(!world.emergent_hostilities().is_empty());
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: false,
    }
    .apply(&mut world)
    .expect("commanded peace");
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: false,
    }
    .apply(&mut replayed)
    .expect("replayed commanded peace");
    assert!(!world.countries_are_hostile(CountryId::new(1), CountryId::new(2)));
    assert!(world.emergent_hostilities().is_empty());
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}
