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
    HouseholdCohort, HouseholdType, Money, MoneyTransactionPurpose, NeedProfileId, NeedTier,
    OwnershipStake, Population, ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region,
    RegionId, SimDate, World, WorldCommand, WorldSeed,
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
    // Grain fetches what a month of the farmer's work costs, and two loaves cover
    // that grain plus the baker's own month exactly. The two wages paid in this
    // world then buy back exactly the bread it bakes: a chain whose prices sat
    // below its wages only ever traded because somebody was made to sell at a loss,
    // and one whose prices sat above them would bake bread nobody could afford.
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(GRAIN),
            Money::from_minor_units(50),
        )
        .expect("grain price");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(BREAD),
            Money::from_minor_units(50),
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
    assert_eq!(fill.spend, Money::from_minor_units(50));
    let journal_entry = world
        .money_journal()
        .iter()
        .find(|entry| {
            matches!(
                entry.purpose(),
                MoneyTransactionPurpose::FirmPurchase {
                    buyer,
                    seller,
                    good,
                    carrier: None,
                } if buyer == FirmId::new(BAKERY)
                    && seller == FirmId::new(FARM)
                    && good == GoodId::new(GRAIN)
            )
        })
        .expect("firm purchase journal entry");
    assert_eq!(journal_entry.postings().len(), 2);

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
    // farm 1000 - 50 payroll + 50 grain sale = 1000
    // bakery 1000 - 50 payroll - 50 grain buy + 100 bread sales = 1000
    // households 100 + 100 wages - 100 bread = 100, no survival borrowing
    assert_eq!(farm.cash(), Money::from_minor_units(1_000));
    assert_eq!(bakery.cash(), Money::from_minor_units(1_000));
    let cohort = &world.household_cohorts()[&CohortId::new(1)];
    assert_eq!(cohort.liquid_wealth(), Money::from_minor_units(100));
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
    // Grain trades at a live regional price. The farm sells out every month while
    // the bakery still wants more, so scarcity repeatedly reprices grain upward and
    // the year's grain bill exceeds twelve trades at the opening price of 5.
    assert!(
        procurement_spend > 60,
        "sold-out grain must reprice upward over the year: {procurement_spend}"
    );

    // Settled household consumption is the only final demand.
    let household_spend: i64 = result
        .months
        .iter()
        .flat_map(|month| &month.commercial.clearing.fills)
        .map(|fill| fill.spend.minor_units())
        .sum();
    // Bread is priced live too. Households want more bread than the single bakery
    // can bake, and no new baker enters, so the shortage reprices bread upward all
    // year and the annual grocery bill exceeds twelve months at the opening price.
    assert!(
        household_spend > 240,
        "a permanently short bread market must reprice upward: {household_spend}"
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

    // Money conservation across the closed fiscal circuit: sales tax leaves firm
    // cash, nets against public spending in the treasury, and the spending itself
    // arrives as household wealth. The only way the stock of money can grow is
    // public borrowing, so firms + households + treasury must equal the opening
    // stock (2000 firm cash + 100 household wealth) plus the debt issued.
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
    assert!(taxes_paid > 0, "the producing chain does pay sales tax");
    let indicators = direct.countries()[&CountryId::new(1)].indicators();
    let treasury = indicators.treasury().minor_units();
    let public_debt = indicators.public_debt().minor_units();
    assert_eq!(
        farm_cash + bakery_cash + household_wealth + treasury,
        2_100 + public_debt,
        "money is created only by public borrowing, never by taxation"
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
    assert_eq!(fill.spend, Money::from_minor_units(60));

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
    assert_eq!(grain_sale.unit_price, Money::from_minor_units(60));
    assert_ne!(grain_sale.unit_price, Money::from_minor_units(50));
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
    assert_eq!(fill.spend, Money::from_minor_units(60));

    // The buyer's captured observation reports the actually paid grain price,
    // not the regional reference price.
    let observation = world.firm_operating_history()[&FirmId::new(BAKERY)]
        .last()
        .expect("bakery observation");
    assert_eq!(
        observation.input_prices()[&GoodId::new(GRAIN)],
        Money::from_minor_units(60),
        "buyer observation must carry the actually paid unit price"
    );
}

#[allow(clippy::too_many_lines)]
fn lean_bakery_world() -> World {
    // Same production chain as `production_chain_world`, except the bakery
    // opens the year without a grain buffer. It then finishes the year holding
    // the grain unit procured in December: a real positive grain inventory
    // investment whose valuation price is visible in the annual measurement.
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
                BTreeMap::from([(GoodId::new(BREAD), QuantityMilli::new(2_000))]),
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
    // Grain fetches what a month of the farmer's work costs, and two loaves cover
    // that grain plus the baker's own month exactly. The two wages paid in this
    // world then buy back exactly the bread it bakes: a chain whose prices sat
    // below its wages only ever traded because somebody was made to sell at a loss,
    // and one whose prices sat above them would bake bread nobody could afford.
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(GRAIN),
            Money::from_minor_units(50),
        )
        .expect("grain price");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(BREAD),
            Money::from_minor_units(50),
        )
        .expect("bread price");
    world
}

