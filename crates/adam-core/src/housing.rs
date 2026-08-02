use crate::{DomainEvent, Money, Population, RegionId, SimDate, WorldError};

const CONSTRUCTION_PRESSURE_BPS: u16 = 9_000;
const CONSTRUCTION_YEARS: u8 = 2;

/// A fully funded public housing project waiting for physical completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HousingConstruction {
    dwellings: u64,
    years_remaining: u8,
    committed_cost: Money,
}

impl HousingConstruction {
    #[must_use]
    pub const fn dwellings(self) -> u64 {
        self.dwellings
    }

    #[must_use]
    pub const fn years_remaining(self) -> u8 {
        self.years_remaining
    }

    #[must_use]
    pub const fn committed_cost(self) -> Money {
        self.committed_cost
    }
}

/// Physical regional housing stock, cost benchmark, and funded public construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalHousingMarket {
    dwelling_capacity: u64,
    base_monthly_cost: Money,
    public_capital: Money,
    construction: Option<HousingConstruction>,
}

impl RegionalHousingMarket {
    /// Creates a housing market with physical dwelling capacity and a positive baseline cost.
    /// # Errors
    /// Rejects zero capacity or non-positive cost.
    pub fn new(dwelling_capacity: u64, base_monthly_cost: Money) -> Result<Self, WorldError> {
        if dwelling_capacity == 0 || base_monthly_cost.minor_units() <= 0 {
            return Err(WorldError::InvalidHousing(
                "housing capacity and baseline cost must be positive",
            ));
        }
        Ok(Self {
            dwelling_capacity,
            base_monthly_cost,
            public_capital: Money::default(),
            construction: None,
        })
    }

    pub(crate) fn derived(population: Population, annual_output: Money) -> Self {
        let dwellings = population.people().div_ceil(2).max(1);
        let cost = if population.people() == 0 {
            1
        } else {
            (annual_output.minor_units()
                / i64::try_from(population.people()).unwrap_or(i64::MAX)
                / 120)
                .max(1)
        };
        Self {
            dwelling_capacity: dwellings,
            base_monthly_cost: Money::from_minor_units(cost),
            public_capital: Money::default(),
            construction: None,
        }
    }

    #[must_use]
    pub const fn dwelling_capacity(self) -> u64 {
        self.dwelling_capacity
    }

    #[must_use]
    pub const fn base_monthly_cost(self) -> Money {
        self.base_monthly_cost
    }

    #[must_use]
    pub const fn public_capital(self) -> Money {
        self.public_capital
    }

    #[must_use]
    pub const fn construction(self) -> Option<HousingConstruction> {
        self.construction
    }

    fn start_construction(&mut self, dwellings: u64, cost: Money) {
        self.construction = Some(HousingConstruction {
            dwellings,
            years_remaining: CONSTRUCTION_YEARS,
            committed_cost: cost,
        });
    }

    fn advance_construction(&mut self) -> Option<HousingConstruction> {
        let mut project = self.construction?;
        project.years_remaining = project.years_remaining.saturating_sub(1);
        if project.years_remaining > 0 {
            self.construction = Some(project);
            return None;
        }
        self.dwelling_capacity = self
            .dwelling_capacity
            .checked_add(project.dwellings)
            .expect("housing capacity addition fits");
        self.public_capital = Money::from_minor_units(
            self.public_capital
                .minor_units()
                .checked_add(project.committed_cost.minor_units())
                .expect("public housing capital addition fits"),
        );
        self.construction = None;
        Some(project)
    }
}

impl crate::World {
    /// Replaces the housing assumptions for a registered region.
    /// # Errors
    /// Rejects unknown regions or invalid physical/economic values.
    pub fn set_regional_housing_market(
        &mut self,
        region: RegionId,
        market: RegionalHousingMarket,
    ) -> Result<(), WorldError> {
        if !self.regions.contains_key(&region) {
            return Err(WorldError::UnknownRegion(region));
        }
        self.regional_housing.insert(region, market);
        Ok(())
    }

    #[must_use]
    pub fn regional_housing_markets(
        &self,
    ) -> &std::collections::BTreeMap<RegionId, RegionalHousingMarket> {
        &self.regional_housing
    }

