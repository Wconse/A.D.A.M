//! Stage 0 gate test: the fiscal circuit is closed.
//!
//! Invariants proven here:
//! 1. Sales tax collected from firms equals the country's recorded revenue.
//! 2. Every minor unit of executed public spending arrives in household wealth,
//!    so taxation moves money instead of destroying it.
//! 3. Treasury and debt close consistently: the change in treasury less the
//!    change in debt equals revenue less spending.
//! 4. The outlay is split across cohorts by population without rounding leaks.
//!
//! World: Farm (labor -> 1.0 grain/month) sells grain to Bakery
//! (1.0 grain -> 2.0 bread/month); two cohorts of different size buy bread.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, DomainEvent, EducationLevel,
    EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId,
    HouseholdCohort, HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake, Population,
    ProductionInput, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, World,
    WorldSeed,
};

const FARM: u32 = 1;
const BAKERY: u32 = 2;
const GRAIN: u32 = 1;
const BREAD: u32 = 2;

#[allow(clippy::too_many_lines)]
fn fiscal_world() -> World {
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
                Population::new(3),
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
    // Two cohorts of different size make the population-weighted split observable.
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
    world
        .register_household_cohort(
            HouseholdCohort::new(
                CohortId::new(2),
                RegionId::new(1),
                NeedProfileId::new(1),
                Population::new(1),
                1,
                AgeBand::Adult,
                HouseholdType::WorkingAge,
                EducationLevel::Basic,
                EmploymentStatus::Unemployed,
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

fn fiscal_closure(world: &World) -> (i64, i64) {
    world
        .events()
        .events()
        .iter()
        .find_map(|envelope| match envelope.event() {
            DomainEvent::CountryFiscalYearClosed {
                country,
                revenue,
                spending,
                ..
            } if *country == CountryId::new(1) => {
                Some((revenue.minor_units(), spending.minor_units()))
            }
            _ => None,
        })
        .expect("fiscal closure event")
}

fn outlays(world: &World) -> Vec<(CohortId, i64)> {
    world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::PublicOutlayDistributed { cohort, amount, .. } => {
                Some((*cohort, amount.minor_units()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn collected_sales_tax_equals_recorded_revenue() {
    let mut world = fiscal_world();
    world.advance_economic_year().expect("economic year");

    let paid: i64 = world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::FirmSalesTaxPaid { paid, .. } => Some(paid.minor_units()),
            _ => None,
        })
        .sum();
    let (revenue, _) = fiscal_closure(&world);
    assert!(paid > 0, "a producing economy must pay some sales tax");
    assert_eq!(paid, revenue, "revenue is exactly what firms handed over");
}

#[test]
fn every_unit_of_public_spending_reaches_households() {
    let mut world = fiscal_world();
    world.advance_economic_year().expect("economic year");

    let (_, spending) = fiscal_closure(&world);
    let distributed: i64 = outlays(&world).iter().map(|(_, amount)| amount).sum();
    assert!(spending > 0, "the demo government does spend");
    assert_eq!(
        distributed, spending,
        "public spending must arrive as household purchasing power, not vanish"
    );
}

#[test]
fn treasury_and_debt_close_against_revenue_and_spending() {
    let mut world = fiscal_world();
    let opening = world.countries()[&CountryId::new(1)].indicators();
    let opening_treasury = opening.treasury().minor_units();
    let opening_debt = opening.public_debt().minor_units();
    world.advance_economic_year().expect("economic year");
    let closing = world.countries()[&CountryId::new(1)].indicators();

    let (revenue, spending) = fiscal_closure(&world);
    assert_eq!(
        (closing.treasury().minor_units() - opening_treasury)
            - (closing.public_debt().minor_units() - opening_debt),
        revenue - spending,
        "the closure may borrow or save, but it may not invent money"
    );
}

#[test]
fn outlay_is_split_by_population_without_rounding_leaks() {
    let mut world = fiscal_world();
    world.advance_economic_year().expect("economic year");

    let (_, spending) = fiscal_closure(&world);
    let shares = outlays(&world);
    assert_eq!(shares.len(), 2, "both cohorts are paid");
    let larger = shares
        .iter()
        .find(|(cohort, _)| *cohort == CohortId::new(1))
        .expect("two-person cohort share")
        .1;
    let smaller = shares
        .iter()
        .find(|(cohort, _)| *cohort == CohortId::new(2))
        .expect("one-person cohort share")
        .1;
    assert!(
        larger > smaller,
        "a cohort of two receives more than a cohort of one"
    );
    assert_eq!(
        larger + smaller,
        spending,
        "largest-remainder split loses no minor unit"
    );
}
