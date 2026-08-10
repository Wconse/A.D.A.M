//! Liveness gate: a world built to feed itself must still be alive in twenty years.
//!
//! Every other integration test proves a mechanism in isolation over a month or a
//! year. None of them asks the question the whole engine exists to answer: given a
//! world that is viable by construction, do the engine's own rules keep it going,
//! or do they kill it?
//!
//! Viable by construction means two things here, and nothing more. Physically, the
//! region can grow more food than its people must eat. Monetarily, the wages paid
//! to produce the food are exactly enough to buy it back, so the circuit closes at
//! zero profit - the honest steady state for a world whose only income is wages.
//! No rule is relaxed, no constraint is softened, nothing is granted to anybody.
//! Only the opening conditions are chosen so that starvation is not preordained.
//!
//! The mirror test halves the harvest and asserts that this world does die. A
//! liveness assertion that cannot fail proves nothing.

use std::collections::BTreeMap;

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, DomainEvent, EducationLevel,
    EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId,
    HouseholdCohort, HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake, Population,
    ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, World, WorldSeed,
};

const FARM: u32 = 1;
const FOOD: u32 = 1;
const PEOPLE: u64 = 4;
const WORKERS: u64 = 2;
const WAGE: i64 = 100;
const PRICE: i64 = 50;

/// A single region that eats one good and grows it with nothing but labor.
///
/// Four people each need one unit of food a month. Two of them work, each turning
/// one worker-month into `harvest_milli` of food, so at the full harvest of 2.0 the
/// region grows exactly the four units it eats and holds twice that in idle
/// capacity for a larger population. Two wages of 100 meet four units priced at 50:
/// the money paid out to make the food is the money that comes back for it.
#[allow(clippy::too_many_lines)]
fn one_good_world(harvest_milli: u64) -> World {
    let mut world = World::new(WorldSeed::new(9_090), SimDate::new(2025, 1).expect("date"));
    world
        .register_country(Country::new(CountryId::new(1), "A").expect("country"))
        .expect("register country");
    world
        .register_good(Good::new(GoodId::new(FOOD), "Food").expect("good"))
        .expect("register food");
    world
        .register_consumption_profile(
            ConsumptionProfile::new(
                NeedProfileId::new(1),
                "Households",
                vec![ConsumptionTarget::new(
                    GoodId::new(FOOD),
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
                Population::new(PEOPLE),
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
                "Food recipe",
                GoodId::new(FOOD),
                QuantityMilli::new(harvest_milli),
                1_000,
                vec![],
            )
            .expect("food recipe"),
        )
        .expect("register food recipe");
    world
        .register_firm(
            Firm::new(
                FirmId::new(FARM),
                "Farm",
                RegionId::new(1),
                RecipeId::new(FARM),
                WORKERS,
                4,
                Money::from_minor_units(1_000),
                BTreeMap::from([(GoodId::new(FOOD), QuantityMilli::new(4_000))]),
            )
            .expect("farm"),
        )
        .expect("register farm");
    world
        .register_household_cohort(
            HouseholdCohort::new(
                CohortId::new(1),
                RegionId::new(1),
                NeedProfileId::new(1),
                Population::new(PEOPLE),
                PEOPLE,
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
        .expect("register cohort");
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
        .expect("set policy");
    world
        .set_firm_production_target(ActorId::new(1), FirmId::new(FARM), 2)
        .expect("target");
    world
        .register_employment_agreement(
            EmploymentAgreement::new(
                FirmId::new(FARM),
                CohortId::new(1),
                WORKERS,
                Money::from_minor_units(WAGE),
            )
            .expect("agreement"),
        )
        .expect("register agreement");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(FOOD),
            Money::from_minor_units(PRICE),
        )
        .expect("food price");
    world
}

/// Measured output for each year the region reported one, oldest first.
fn annual_output(world: &World) -> Vec<i64> {
    world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::RegionalOutputMeasured { annual_output, .. } => {
                Some(annual_output.minor_units())
            }
            _ => None,
        })
        .collect()
}

/// Everyone the region buried beyond its ordinary mortality.
fn excess_deaths(world: &World) -> u64 {
    world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::CohortHealthUpdated { excess_deaths, .. } => Some(*excess_deaths),
            _ => None,
        })
        .sum()
}

/// The share of the survival need the last reported reading found met.
fn final_survival_fulfillment(world: &World) -> Option<u16> {
    world
        .events()
        .events()
        .iter()
        .filter_map(|envelope| match envelope.event() {
            DomainEvent::CohortHealthUpdated {
                survival_fulfillment,
                ..
            } => Some(survival_fulfillment.get()),
            _ => None,
        })
        .next_back()
}

// The target this slice is aimed at, red on purpose until the circuit closes.
// Diagnosed: the farm's wage bill is its whole revenue, so the annual sales tax of
// twenty percent is paid out of capital and the firm dies in its third year. The
// seller defends a price that covers wages but not the tax it must hand over.
#[test]
#[ignore = "liveness target: a seller does not yet price in the tax it owes"]
fn a_world_that_can_feed_itself_is_still_alive_after_twenty_years() {
    let mut world = one_good_world(2_000);
    world.advance_economic_years(20).expect("twenty years");

    let output = annual_output(&world);
    assert_eq!(output.len(), 20, "every year must close its accounts");
    let live_years = output.iter().filter(|measured| **measured > 0).count();
    assert_eq!(
        live_years, 20,
        "a region that can grow its own food must not stop growing it: {output:?}"
    );

    let fulfillment = final_survival_fulfillment(&world).expect("a health reading");
    assert!(
        fulfillment >= 9_000,
        "the people of a self-feeding region must still be eating: {fulfillment} bps"
    );
}

#[test]
fn a_world_that_cannot_grow_enough_food_starves_however_well_it_prices() {
    let mut world = one_good_world(1_000);
    world.advance_economic_years(20).expect("twenty years");

    assert!(
        excess_deaths(&world) > 0,
        "half a harvest for four mouths must cost lives, or the liveness test above proves nothing"
    );
}
