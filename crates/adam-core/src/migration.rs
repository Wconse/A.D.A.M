use crate::{
    AgeBand, CohortId, DomainEvent, EmploymentStatus, Money, Population, RegionId, SimDate, World,
};

const PRESSURE_GATE_MONTHS: u8 = 6;
const MAX_SERVICE_DISADVANTAGE_BPS: i32 = 1_000;

/// Auditable result of one household-scale internal migration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HouseholdMigration {
    pub source_cohort: CohortId,
    pub migrant_cohort: CohortId,
    pub from_region: RegionId,
    pub to_region: RegionId,
    pub people: Population,
    pub households: u64,
    pub liquid_wealth: Money,
    pub debt: Money,
    pub relocation_fee: Money,
    pub destination_vacancy_months: u8,
    pub service_advantage_basis_points: i16,
    pub destination_housing_pressure_basis_points: u16,
}

#[derive(Clone, Copy)]
struct Opportunity {
    destination: RegionId,
    vacancy_months: u8,
    service_advantage: i16,
    housing_pressure: u16,
    relocation_fee: Money,
    score: i32,
}

impl World {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn execute_annual_internal_migration(
        &mut self,
        date: SimDate,
    ) -> Vec<HouseholdMigration> {
        let source_regions: Vec<_> = self.regions.keys().copied().collect();
        let mut migrations = Vec::new();
        let mut next_id = self
            .cohorts
            .keys()
            .map(|id| id.get())
            .max()
            .unwrap_or(0)
            .saturating_add(1);

        for source in source_regions {
            let Some(opportunity) = self.best_migration_opportunity(source) else {
                continue;
            };
            let Some(source_cohort) = self.select_mobile_cohort(source) else {
                continue;
            };
            let migrant_id = CohortId::new(next_id);
            next_id = next_id.saturating_add(1);
            let source_snapshot = self.cohorts[&source_cohort].clone();
            let migrant_people = source_snapshot
                .people()
                .people()
                .div_ceil(source_snapshot.households());
            let required_reserve = household_monthly_income_share(&source_snapshot, migrant_people);
            let required_cash = required_reserve
                .checked_add(opportunity.relocation_fee.minor_units())
                .expect("migration cash requirement fits");
            if source_snapshot.liquid_wealth().minor_units() < required_cash {
                continue;
            }

            let mut migrant = self
                .cohorts
                .get_mut(&source_cohort)
                .expect("selected migration cohort exists")
                .split_migrating_household(migrant_id, opportunity.destination)
                .expect("selected cohort supports one-household partition");
            migrant
                .debit_wealth(opportunity.relocation_fee)
                .expect("selected migrant can fund relocation fee");
            let row = HouseholdMigration {
                source_cohort,
                migrant_cohort: migrant_id,
                from_region: source,
                to_region: opportunity.destination,
                people: migrant.people(),
                households: migrant.households(),
                liquid_wealth: migrant.liquid_wealth(),
                debt: migrant.debt(),
                relocation_fee: opportunity.relocation_fee,
                destination_vacancy_months: opportunity.vacancy_months,
                service_advantage_basis_points: opportunity.service_advantage,
                destination_housing_pressure_basis_points: opportunity.housing_pressure,
            };
            self.copy_persistent_cohort_state(source_cohort, migrant_id);
            self.cohorts.insert(migrant_id, migrant);
            let moved = row.people.people();
            let source_population = self.regions[&source].population().people() - moved;
            let destination_population = self.regions[&opportunity.destination]
                .population()
                .people()
                .checked_add(moved)
                .expect("migration destination population fits");
            self.regions
                .get_mut(&source)
                .expect("source region exists")
                .set_population(Population::new(source_population));
            self.regions
                .get_mut(&opportunity.destination)
                .expect("destination region exists")
                .set_population(Population::new(destination_population));
            let destination_country = self.regions[&opportunity.destination].country();
            let indicators = self
                .countries
                .get_mut(&destination_country)
                .expect("destination country exists")
                .indicators_mut();
            indicators.set_treasury(Money::from_minor_units(
                indicators
                    .treasury()
                    .minor_units()
                    .checked_add(row.relocation_fee.minor_units())
                    .expect("relocation fee treasury credit fits"),
            ));
            self.events.append(
                date,
                DomainEvent::HouseholdMigrated {
                    source_cohort: row.source_cohort,
                    migrant_cohort: row.migrant_cohort,
                    from_region: row.from_region,
                    to_region: row.to_region,
                    people: row.people,
                    households: row.households,
                    liquid_wealth: row.liquid_wealth,
                    debt: row.debt,
                    relocation_fee: row.relocation_fee,
                    destination_vacancy_months: row.destination_vacancy_months,
                    service_advantage_basis_points: row.service_advantage_basis_points,
                    destination_housing_pressure_basis_points: row
                        .destination_housing_pressure_basis_points,
                },
            );
            migrations.push(row);
        }
        if !migrations.is_empty() {
            debug_assert!(self.validate_population_accounting().is_ok());
        }
        migrations
    }

