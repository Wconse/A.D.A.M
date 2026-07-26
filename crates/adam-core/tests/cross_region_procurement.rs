//! Stage 0 gate test: cross-region firm procurement over logistics routes.
//!
//! Invariants proven here:
//! 1. A firm without local suppliers imports intermediates over a registered
//!    route and settles at the delivered price: offer price plus route tariff.
//! 2. Local offers keep strict priority; imports only see the remainder, even
//!    when the delivered import price undercuts the local offer.
//! 3. Without a connecting route, import demand stays unmet: geography is
//!    causal, not cosmetic.
//! 4. A year with cross-region trade is replayable through the shared command
//!    boundary.
//!
//! World: a grain Farm in Farmland (region 1) and a Bakery in Bakerton
//! (region 2), optionally connected by a road route and optionally joined by
//! a pricier local grain farm in Bakerton.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, DomainEvent, EducationLevel,
    EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId,
    HouseholdCohort, HouseholdType, LogisticsRoute, Money, NeedProfileId, NeedTier, OwnershipStake,
    Population, ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId,
    RouteId, SimDate, TransportMode, World, WorldCommand, WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const LOCAL_FARM: u32 = 3;
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

#[allow(clippy::too_many_lines)]
fn two_region_world(
    route_tariff_minor: Option<i64>,
    with_local_farm: bool,
    route_capacity: u64,
) -> World {
    let mut world = World::new(WorldSeed::new(7_070), SimDate::new(2025, 1).expect("date"));
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
        .expect("register profile");
    for (region, country, name) in [(FARMLAND, 1, "Farmland"), (BAKERTON, 2, "Bakerton")] {
        world
            .register_region(
                Region::new(
                    RegionId::new(region),
                    CountryId::new(country),
                    name,
                    Population::new(2),
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
        .register_actor(
            Actor::new(
                ActorId::new(2),
                "Bakery Owner",
                RegionId::new(BAKERTON),
                1980,
            )
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
                1,
                1,
                Money::from_minor_units(1_000),
                BTreeMap::from([(GoodId::new(BREAD), QuantityMilli::new(2_000))]),
            )
            .expect("bakery"),
        )
        .expect("register bakery");
    if with_local_farm {
        world
            .register_firm(
                Firm::new(
                    FirmId::new(LOCAL_FARM),
                    "Bakerton Farm",
                    RegionId::new(BAKERTON),
                    RecipeId::new(GRAIN_RECIPE),
                    1,
                    1,
                    Money::from_minor_units(1_000),
                    BTreeMap::new(),
                )
                .expect("local farm"),
            )
            .expect("register local farm");
    }
    for (cohort, region) in [(1, FARMLAND), (2, BAKERTON)] {
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(cohort),
                    RegionId::new(region),
                    NeedProfileId::new(1),
                    Population::new(2),
                    2,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::default(),
                    Money::from_minor_units(100),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("register cohort");
    }
    install_governance(&mut world, FirmId::new(FARM), ActorId::new(1));
    install_governance(&mut world, FirmId::new(BAKERY), ActorId::new(2));
    if with_local_farm {
        install_governance(&mut world, FirmId::new(LOCAL_FARM), ActorId::new(2));
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
        .expect("register farm agreement");
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
        .expect("register bakery agreement");
    if with_local_farm {
        world
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(LOCAL_FARM),
                    CohortId::new(2),
                    1,
                    Money::from_minor_units(50),
                )
                .expect("local farm agreement"),
            )
            .expect("register local farm agreement");
    }
    world
        .set_regional_price(
            RegionId::new(FARMLAND),
            GoodId::new(GRAIN),
            Money::from_minor_units(5),
        )
        .expect("farmland grain price");
    world
        .set_regional_price(
            RegionId::new(FARMLAND),
            GoodId::new(BREAD),
            Money::from_minor_units(10),
        )
        .expect("farmland bread price");
    world
        .set_regional_price(
            RegionId::new(BAKERTON),
            GoodId::new(GRAIN),
            Money::from_minor_units(7),
        )
        .expect("bakerton grain price");
    world
        .set_regional_price(
            RegionId::new(BAKERTON),
            GoodId::new(BREAD),
            Money::from_minor_units(10),
        )
        .expect("bakerton bread price");
    if let Some(tariff) = route_tariff_minor {
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(FARMLAND),
                    RegionId::new(BAKERTON),
                    TransportMode::Road,
                    QuantityMilli::new(route_capacity),
                    Money::from_minor_units(tariff),
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

#[test]
fn imports_settle_at_delivered_price_when_no_local_supply() {
    let mut world = two_region_world(Some(2), false, 1_000_000);
    let result = world
        .execute_monthly_economic_cycle()
        .expect("first economic month");

    // The bakery imported one grain unit from the only supplier: the foreign
    // farm, at delivered price 5 (offer) + 2 (road tariff) = 7.
    assert_eq!(result.commercial.procurement.fills.len(), 1);
    let fill = result.commercial.procurement.fills[0];
    assert_eq!(fill.buyer, FirmId::new(BAKERY));
    assert_eq!(fill.seller, FirmId::new(FARM));
    assert_eq!(fill.quantity, QuantityMilli::new(1_000));
    assert_eq!(
        fill.spend,
        Money::from_minor_units(7),
        "grain offer 5 plus road tariff 2"
    );
    assert!(result.commercial.procurement.unmet.is_empty());

    // Physical delivery happened: the grain crossed regions.
    let farm = &world.firms()[&FirmId::new(FARM)];
    let bakery = &world.firms()[&FirmId::new(BAKERY)];
    assert!(farm.inventories().get(&GoodId::new(GRAIN)).is_none());
    assert_eq!(
        bakery.inventories()[&GoodId::new(GRAIN)],
        QuantityMilli::new(1_000)
    );

    // Both cash legs of the delivered price:
    // farm 1000 - 50 payroll + 7 delivered grain sale = 957
    // bakery 1000 - 50 payroll - 7 grain import + 20 bread sales = 963
    assert_eq!(farm.cash(), Money::from_minor_units(957));
    assert_eq!(bakery.cash(), Money::from_minor_units(963));

    // The buyer's observation carries the actually paid delivered price.
    let observation = world.firm_operating_history()[&FirmId::new(BAKERY)]
        .last()
        .expect("bakery observation");
    assert_eq!(
        observation.input_prices()[&GoodId::new(GRAIN)],
        Money::from_minor_units(7),
        "buyer observation must carry the delivered unit price"
    );
}

#[test]
fn missing_route_leaves_import_demand_unmet() {
    let mut world = two_region_world(None, false, 1_000_000);
    let result = world
        .execute_monthly_economic_cycle()
        .expect("first economic month");

    // Without a connecting route the foreign grain offer is unreachable.
    assert!(result.commercial.procurement.fills.is_empty());
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(1_000)
    );
    let bakery = &world.firms()[&FirmId::new(BAKERY)];
    assert!(bakery.inventories().get(&GoodId::new(GRAIN)).is_none());
}

#[test]
fn route_capacity_shortage_does_not_accrue_bilateral_grievance() {
    let mut direct = two_region_world(Some(2), false, 400);
    let mut replayed = direct.clone();
    let result = direct
        .execute_monthly_economic_cycle()
        .expect("capacity-limited economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed capacity-limited month");

    assert_eq!(result.commercial.procurement.fills.len(), 1);
    assert_eq!(
        result.commercial.procurement.fills[0].quantity,
        QuantityMilli::new(400),
        "the foreign farm still has grain, but the road can carry only 400"
    );
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(600)
    );
    assert!(
        direct.bilateral_grievances().is_empty(),
        "route-capacity scarcity must not blame the foreign supplier"
    );
    assert!(direct.events().events().iter().any(|event| matches!(
        event.event(),
        DomainEvent::FirmProcurementRouteCapacityShortfall {
            buyer,
            good,
            quantity,
        } if *buyer == FirmId::new(BAKERY)
            && *good == GoodId::new(GRAIN)
            && *quantity == QuantityMilli::new(600)
    )));
    assert!(!direct.events().events().iter().any(|event| matches!(
        event.event(),
        DomainEvent::FirmProcurementShortfall { buyer, good, .. }
            if *buyer == FirmId::new(BAKERY) && *good == GoodId::new(GRAIN)
    )));
    assert!(
        !direct
            .events()
            .events()
            .iter()
            .any(|event| matches!(event.event(), DomainEvent::BilateralGrievanceChanged { .. }))
    );
    assert_eq!(direct, replayed);
    assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn bilateral_hostility_severs_cross_border_procurement() {
    let mut world = two_region_world(Some(2), false, 1_000_000);
    WorldCommand::SetCountryHostility {
        first: CountryId::new(1),
        second: CountryId::new(2),
        active: true,
    }
    .apply(&mut world)
    .expect("hostility command");
    let mut replayed = world.clone();

    let result = world
        .execute_monthly_economic_cycle()
        .expect("hostile economic month");
    WorldCommand::ExecuteMonthlyEconomicCycle
        .apply(&mut replayed)
        .expect("replayed hostile economic month");

    assert!(result.commercial.procurement.fills.is_empty());
    assert_eq!(
        result.commercial.procurement.unmet[&(FirmId::new(BAKERY), GoodId::new(GRAIN))],
        QuantityMilli::new(1_000),
        "an active bilateral hostility blocks the only foreign supplier"
    );
    assert!(world.events().events().iter().any(|event| matches!(
        event.event(),
        DomainEvent::BilateralHostilityChanged {
            first,
            second,
            active: true,
        } if *first == CountryId::new(1) && *second == CountryId::new(2)
    )));
    assert_eq!(world, replayed);
    assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn local_offers_keep_priority_over_cheaper_delivered_imports() {
    let mut world = two_region_world(Some(1), true, 1_000_000);
    let result = world
        .execute_monthly_economic_cycle()
        .expect("first economic month");

    // Local grain sells at 7 while the import would deliver at 5 + 1 = 6.
    // Local supply still fills the whole order first.
    assert_eq!(result.commercial.procurement.fills.len(), 1);
    let fill = result.commercial.procurement.fills[0];
    assert_eq!(fill.buyer, FirmId::new(BAKERY));
    assert_eq!(
        fill.seller,
        FirmId::new(LOCAL_FARM),
        "local supply must fill before any import"
    );
    assert_eq!(fill.quantity, QuantityMilli::new(1_000));
    assert_eq!(fill.spend, Money::from_minor_units(7));
    assert!(result.commercial.procurement.unmet.is_empty());
}

#[test]
fn cross_region_trade_year_is_replayable() {
    let mut direct = two_region_world(Some(2), false, 1_000_000);
    let mut replayed = direct.clone();
    direct.advance_economic_year().expect("economic year");
    WorldCommand::AdvanceEconomicYear
        .apply(&mut replayed)
        .expect("replayed economic year");

    // The annual cycle preserves the delivered-price settlement recorded in
    // its first active procurement month.
    let import_spend: i64 = direct
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::FirmProcurementTrade {
                buyer,
                seller,
                spend,
                ..
            } if *buyer == FirmId::new(BAKERY) && *seller == FirmId::new(FARM) => {
                Some(spend.minor_units())
            }
            _ => None,
        })
        .sum();
    assert_eq!(
        import_spend, 7,
        "the recorded cross-region grain import uses delivered price 7"
    );

    // The year is replayable through the shared command boundary.
    assert_eq!(direct, replayed);
    assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
}
