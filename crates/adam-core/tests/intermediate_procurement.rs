//! Stage 0 gate test: explicit intermediate-goods procurement between firms.
//!
//! Invariants proven here:
//! 1. B2B trades move real cash and inventories between firms every month.
//! 2. Measured regional output equals settled household final consumption
//!    plus valued inventory investment, and excludes intermediate turnover.
//! 3. Money is conserved across firms, households, and sales taxes.
//! 4. The full economic year is replayable through the shared command boundary.
//!
//! World: Farm (labor -> 1.0 grain/month) sells grain to Bakery
//! (1.0 grain -> 2.0 bread/month), households buy bread with wages.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, DomainEvent, EducationLevel,
    EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId,
    HouseholdCohort, HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake, Population,
    ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, World,
    WorldCommand, WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const GRAIN: u32 = 1;
const BREAD: u32 = 2;

#[allow(clippy::too_many_lines)]
fn production_chain_world() -> World {
    let mut world = World::new(WorldSeed::new(4_242), SimDate::new(2025, 1).expect("date"));
    world
        .register_country(Country::new(CountryId::new(1), "A").expect("country"))
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
    world
        .register_region(
            Region::new(
                RegionId::new(1),
                CountryId::new(1),
                "R",
                Population::new(2),
                Money::from_minor_units(1),
            )
            .expect("region"),
        )
        .expect("register region");
    world
        .register_actor(
            Actor::new(ActorId::new(1), "Owner", RegionId::new(1), 1980).expect("actor"),
        )
        .expect("register actor");
    world
        .register_production_recipe(
            ProductionRecipe::new(
                RecipeId::new(FARM),
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
                RecipeId::new(BAKERY),
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
                RegionId::new(1),
                RecipeId::new(FARM),
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
                RegionId::new(1),
                RecipeId::new(BAKERY),
                1,
                1,
                Money::from_minor_units(1_000),
                BTreeMap::from([
                    (GoodId::new(GRAIN), QuantityMilli::new(1_000)),
                    (GoodId::new(BREAD), QuantityMilli::new(2_000)),
                ]),
            )
            .expect("bakery"),
        )
        .expect("register bakery");
    world
        .register_household_cohort(
            HouseholdCohort::new(
                CohortId::new(1),
                RegionId::new(1),
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
    for firm in [FirmId::new(FARM), FirmId::new(BAKERY)] {
        world
            .register_ownership_stake(OwnershipStake::new(
                firm,
                ActorId::new(1),
                BasisPoints::new(6_000).expect("rights"),
                BasisPoints::new(6_000).expect("rights"),
            ))
            .expect("ownership");
        world
            .register_firm_appointment(FirmAppointment::new(
                firm,
                ActorId::new(1),
                CorporateRole::OperationsManager,
            ))
            .expect("appointment");
        world
            .set_firm_policy(
                ActorId::new(1),
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
            .set_firm_production_target(ActorId::new(1), firm, 1)
            .expect("target");
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
                CohortId::new(1),
                1,
                Money::from_minor_units(50),
            )
            .expect("bakery agreement"),
        )
        .expect("register bakery agreement");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(GRAIN),
            Money::from_minor_units(5),
        )
        .expect("grain price");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(BREAD),
            Money::from_minor_units(10),
        )
        .expect("bread price");
    world
}

#[test]
fn first_month_b2b_trade_moves_cash_and_inventories() {
    let mut world = production_chain_world();
    let result = world
        .execute_monthly_economic_cycle()
        .expect("first economic month");

    // One grain unit was bought by the bakery from the farm.
    assert_eq!(result.commercial.procurement.fills.len(), 1);
    let fill = result.commercial.procurement.fills[0];
    assert_eq!(fill.buyer, FirmId::new(BAKERY));
    assert_eq!(fill.seller, FirmId::new(FARM));
    assert_eq!(fill.quantity, QuantityMilli::new(1_000));
    assert_eq!(fill.spend, Money::from_minor_units(5));

    // Inventories moved: the farm sold all produced grain, the bakery holds it
    // for next month and kept two bread units after selling two to households.
    let farm = &world.firms()[&FirmId::new(FARM)];
    let bakery = &world.firms()[&FirmId::new(BAKERY)];
    assert!(farm.inventories().get(&GoodId::new(GRAIN)).is_none());
    assert_eq!(
        bakery.inventories()[&GoodId::new(GRAIN)],
        QuantityMilli::new(1_000)
    );
    assert_eq!(
        bakery.inventories()[&GoodId::new(BREAD)],
        QuantityMilli::new(2_000)
    );

    // Cash moved along payroll, procurement, and household purchases:
    // farm 1000 - 50 payroll + 5 grain sale = 955
    // bakery 1000 - 50 payroll - 5 grain buy + 20 bread sales = 965
    // households 100 + 100 wages - 20 bread = 180, no survival borrowing
    assert_eq!(farm.cash(), Money::from_minor_units(955));
    assert_eq!(bakery.cash(), Money::from_minor_units(965));
    let cohort = &world.household_cohorts()[&CohortId::new(1)];
    assert_eq!(cohort.liquid_wealth(), Money::from_minor_units(180));
    assert_eq!(cohort.debt(), Money::default());
    assert!(result.household_borrowing.is_empty());
}