/// Capacity is bought, not wished for. Bread here opens far below the wage of
/// the month that bakes it, so the baker cannot pay for grain what the farmer
/// must charge to cover the field, and the farm ends the year selling less than
/// its payroll. A firm losing money on what it already makes does not pay
/// builders to make more of it, however hungry the market looks.
#[test]
fn a_loss_making_firm_refuses_to_buy_capacity() {
    let mut world = production_chain_world();
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(BREAD),
            Money::from_minor_units(5),
        )
        .expect("cheap bread");
    let opening_capacity = world.firms()[&FirmId::new(FARM)].capacity_batches();

    world.advance_economic_year().expect("economic year");

    let proposals = world
        .plan_observed_production_adjustments()
        .expect("proposals");
    let farm_margin = proposals
        .iter()
        .find(|proposal| proposal.firm == FirmId::new(FARM))
        .and_then(|proposal| proposal.expected_operating_cash_margin)
        .expect("farm margin");
    assert!(
        farm_margin.minor_units() < 0,
        "the farm is expected to run at a loss here: {proposals:?}"
    );

    // Whoever else in this world may be earning, the firm that is losing money
    // does not grow. Capacity is installed with cash, and this farm has none to
    // spare on more of a crop it cannot sell for what it costs to grow.
    assert_eq!(
        world.firms()[&FirmId::new(FARM)].capacity_batches(),
        opening_capacity,
        "capacity cannot grow without somebody paying for it"
    );
}

#[test]
fn inventory_investment_is_valued_at_observed_prices_not_reference_prices() {
    let mut world = lean_bakery_world();
    // A 20% farm markup separates the actual grain trade price (6) from the
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
    world.advance_economic_year().expect("economic year");

    // Physical premise: the bakery ends the year holding exactly the grain
    // unit procured in December (opening grain was zero) and no bread.
    let bakery = &world.firms()[&FirmId::new(BAKERY)];
    assert_eq!(
        bakery.inventories()[&GoodId::new(GRAIN)],
        QuantityMilli::new(1_000)
    );
    assert_eq!(
        bakery
            .inventories()
            .get(&GoodId::new(BREAD))
            .copied()
            .unwrap_or_default(),
        QuantityMilli::default()
    );

    // Monetary premise: all twelve monthly grain trades settled at 6.
    let procurement_spend: i64 = world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::FirmProcurementTrade { spend, .. } => Some(spend.minor_units()),
            _ => None,
        })
        .sum();
    // The 20% farm markup separates the trade price (60) from the reference price
    // (50), so the year's bill is not twelve trades at the table price. It is not
    // twelve trades at all: wages are the only income in this world, and two wages
    // of 50 cannot buy back bread that now has to carry a marked-up grain bill on
    // top of the baker's own month. Bread stays on the shelf, the loaf stops being
    // worth the grain, and the baker walks away from the counter rather than buy
    // an input it cannot recover. The markup is refused by the circuit, not by a rule.
    assert!(
        procurement_spend >= 60,
        "the surviving grain unit was bought at the marked-up price: {procurement_spend}"
    );
    assert_ne!(
        procurement_spend, 600,
        "the grain bill must not follow the reference price table"
    );

    // The grain investment (+1.0) must be valued at the actually paid price 6
    // and the bread drawdown (-2.0) at the actually observed sale price 10:
    // inventory change -14, where the reference table would report -15.
    let (final_consumption, inventory_change, annual_output) = world
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
    assert!(final_consumption > 0, "the bakery sells bread every month");
    // The surviving grain unit is carried at the price actually paid for it, and the
    // bread drawn down leaves at the price it actually sold for. Valuing the same
    // physical change from the reference price table would give a different number,
    // which is precisely what this test forbids.
    let grain_reference =
        world.regional_prices()[&(RegionId::new(1), GoodId::new(GRAIN))].minor_units();
    let bread_reference =
        world.regional_prices()[&(RegionId::new(1), GoodId::new(BREAD))].minor_units();
    assert_ne!(
        inventory_change,
        grain_reference - 2 * bread_reference,
        "inventory investment must not be valued from the reference price table"
    );
    assert_eq!(annual_output, final_consumption + inventory_change);
}

