//! Temporary diagnostic probe for the closed-circuit liveness failure. Delete once diagnosed.

use std::collections::{BTreeMap, BTreeSet};

use adam_core::{
    Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
    CorporateRole, Country, CountryId, DemandBasis, EducationLevel, EmploymentAgreement,
    EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good, GoodId, HouseholdCohort,
    HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake, Population, ProductionRecipe,
    QuantityMilli, RecipeId, Region, RegionId, SimDate, World, WorldSeed,
};

const FARM: u32 = 1;
const FOOD: u32 = 1;

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
                Population::new(4),
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
                2,
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
                Population::new(4),
                4,
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
                2,
                Money::from_minor_units(100),
            )
            .expect("agreement"),
        )
        .expect("register agreement");
    world
        .set_regional_price(
            RegionId::new(1),
            GoodId::new(FOOD),
            Money::from_minor_units(50),
        )
        .expect("food price");
    world
}

fn variant_name(debug: &str) -> String {
    debug
        .split([' ', '{', '('])
        .next()
        .unwrap_or(debug)
        .to_string()
}

#[test]
fn probe_three_years() {
    let mut world = one_good_world(2_000);
    let mut seen = 0_usize;
    let watched: BTreeSet<&str> = [
        "Employment",
        "Insolven",
        "Downsiz",
        "Arrears",
        "Payroll",
        "Liquidat",
        "SalesTax",
        "MarketTrade",
        "Production",
        "Shed",
        "Unprofitable",
        "Target",
    ]
    .into_iter()
    .collect();

    for year in 1..=5 {
        world.advance_economic_year().expect("year");
        let events = world.events().events();
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        let mut interesting: Vec<String> = Vec::new();
        for envelope in events.iter().skip(seen) {
            let debug = format!("{:?}", envelope.event());
            let name = variant_name(&debug);
            *counts.entry(name.clone()).or_default() += 1;
            if watched.iter().any(|needle| name.contains(needle)) {
                let mut line = debug;
                line.truncate(150);
                interesting.push(line);
            }
        }
        seen = events.len();

        let farm = &world.firms()[&FirmId::new(FARM)];
        let cohort = &world.household_cohorts()[&CohortId::new(1)];
        println!("===== YEAR {year} =====");
        println!(
            "farm cash {:?} inventories {:?} | cohort wealth {:?} debt {:?}",
            farm.cash(),
            farm.inventories(),
            cohort.liquid_wealth(),
            cohort.debt()
        );
        println!("-- counts --");
        for (name, count) in &counts {
            println!("{count:>4} {name}");
        }
        println!("-- watched --");
        for line in interesting.iter().take(40) {
            println!("{line}");
        }
    }
}
