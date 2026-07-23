use std::collections::BTreeMap;

use crate::{AgeBand, BasisPoints, CohortId, DomainEvent, NeedTier, Population, World, WorldError};

const HEALTH_MAINTENANCE_FULFILLMENT_BPS: u16 = 8_000;
const HEALTH_LOSS_DIVISOR: u16 = 12;
const HEALTH_RECOVERY_DIVISOR: u16 = 8;
const MORTALITY_RATE_SCALE: u128 = 1_000_000;

/// Persistent material health state derived from fulfilled survival consumption.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CohortHealth {
    survival_fulfillment: BasisPoints,
    functional_capacity: BasisPoints,
    mortality_remainder_ppm: u64,
}

impl Default for CohortHealth {
    fn default() -> Self {
        Self {
            survival_fulfillment: bounded_basis_points(BasisPoints::MAX),
            functional_capacity: bounded_basis_points(BasisPoints::MAX),
            mortality_remainder_ppm: 0,
        }
    }
}

impl CohortHealth {
    #[must_use]
    pub const fn survival_fulfillment(self) -> BasisPoints {
        self.survival_fulfillment
    }

    #[must_use]
    pub const fn functional_capacity(self) -> BasisPoints {
        self.functional_capacity
    }

    pub(crate) const fn mortality_remainder_ppm(self) -> u64 {
        self.mortality_remainder_ppm
    }
}