/// Registers one more firm on the same chain: same owner, same cohort, same
/// wage. Tests use it to put a second buyer or a second seller in one market.
fn register_chain_firm(
    world: &mut World,
    id: u32,
    recipe: u32,
    capacity: u64,
    cash: i64,
    inventories: BTreeMap<GoodId, QuantityMilli>,
) {
    world
        .register_firm(
            Firm::new(
                FirmId::new(id),
                format!("Firm {id}"),
                RegionId::new(1),
                RecipeId::new(recipe),
                capacity,
                capacity,
                Money::from_minor_units(cash),
                inventories,
            )
            .expect("firm"),
        )
        .expect("register firm");
    world
        .register_ownership_stake(OwnershipStake::new(
            FirmId::new(id),
            ActorId::new(1),
            BasisPoints::new(6_000).expect("rights"),
            BasisPoints::new(6_000).expect("rights"),
        ))
        .expect("ownership");
    world
        .register_firm_appointment(FirmAppointment::new(
            FirmId::new(id),
            ActorId::new(1),
            CorporateRole::OperationsManager,
        ))
        .expect("appointment");
    world
        .set_firm_policy(
            ActorId::new(1),
            FirmId::new(id),
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
        .set_firm_production_target(ActorId::new(1), FirmId::new(id), capacity)
        .expect("target");
    // No employment agreement: the two people in this world are already hired
    // by the farm and the bakery, and these firms exist to place orders, not
    // to compete for labour.
}

/// A buyer weighs an input against what it can make of it. One grain unit
/// becomes two bread units worth 120 in this chain, so grain asked at 400 is
/// refused however hungry the bread market is, and a price nobody will pay
/// cannot keep climbing on the strength of that refusal.
#[test]
fn a_buyer_refuses_an_input_dearer_than_the_bread_it_would_become() {
    let mut world = production_chain_world();
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(GRAIN),
            Money::from_minor_units(400),
        )
        .expect("dear grain");
    // A farm with grain on the shelf: the good is there and deliverable, so
    // nothing but the price stands between buyer and seller.
    register_chain_firm(
        &mut world,
        5,
        FARM,
        1,
        1_000,
        BTreeMap::from([(GoodId::new(GRAIN), QuantityMilli::new(9_000))]),
    );
    let result = world
        .execute_monthly_economic_cycle()
        .expect("economic month");

    assert!(
        result.commercial.procurement.fills.is_empty(),
        "a buyer that cannot recover the asking price must not buy: {:?}",
        result.commercial.procurement.fills
    );
    assert!(
        result
            .commercial
            .procurement
            .unmet
            .contains_key(&(FirmId::new(BAKERY), GoodId::new(GRAIN))),
        "the need does not vanish because the price was refused"
    );
    let grain_price =
        world.regional_prices()[&(RegionId::new(1), GoodId::new(GRAIN))].minor_units();
    assert!(
        grain_price < 400,
        "grain left standing at a refused price must fall, not rise: {grain_price}"
    );
}