#[test]
fn intermediate_turnover_is_excluded_from_regional_output_and_money_is_conserved() {
    let mut direct = production_chain_world();
    let mut replayed = direct.clone();
    let result = direct.advance_economic_year().expect("economic year");
    WorldCommand::AdvanceEconomicYear
        .apply(&mut replayed)
        .expect("replayed economic year");

    // Twelve monthly B2B grain trades moved money between the firms.
    let procurement_spend: i64 = direct
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
    assert_eq!(procurement_spend, 60, "twelve monthly grain trades at 5");

    // Settled household consumption is the only final demand.
    let household_spend: i64 = result
        .months
        .iter()
        .flat_map(|month| &month.commercial.clearing.fills)
        .map(|fill| fill.spend.minor_units())
        .sum();
    assert_eq!(
        household_spend, 240,
        "twelve months of two bread units at 10"
    );

    // Measured regional output counts final demand plus inventory investment
    // and excludes the intermediate grain turnover.
    let (final_consumption, inventory_change, annual_output) = direct
        .events()
        .events()
        .iter()
        .find_map(|envelope| match envelope.event() {
            DomainEvent::RegionalOutputMeasured {
                region,
                final_consumption,
                inventory_change,
                annual_output,
            } if *region == RegionId::new(1) => Some((
                final_consumption.minor_units(),
                inventory_change.minor_units(),
                annual_output.minor_units(),
            )),
            _ => None,
        })
        .expect("regional output measurement");
    assert_eq!(final_consumption, household_spend);
    assert_eq!(
        inventory_change, 0,
        "steady-state chain ends the year with opening inventories"
    );
    assert_eq!(annual_output, final_consumption + inventory_change);

    // Money conservation: initial firm cash (2000) + household wealth (100)
    // is preserved across firms, households, and paid sales taxes.
    let taxes_paid: i64 = direct
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::FirmSalesTaxPaid { paid, .. } => Some(paid.minor_units()),
            _ => None,
        })
        .sum();
    let farm_cash = direct.firms()[&FirmId::new(FARM)].cash().minor_units();
    let bakery_cash = direct.firms()[&FirmId::new(BAKERY)].cash().minor_units();
    let household_wealth = direct.household_cohorts()[&CohortId::new(1)]
        .liquid_wealth()
        .minor_units();
    assert_eq!(
        direct.household_cohorts()[&CohortId::new(1)].debt(),
        Money::default(),
        "no survival borrowing may mint money in this world"
    );
    assert_eq!(
        farm_cash + bakery_cash + household_wealth + taxes_paid,
        2_100,
        "cash moved between agents without creation or destruction"
    );

    // The year is replayable through the shared command boundary.
    assert_eq!(direct, replayed);
    assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
}

#[test]
fn procurement_observation_records_actual_trade_price_not_reference_price() {
    let mut world = production_chain_world();
    // A 20% farm markup separates the actual trade price (6) from the
    // regional reference price (5).
    world
        .set_firm_policy(
            ActorId::new(1),
            FirmId::new(FARM),
            FirmPolicy::new(
                0,
                BasisPoints::new(2_000).expect("markup"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
            )
            .expect("policy"),
        )
        .expect("set farm markup");
    let result = world
        .execute_monthly_economic_cycle()
        .expect("first economic month");

    // The B2B fill settled at the marked-up offer price, not the reference.
    assert_eq!(result.commercial.procurement.fills.len(), 1);
    let fill = result.commercial.procurement.fills[0];
    assert_eq!(fill.quantity, QuantityMilli::new(1_000));
    assert_eq!(fill.spend, Money::from_minor_units(6));

    // The seller's captured monthly observation reports the actual trade
    // price, not the regional reference price.
    let observation = world.firm_operating_history()[&FirmId::new(FARM)]
        .last()
        .expect("farm observation");
    let grain_sale = observation
        .market_outcomes()
        .iter()
        .find(|outcome| outcome.good == GoodId::new(GRAIN) && outcome.sold.get() > 0)
        .expect("settled grain sale outcome");
    assert_eq!(grain_sale.unit_price, Money::from_minor_units(6));
    assert_ne!(grain_sale.unit_price, Money::from_minor_units(5));
}
#[test]
fn buyer_observation_records_actual_procurement_price_not_reference_price() {
    let mut world = production_chain_world();
    // A 20% farm markup makes the actual B2B grain trade settle at 6 while
    // the regional reference price stays 5.
    world
        .set_firm_policy(
            ActorId::new(1),
            FirmId::new(FARM),
            FirmPolicy::new(
                0,
                BasisPoints::new(2_000).expect("markup"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
            )
            .expect("policy"),
        )
        .expect("set farm markup");
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");

    // The bakery actually paid 6 for one grain unit.
    assert_eq!(result.commercial.procurement.fills.len(), 1);
    let fill = result.commercial.procurement.fills[0];
    assert_eq!(fill.buyer, FirmId::new(BAKERY));
    assert_eq!(fill.spend, Money::from_minor_units(6));

    // The buyer's captured observation reports the actually paid grain price,
    // not the regional reference price.
    let observation = world.firm_operating_history()[&FirmId::new(BAKERY)]
        .last()
        .expect("bakery observation");
    assert_eq!(
        observation.input_prices()[&GoodId::new(GRAIN)],
        Money::from_minor_units(6),
        "buyer observation must carry the actually paid unit price"
    );
}