impl World {
    /// Converts settled survival consumption into persistent health, excess deaths, and workforce attrition.
    ///
    /// Healthy months restore capacity slowly. Severe shortages consume capacity and create age-sensitive
    /// excess mortality with a retained fixed-point remainder, so small cohorts cannot avoid mortality by
    /// integer rounding.
    ///
    /// # Errors
    /// Returns an error on duplicate execution or arithmetic overflow without partially applying changes.
    pub fn update_monthly_cohort_health(&mut self) -> Result<(), WorldError> {
        if self.last_cohort_health_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "cohort health",
                date: self.date,
            });
        }
        let mut cohorts = self.cohorts.clone();
        let mut regions = self.regions.clone();
        let mut agreements = self.employment_agreements.clone();
        let mut health = self.cohort_health.clone();
        let mut results = Vec::new();
        for cohort in cohorts.values_mut() {
            let previous_people = cohort.people().people();
            let row = health.entry(cohort.id()).or_default();
            let fulfillment = self.survival_fulfillment_bps(cohort.id())?;
            row.survival_fulfillment = bounded_basis_points(fulfillment);
            row.functional_capacity =
                next_functional_capacity(row.functional_capacity, fulfillment);
            let mortality_rate =
                excess_mortality_rate_ppm(cohort.age_band(), row.functional_capacity);
            let numerator = u128::from(previous_people)
                .checked_mul(u128::from(mortality_rate))
                .and_then(|value| value.checked_add(u128::from(row.mortality_remainder_ppm)))
                .ok_or(WorldError::ArithmeticOverflow("cohort excess mortality"))?;
            let deaths = u64::try_from(numerator / MORTALITY_RATE_SCALE)
                .map_err(|_| WorldError::ArithmeticOverflow("cohort excess deaths"))?
                .min(previous_people);
            row.mortality_remainder_ppm = u64::try_from(numerator % MORTALITY_RATE_SCALE)
                .map_err(|_| WorldError::ArithmeticOverflow("mortality remainder"))?;
            let survivors = previous_people - deaths;
            if deaths > 0 {
                cohort.apply_excess_deaths(Population::new(survivors))?;
                let region = regions
                    .get_mut(&cohort.region())
                    .ok_or(WorldError::UnknownRegion(cohort.region()))?;
                region.set_population(Population::new(
                    region
                        .population()
                        .people()
                        .checked_sub(deaths)
                        .ok_or(WorldError::ArithmeticOverflow("regional excess mortality"))?,
                ));
                for agreement in agreements
                    .values_mut()
                    .filter(|agreement| agreement.cohort() == cohort.id() && agreement.active())
                {
                    let previous_workers = agreement.workers();
                    let current_workers =
                        proportional_survivors(previous_workers, survivors, previous_people)?;
                    agreement.set_workers(current_workers);
                    if current_workers != previous_workers {
                        results.push(HealthResult::Employment {
                            firm: agreement.firm(),
                            cohort: cohort.id(),
                            previous_workers,
                            current_workers,
                        });
                    }
                }
            }
            results.push(HealthResult::Cohort {
                cohort: cohort.id(),
                survival_fulfillment: row.survival_fulfillment,
                functional_capacity: row.functional_capacity,
                excess_deaths: deaths,
                survivors,
            });
        }
        self.commit_monthly_health(cohorts, regions, agreements, health, results)
    }

    fn commit_monthly_health(
        &mut self,
        cohorts: BTreeMap<CohortId, crate::HouseholdCohort>,
        regions: BTreeMap<crate::RegionId, crate::Region>,
        agreements: BTreeMap<(crate::FirmId, CohortId), crate::EmploymentAgreement>,
        health: BTreeMap<CohortId, CohortHealth>,
        results: Vec<HealthResult>,
    ) -> Result<(), WorldError> {
        self.cohorts = cohorts;
        self.regions = regions;
        self.employment_agreements = agreements;
        self.cohort_health = health;
        self.last_cohort_health_date = Some(self.date);
        for result in results {
            match result {
                HealthResult::Cohort {
                    cohort,
                    survival_fulfillment,
                    functional_capacity,
                    excess_deaths,
                    survivors,
                } => self.events.append(
                    self.date,
                    DomainEvent::CohortHealthUpdated {
                        cohort,
                        survival_fulfillment,
                        functional_capacity,
                        excess_deaths,
                        survivors: Population::new(survivors),
                    },
                ),
                HealthResult::Employment {
                    firm,
                    cohort,
                    previous_workers,
                    current_workers,
                } => self.events.append(
                    self.date,
                    DomainEvent::EmploymentChanged {
                        firm,
                        cohort,
                        previous_workers,
                        current_workers,
                    },
                ),
            }
        }
        self.validate_population_accounting()
    }

    #[must_use]
    pub fn cohort_health(&self) -> &BTreeMap<CohortId, CohortHealth> {
        &self.cohort_health
    }

    fn survival_fulfillment_bps(&self, cohort: CohortId) -> Result<u16, WorldError> {
        let met = self
            .monthly_consumption
            .iter()
            .filter(|((id, _good, tier), _)| *id == cohort && *tier == NeedTier::Survival)
            .try_fold(0_u128, |sum, (_, quantity)| {
                sum.checked_add(u128::from(quantity.get()))
                    .ok_or(WorldError::ArithmeticOverflow("survival consumption"))
            })?;
        let unmet = self
            .unmet_demand
            .iter()
            .filter(|((id, _good, tier), _)| *id == cohort && *tier == NeedTier::Survival)
            .try_fold(0_u128, |sum, (_, quantity)| {
                sum.checked_add(u128::from(quantity.get()))
                    .ok_or(WorldError::ArithmeticOverflow("unmet survival demand"))
            })?;
        let total = met
            .checked_add(unmet)
            .ok_or(WorldError::ArithmeticOverflow("total survival demand"))?;
        if total == 0 {
            return Ok(BasisPoints::MAX);
        }
        u16::try_from(met * u128::from(BasisPoints::MAX) / total)
            .map_err(|_| WorldError::ArithmeticOverflow("survival fulfillment"))
    }
}

#[derive(Clone, Copy)]
enum HealthResult {
    Cohort {
        cohort: CohortId,
        survival_fulfillment: BasisPoints,
        functional_capacity: BasisPoints,
        excess_deaths: u64,
        survivors: u64,
    },
    Employment {
        firm: crate::FirmId,
        cohort: CohortId,
        previous_workers: u64,
        current_workers: u64,
    },
}