/// Wanting is not demand. A mill without a coin in its till queues for grain
/// it cannot pay for, and the price must land exactly where it would have
/// landed had that mill never opened its doors.
#[test]
fn a_penniless_buyer_lifts_no_price_by_queueing() {
    let settled_grain_price = |with_pauper: bool| -> i64 {
        let mut world = production_chain_world();
        register_chain_firm(
            &mut world,
            5,
            FARM,
            1,
            1_000,
            BTreeMap::from([(GoodId::new(GRAIN), QuantityMilli::new(9_000))]),
        );
        if with_pauper {
            register_chain_firm(&mut world, 4, BAKERY, 3, 0, BTreeMap::new());
        }
        world
            .execute_monthly_economic_cycle()
            .expect("economic month");
        world.regional_prices()[&(RegionId::new(1), GoodId::new(GRAIN))].minor_units()
    };

    assert_eq!(
        settled_grain_price(false),
        settled_grain_price(true),
        "a queue of buyers who cannot pay is not scarcity and must not bid grain up"
    );
}

fn grain_outcome(world: &World, seller: u32) -> adam_core::MarketOfferOutcome {
    *world.firm_operating_history()[&FirmId::new(seller)]
        .last()
        .expect("seller observation")
        .market_outcomes()
        .iter()
        .find(|outcome| outcome.good == GoodId::new(GRAIN) && outcome.sold.get() > 0)
        .expect("settled grain sale outcome")
}

#[test]
fn a_farm_that_sold_out_with_a_mill_still_waiting_reads_the_queue() {
    let mut world = production_chain_world();
    // A second bakery wants two grain units a month. The farm can grow one, so
    // a real buyer walks away from this market empty-handed every month.
    register_chain_firm(&mut world, 3, BAKERY, 2, 1_000, BTreeMap::new());
    world
        .execute_monthly_economic_cycle()
        .expect("economic month");

    let grain = grain_outcome(&world, FARM);
    assert_eq!(grain.unsold, QuantityMilli::default());
    assert!(
        grain.unmet_market_demand.get() > 0,
        "a seller that emptied its shelf must be told what it could not serve"
    );
    assert!(
        grain.sold_out_while_demand_remained(),
        "a queue at the gate is the same evidence for a farm as for a baker"
    );
}

#[test]
fn a_farm_with_grain_left_on_the_shelf_does_not_claim_the_queue() {
    let mut world = production_chain_world();
    // One bakery with cash wants three grain units, one bakery without cash
    // wants one and gets nothing. A second farm opens the month holding far
    // more grain than anybody will take from it.
    register_chain_firm(&mut world, 3, BAKERY, 3, 1_000, BTreeMap::new());
    register_chain_firm(&mut world, 4, BAKERY, 1, 0, BTreeMap::new());
    register_chain_firm(
        &mut world,
        5,
        FARM,
        1,
        1_000,
        BTreeMap::from([(GoodId::new(GRAIN), QuantityMilli::new(9_000))]),
    );
    world
        .execute_monthly_economic_cycle()
        .expect("economic month");

    // The small farm sold every grain it had while a buyer still wanted grain.
    let emptied = grain_outcome(&world, FARM);
    assert!(
        emptied.sold_out_while_demand_remained(),
        "the farm that ran out did meet the queue"
    );
    // The large farm sold into the same market and kept stock. The unserved
    // buyer was not its problem, so it must not read a reason to expand.
    let stocked = grain_outcome(&world, 5);
    assert_eq!(
        stocked.unmet_market_demand,
        QuantityMilli::default(),
        "grain still on the shelf means nobody was turned away by this seller"
    );
    assert!(!stocked.sold_out_while_demand_remained());
}