    fn best_migration_opportunity(&self, source: RegionId) -> Option<Opportunity> {
        let source_region = self.regions.get(&source)?;
        let source_labor = self.regional_labor_market.get(&source)?;
        if source_labor.unemployment_pressure_months < PRESSURE_GATE_MONTHS
            || source_labor.available_workers <= source_labor.vacancies
        {
            return None;
        }
        let source_services = service_score(self.regional_public_services.get(&source));
        self.regions
            .values()
            .filter(|region| region.id() != source && region.country() == source_region.country())
            .filter_map(|region| {
                let labor = self.regional_labor_market.get(&region.id())?;
                if labor.vacancy_pressure_months < PRESSURE_GATE_MONTHS
                    || labor.vacancies <= labor.available_workers
                {
                    return None;
                }
                let service_advantage =
                    service_score(self.regional_public_services.get(&region.id()))
                        - source_services;
                if service_advantage < -MAX_SERVICE_DISADVANTAGE_BPS {
                    return None;
                }
                let housing = self.regional_housing.get(&region.id())?;
                let occupied: u64 = self
                    .cohorts
                    .values()
                    .filter(|cohort| cohort.region() == region.id())
                    .map(crate::HouseholdCohort::households)
                    .sum();
                let projected_occupancy = occupied.checked_add(1)?;
                if projected_occupancy > housing.dwelling_capacity() {
                    return None;
                }
                let housing_pressure = u16::try_from(
                    (u128::from(projected_occupancy) * 10_000
                        / u128::from(housing.dwelling_capacity()))
                    .min(10_000),
                )
                .ok()?;
                let relocation_fee = relocation_fee(*housing, housing_pressure);
                let score = i32::from(labor.vacancy_pressure_months) * 200
                    + i32::from(source_labor.unemployment_pressure_months) * 100
                    + service_advantage
                    - i32::from(housing_pressure) / 2;
                Some(Opportunity {
                    destination: region.id(),
                    vacancy_months: labor.vacancy_pressure_months,
                    service_advantage: service_advantage
                        .clamp(i32::from(i16::MIN), i32::from(i16::MAX))
                        as i16,
                    housing_pressure,
                    relocation_fee,
                    score,
                })
            })
            .max_by(|a, b| {
                a.score
                    .cmp(&b.score)
                    .then_with(|| b.destination.cmp(&a.destination))
            })
    }

    fn select_mobile_cohort(&self, source: RegionId) -> Option<CohortId> {
        self.cohorts
            .values()
            .filter(|cohort| {
                cohort.region() == source
                    && cohort.employment() == EmploymentStatus::Unemployed
                    && matches!(cohort.age_band(), AgeBand::Adult | AgeBand::Mature)
                    && cohort.people().people() > 1
                    && cohort.households() > 1
                    && !self.workforce_training.contains_key(&cohort.id())
                    && !self
                        .employment_agreements
                        .values()
                        .any(|agreement| agreement.cohort() == cohort.id() && agreement.active())
            })
            .max_by(|a, b| {
                liquidity_per_household(a)
                    .cmp(&liquidity_per_household(b))
                    .then_with(|| b.id().cmp(&a.id()))
            })
            .map(crate::HouseholdCohort::id)
    }

    fn copy_persistent_cohort_state(&mut self, source: CohortId, migrant: CohortId) {
        if let Some(value) = self.cohort_health.get(&source).copied() {
            self.cohort_health.insert(migrant, value);
        }
        if let Some(value) = self.cohort_experience.get(&source).copied() {
            self.cohort_experience.insert(migrant, value);
        }
        if let Some(value) = self.social_stress.get(&source).copied() {
            self.social_stress.insert(migrant, value);
        }
        if let Some(value) = self.social_stress_memory.get(&source).copied() {
            self.social_stress_memory.insert(migrant, value);
        }
        if let Some(value) = self.deprivation_pressure.get(&source).copied() {
            self.deprivation_pressure.insert(migrant, value);
        }
        let skills: Vec<_> = self
            .cohort_skills
            .iter()
            .filter(|((cohort, _), _)| *cohort == source)
            .map(|((_, skill), proficiency)| (*skill, *proficiency))
            .collect();
        for (skill, proficiency) in skills {
            self.cohort_skills.insert((migrant, skill), proficiency);
        }
    }
}

fn service_score(services: Option<&crate::RegionalPublicServices>) -> i32 {
    services.map_or(5_000, |value| {
        (i32::from(value.healthcare().get())
            + i32::from(value.infrastructure().get())
            + i32::from(value.administration().get()))
            / 3
    })
}

fn relocation_fee(housing: crate::RegionalHousingMarket, pressure: u16) -> Money {
    let multiplier = 5_000_i128 + i128::from(pressure);
    let fee = (i128::from(housing.base_monthly_cost().minor_units()) * multiplier / 10_000).max(1);
    Money::from_minor_units(i64::try_from(fee).unwrap_or(i64::MAX))
}

fn liquidity_per_household(cohort: &crate::HouseholdCohort) -> i64 {
    cohort.liquid_wealth().minor_units() / i64::try_from(cohort.households()).unwrap_or(i64::MAX)
}