    pub(crate) fn execute_annual_housing_investment(&mut self, date: SimDate) {
        let regions: Vec<_> = self.regions.keys().copied().collect();
        for region in &regions {
            let completed = self
                .regional_housing
                .get_mut(region)
                .expect("registered region has housing")
                .advance_construction();
            if let Some(project) = completed {
                let capacity = self.regional_housing[region].dwelling_capacity();
                self.events.append(
                    date,
                    DomainEvent::RegionalHousingConstructionCompleted {
                        region: *region,
                        dwellings: project.dwellings(),
                        committed_cost: project.committed_cost(),
                        dwelling_capacity: capacity,
                    },
                );
            }
        }

        for region in regions {
            let market = self.regional_housing[&region];
            if market.construction().is_some() {
                continue;
            }
            let occupied: u64 = self
                .cohorts
                .values()
                .filter(|cohort| cohort.region() == region)
                .map(crate::HouseholdCohort::households)
                .sum();
            let pressure = u16::try_from(
                (u128::from(occupied) * 10_000 / u128::from(market.dwelling_capacity()))
                    .min(10_000),
            )
            .expect("housing pressure is bounded");
            if pressure < CONSTRUCTION_PRESSURE_BPS {
                continue;
            }
            let dwellings = market.dwelling_capacity().div_ceil(10).max(1);
            let cost_minor =
                i128::from(market.base_monthly_cost().minor_units()) * 12 * i128::from(dwellings);
            let Ok(cost_minor) = i64::try_from(cost_minor) else {
                continue;
            };
            let cost = Money::from_minor_units(cost_minor);
            let country = self.regions[&region].country();
            let treasury = self.countries[&country].indicators().treasury();
            if treasury.minor_units() < cost.minor_units() {
                continue;
            }
            // Dwellings are built by people, and those people are paid. Until
            // now the treasury was debited and nobody was credited, so the money
            // simply left the world - the same leak ADR 0170 closed for taxation
            // and ADR 0172 closed for capital works. A region with no households
            // has nobody to do the building, so no project starts there.
            let shares = self.plan_regional_capital_shares(region, cost);
            if shares.is_empty() {
                continue;
            }
            self.countries
                .get_mut(&country)
                .expect("housing country exists")
                .indicators_mut()
                .set_treasury(Money::from_minor_units(
                    treasury.minor_units() - cost.minor_units(),
                ));
            for (cohort, share) in shares {
                self.cohorts
                    .get_mut(&cohort)
                    .expect("regional cohort exists")
                    .credit_wealth(share)
                    .expect("building income fits in the cohort's wealth");
                self.events.append(
                    date,
                    DomainEvent::HousingOutlayDistributed {
                        region,
                        cohort,
                        amount: share,
                    },
                );
            }
            self.regional_housing
                .get_mut(&region)
                .expect("housing market exists")
                .start_construction(dwellings, cost);
            self.events.append(
                date,
                DomainEvent::RegionalHousingConstructionStarted {
                    region,
                    dwellings,
                    committed_cost: cost,
                    housing_pressure_basis_points: pressure,
                    years_remaining: CONSTRUCTION_YEARS,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget, Country, CountryId,
        CountryIndicators, DemandBasis, EducationLevel, EmploymentStatus, Good, GoodId,
        HouseholdCohort, HouseholdType, NeedProfileId, NeedTier, QuantityMilli, Region, World,
        WorldSeed,
    };

    use super::*;

    #[allow(clippy::too_many_lines)]
    fn pressured_world(treasury: i64) -> World {
        let mut world = World::new(WorldSeed::new(4), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(
                Country::new(CountryId::new(1), "A")
                    .expect("country")
                    .with_indicators(CountryIndicators::new(
                        Money::from_minor_units(treasury),
                        Money::default(),
                        BasisPoints::HALF,
                        BasisPoints::HALF,
                    )),
            )
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
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Capital",
                    Population::new(10),
                    Money::from_minor_units(12_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(10),
                    5,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Unemployed,
                    Money::from_minor_units(1_200),
                    Money::from_minor_units(100),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
            .set_regional_housing_market(
                RegionId::new(1),
                RegionalHousingMarket::new(5, Money::from_minor_units(10)).expect("housing"),
            )
            .expect("housing market");
        world
    }

    #[test]
    fn pressure_funds_delayed_capacity_without_free_dwellings() {
        let mut world = pressured_world(1_000);
        let start = world.date();
        world.execute_annual_housing_investment(start);
        let market = world.regional_housing[&RegionId::new(1)];
        assert_eq!(market.dwelling_capacity(), 5);
        assert_eq!(market.construction().expect("project").dwellings(), 1);
        assert_eq!(
            market.construction().expect("project").committed_cost(),
            Money::from_minor_units(120)
        );
        assert_eq!(
            world.countries[&CountryId::new(1)].indicators().treasury(),
            Money::from_minor_units(880)
        );

        world.execute_annual_housing_investment(SimDate::new(2026, 1).expect("date"));
        assert_eq!(
            world.regional_housing[&RegionId::new(1)].dwelling_capacity(),
            5
        );
        world.execute_annual_housing_investment(SimDate::new(2027, 1).expect("date"));
        let completed = world.regional_housing[&RegionId::new(1)];
        assert_eq!(completed.dwelling_capacity(), 6);
        assert_eq!(completed.public_capital(), Money::from_minor_units(120));
        assert!(completed.construction().is_none());
    }

    #[test]
    fn building_money_reaches_the_households_that_build() {
        let mut world = pressured_world(1_000);
        let opening_wealth = world.cohorts[&CohortId::new(1)].liquid_wealth();
        let opening_treasury = world.countries[&CountryId::new(1)].indicators().treasury();
        world.execute_annual_housing_investment(world.date());

        let closing_wealth = world.cohorts[&CohortId::new(1)].liquid_wealth();
        let closing_treasury = world.countries[&CountryId::new(1)].indicators().treasury();
        let spent = opening_treasury.minor_units() - closing_treasury.minor_units();
        assert_eq!(spent, 120, "the project was funded");
        assert_eq!(
            closing_wealth.minor_units() - opening_wealth.minor_units(),
            spent,
            "every unit the treasury spent on building was earned by a builder"
        );
        let paid: i64 = world
            .events
            .events()
            .iter()
            .filter_map(|envelope| match envelope.event() {
                DomainEvent::HousingOutlayDistributed { amount, .. } => Some(amount.minor_units()),
                _ => None,
            })
            .sum();
        assert_eq!(paid, spent, "the payment is reported, not silent");
    }

    #[test]
    fn insufficient_treasury_cannot_start_construction() {
        let mut world = pressured_world(119);
        world.execute_annual_housing_investment(world.date());
        assert!(
            world.regional_housing[&RegionId::new(1)]
                .construction()
                .is_none()
        );
        assert_eq!(
            world.countries[&CountryId::new(1)].indicators().treasury(),
            Money::from_minor_units(119)
        );
    }
}
