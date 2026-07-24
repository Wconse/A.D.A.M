//! Regression test: a fully extinct region must close economic years
//! without crashing the annual demographic rescale.

use adam_core::{
    AgeBand, CohortId, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
    DomainEvent, EducationLevel, EmploymentStatus, Good, GoodId, HouseholdCohort, HouseholdType,
    Money, NeedProfileId, NeedTier, Population, QuantityMilli, Region, RegionId, SimDate, World,
    WorldSeed,
};

fn extinct_world() -> World {
    let mut world = World::new(WorldSeed::new(11), SimDate::new(2026, 1).expect("date"));
    world
        .register_country(Country::new(CountryId::new(1), "Arcadia").expect("country"))
        .expect("register country");
    world
        .register_good(Good::new(GoodId::new(1), "Bread").expect("good"))
        .expect("register good");
    world
        .register_consumption_profile(
            ConsumptionProfile::new(
                NeedProfileId::new(1),
                "Households",
                vec![ConsumptionTarget::new(
                    GoodId::new(1),
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
                "Ashes",
                Population::new(0),
                Money::from_minor_units(1),
            )
            .expect("region"),
        )
        .expect("register region");
    // A regional price is mandatory world configuration for every consumed
    // good, even when the local population has already died out.
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(1),
            Money::from_minor_units(10),
        )
        .expect("bread price");
    world
        .register_household_cohort(
            HouseholdCohort::new(
                CohortId::new(1),
                RegionId::new(1),
                NeedProfileId::new(1),
                Population::new(0),
                0,
                AgeBand::Adult,
                HouseholdType::WorkingAge,
                EducationLevel::Secondary,
                EmploymentStatus::Employed,
                Money::default(),
                Money::default(),
                Money::default(),
            )
            .expect("cohort"),
        )
        .expect("register cohort");
    world
}

#[test]
fn extinct_region_survives_annual_closure() {
    let mut world = extinct_world();
    world
        .advance_economic_years(3)
        .expect("extinct region years close without errors");
    let last_population = world
        .events()
        .events()
        .iter()
        .rev()
        .find_map(|envelope| match envelope.event() {
            DomainEvent::RegionPopulationChanged { population, .. } => Some(population.people()),
            _ => None,
        })
        .expect("population event");
    assert_eq!(last_population, 0, "an extinct region must stay extinct");
}