fn household_monthly_income_share(cohort: &crate::HouseholdCohort, people: u64) -> i64 {
    if cohort.people().people() == 0 {
        return 1;
    }
    let share = i128::from(cohort.annual_income().minor_units()) * i128::from(people)
        / i128::from(cohort.people().people())
        / 12;
    i64::try_from(share.max(1)).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis, EducationLevel,
        Good, GoodId, HouseholdCohort, HouseholdType, NeedProfileId, NeedTier, QuantityMilli,
        Region, RegionalHousingMarket, RegionalLaborMarketObservation, SimDate, WorldSeed,
    };

    #[allow(clippy::too_many_lines)]
    fn test_world() -> World {
        let mut world = World::new(WorldSeed::new(9), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Basic",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile");
        for (id, name, population) in [(1, "Oldtown", 100), (2, "Newport", 20)] {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        name,
                        Population::new(population),
                        Money::from_minor_units(10_000),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(100),
                    50,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Unemployed,
                    Money::from_minor_units(12_000),
                    Money::from_minor_units(5_000),
                    Money::from_minor_units(700),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(2),
                    RegionId::new(2),
                    NeedProfileId::new(1),
                    Population::new(20),
                    10,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::from_minor_units(2_400),
                    Money::from_minor_units(200),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world.regional_labor_market.insert(
            RegionId::new(1),
            RegionalLaborMarketObservation {
                region: RegionId::new(1),
                available_workers: 20,
                vacancies: 0,
                offers: 0,
                hires: 0,
                average_offered_wage: Money::default(),
                unemployment_pressure_months: 8,
                vacancy_pressure_months: 0,
            },
        );
        world.regional_labor_market.insert(
            RegionId::new(2),
            RegionalLaborMarketObservation {
                region: RegionId::new(2),
                available_workers: 1,
                vacancies: 8,
                offers: 3,
                hires: 0,
                average_offered_wage: Money::from_minor_units(130),
                unemployment_pressure_months: 0,
                vacancy_pressure_months: 7,
            },
        );
        world
            .set_regional_housing_market(
                RegionId::new(2),
                RegionalHousingMarket::new(20, Money::from_minor_units(10)).expect("housing"),
            )
            .expect("housing market");
        world
    }

    #[test]
    fn one_household_moves_without_creating_people_money_or_debt() {
        let mut world = test_world();
        let before_people: u64 = world
            .household_cohorts()
            .values()
            .map(|c| c.people().people())
            .sum();
        let before_wealth: i64 = world
            .household_cohorts()
            .values()
            .map(|c| c.liquid_wealth().minor_units())
            .sum::<i64>()
            + world.countries()[&crate::CountryId::new(1)]
                .indicators()
                .treasury()
                .minor_units();
        let before_debt: i64 = world
            .household_cohorts()
            .values()
            .map(|c| c.debt().minor_units())
            .sum();
        let rows = world.execute_annual_internal_migration(world.date());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].people.people(), 2);
        assert_eq!(rows[0].households, 1);
        assert_eq!(rows[0].relocation_fee, Money::from_minor_units(10));
        assert_eq!(rows[0].destination_housing_pressure_basis_points, 5_500);
        assert_eq!(
            world.household_cohorts()[&rows[0].migrant_cohort].region(),
            RegionId::new(2)
        );
        assert_eq!(world.regions()[&RegionId::new(1)].population().people(), 98);
        assert_eq!(world.regions()[&RegionId::new(2)].population().people(), 22);
        assert_eq!(
            before_people,
            world
                .household_cohorts()
                .values()
                .map(|c| c.people().people())
                .sum()
        );
        assert_eq!(
            before_wealth,
            world
                .household_cohorts()
                .values()
                .map(|c| c.liquid_wealth().minor_units())
                .sum::<i64>()
                + world.countries()[&crate::CountryId::new(1)]
                    .indicators()
                    .treasury()
                    .minor_units()
        );
        assert_eq!(
            before_debt,
            world
                .household_cohorts()
                .values()
                .map(|c| c.debt().minor_units())
                .sum()
        );
        world
            .validate_population_accounting()
            .expect("balanced after migration");
    }

    #[test]
    fn weak_pressure_or_missing_liquidity_prevents_migration() {
        let mut world = test_world();
        world
            .regional_labor_market
            .get_mut(&RegionId::new(2))
            .expect("labor")
            .vacancy_pressure_months = 5;
        assert!(
            world
                .execute_annual_internal_migration(world.date())
                .is_empty()
        );
        let mut full = test_world();
        full.set_regional_housing_market(
            RegionId::new(2),
            RegionalHousingMarket::new(10, Money::from_minor_units(10)).expect("housing"),
        )
        .expect("housing market");
        assert!(
            full.execute_annual_internal_migration(full.date())
                .is_empty()
        );
        let mut poor = test_world();
        poor.cohorts
            .get_mut(&CohortId::new(1))
            .expect("cohort")
            .debit_wealth(Money::from_minor_units(5_000))
            .expect("debit");
        assert!(
            poor.execute_annual_internal_migration(poor.date())
                .is_empty()
        );
    }
}