fn next_functional_capacity(current: BasisPoints, fulfillment: u16) -> BasisPoints {
    let current = i32::from(current.get());
    let fulfillment = i32::from(fulfillment);
    let next = if fulfillment < i32::from(HEALTH_MAINTENANCE_FULFILLMENT_BPS) {
        let deficit = i32::from(HEALTH_MAINTENANCE_FULFILLMENT_BPS) - fulfillment;
        current - divide_round_up(deficit, i32::from(HEALTH_LOSS_DIVISOR))
    } else {
        let surplus = fulfillment - i32::from(HEALTH_MAINTENANCE_FULFILLMENT_BPS);
        current + surplus / i32::from(HEALTH_RECOVERY_DIVISOR)
    };
    bounded_basis_points(u16::try_from(next.clamp(0, i32::from(BasisPoints::MAX))).unwrap_or(0))
}

const fn divide_round_up(value: i32, divisor: i32) -> i32 {
    (value + divisor - 1) / divisor
}

const fn age_vulnerability(age: AgeBand) -> u32 {
    match age {
        AgeBand::Child | AgeBand::Mature => 3,
        AgeBand::Youth | AgeBand::Adult => 2,
        AgeBand::Senior => 5,
    }
}

fn excess_mortality_rate_ppm(age: AgeBand, capacity: BasisPoints) -> u32 {
    u32::from(BasisPoints::MAX - capacity.get()) * age_vulnerability(age)
}

fn proportional_survivors(
    value: u64,
    survivors: u64,
    previous_people: u64,
) -> Result<u64, WorldError> {
    if previous_people == 0 {
        return Ok(0);
    }
    u64::try_from(u128::from(value) * u128::from(survivors) / u128::from(previous_people))
        .map_err(|_| WorldError::ArithmeticOverflow("proportional survivor allocation"))
}

fn bounded_basis_points(value: u16) -> BasisPoints {
    BasisPoints::new(value.min(BasisPoints::MAX)).unwrap_or(BasisPoints::HALF)
}

#[cfg(test)]
mod tests {
    use crate::{
        AgeBand, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
        EducationLevel, EmploymentStatus, Good, GoodId, HouseholdCohort, HouseholdType, Money,
        NeedProfileId, NeedTier, QuantityMilli, Region, RegionId, SimDate, WorldSeed,
    };

    use super::*;

    fn survival_world() -> World {
        let mut world = World::new(WorldSeed::new(47), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country registration");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good registration");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Food",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile registration");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Capital",
                    Population::new(100),
                    Money::from_minor_units(1_000_000),
                )
                .expect("region"),
            )
            .expect("region registration");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(100),
                    40,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::from_minor_units(1_200),
                    Money::default(),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort registration");
        world
    }

    #[test]
    fn persistent_severe_shortage_causes_health_loss_and_excess_deaths() {
        let mut world = survival_world();
        for _ in 0..24 {
            world.monthly_consumption.insert(
                (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
                QuantityMilli::new(10_000),
            );
            world.unmet_demand.insert(
                (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
                QuantityMilli::new(90_000),
            );
            world.update_monthly_cohort_health().expect("health update");
            world.advance_month().expect("month");
        }
        let health = world.cohort_health()[&CohortId::new(1)];
        assert!(health.functional_capacity().get() < 2_000);
        assert!(
            world.household_cohorts()[&CohortId::new(1)]
                .people()
                .people()
                < 100
        );
        world.validate_population_accounting().expect("accounting");
    }

    #[test]
    fn adequate_consumption_recovers_capacity_without_excess_deaths() {
        let mut world = survival_world();
        world.cohort_health.insert(
            CohortId::new(1),
            CohortHealth {
                survival_fulfillment: bounded_basis_points(5_000),
                functional_capacity: bounded_basis_points(5_000),
                mortality_remainder_ppm: 0,
            },
        );
        world.monthly_consumption.insert(
            (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
            QuantityMilli::new(100_000),
        );
        world.update_monthly_cohort_health().expect("health update");
        assert_eq!(
            world.household_cohorts()[&CohortId::new(1)].people(),
            Population::new(100)
        );
        assert!(
            world.cohort_health()[&CohortId::new(1)]
                .functional_capacity()
                .get()
                > 5_000
        );
    }
}
