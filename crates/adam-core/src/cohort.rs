use std::collections::BTreeMap;

use crate::{
    BasisPoints, CohortId, DomainEvent, Money, NeedProfileId, Population, RegionId, World,
    WorldError,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum AgeBand {
    Child,
    Youth,
    Adult,
    Mature,
    Senior,
}
impl AgeBand {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::Child => 1,
            Self::Youth => 2,
            Self::Adult => 3,
            Self::Mature => 4,
            Self::Senior => 5,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum HouseholdType {
    FamilyWithChildren,
    WorkingAge,
    Multigenerational,
    Retired,
}
impl HouseholdType {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::FamilyWithChildren => 1,
            Self::WorkingAge => 2,
            Self::Multigenerational => 3,
            Self::Retired => 4,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum EducationLevel {
    None,
    Basic,
    Secondary,
    Vocational,
    Tertiary,
}
impl EducationLevel {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::None => 1,
            Self::Basic => 2,
            Self::Secondary => 3,
            Self::Vocational => 4,
            Self::Tertiary => 5,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum EmploymentStatus {
    Dependent,
    Employed,
    Unemployed,
    Inactive,
    Retired,
}
impl EmploymentStatus {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::Dependent => 1,
            Self::Employed => 2,
            Self::Unemployed => 3,
            Self::Inactive => 4,
            Self::Retired => 5,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HouseholdCohort {
    id: CohortId,
    region: RegionId,
    need_profile: NeedProfileId,
    people: Population,
    households: u64,
    age_band: AgeBand,
    years_in_age_band: u8,
    household_type: HouseholdType,
    education: EducationLevel,
    employment: EmploymentStatus,
    annual_income: Money,
    liquid_wealth: Money,
    debt: Money,
}

impl HouseholdCohort {
    /// Creates one behaviorally homogeneous population/household cohort.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::InvalidCohort`] for impossible household counts or negative stocks.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: CohortId,
        region: RegionId,
        need_profile: NeedProfileId,
        people: Population,
        households: u64,
        age_band: AgeBand,
        household_type: HouseholdType,
        education: EducationLevel,
        employment: EmploymentStatus,
        annual_income: Money,
        liquid_wealth: Money,
        debt: Money,
    ) -> Result<Self, WorldError> {
        let people_count = people.people();
        if (people_count == 0 && households != 0)
            || (people_count > 0 && (households == 0 || households > people_count))
        {
            return Err(WorldError::InvalidCohort(
                "household count must be 1..=people for non-empty cohorts",
            ));
        }
        if annual_income.minor_units() < 0
            || liquid_wealth.minor_units() < 0
            || debt.minor_units() < 0
        {
            return Err(WorldError::InvalidCohort(
                "income, liquid wealth, and debt must be non-negative",
            ));
        }
        Ok(Self {
            id,
            region,
            need_profile,
            people,
            households,
            age_band,
            years_in_age_band: 0,
            household_type,
            education,
            employment,
            annual_income,
            liquid_wealth,
            debt,
        })
    }

    #[must_use]
    pub const fn id(&self) -> CohortId {
        self.id
    }
    #[must_use]
    pub const fn region(&self) -> RegionId {
        self.region
    }
    #[must_use]
    pub const fn need_profile(&self) -> NeedProfileId {
        self.need_profile
    }
    #[must_use]
    pub const fn people(&self) -> Population {
        self.people
    }
    #[must_use]
    pub const fn households(&self) -> u64 {
        self.households
    }
    #[must_use]
    pub const fn age_band(&self) -> AgeBand {
        self.age_band
    }
    #[must_use]
    pub const fn years_in_age_band(&self) -> u8 {
        self.years_in_age_band
    }
    pub(crate) fn advance_lifecycle_year(&mut self) -> Option<(AgeBand, AgeBand)> {
        let threshold = match self.age_band {
            AgeBand::Child => 13,
            AgeBand::Youth => 5,
            AgeBand::Adult => 30,
            AgeBand::Mature => 17,
            AgeBand::Senior => {
                self.years_in_age_band = self.years_in_age_band.saturating_add(1);
                return None;
            }
        };
        self.years_in_age_band = self.years_in_age_band.saturating_add(1);
        if self.years_in_age_band < threshold {
            return None;
        }
        let previous = self.age_band;
        self.years_in_age_band = 0;
        self.age_band = match previous {
            AgeBand::Child => AgeBand::Youth,
            AgeBand::Youth => AgeBand::Adult,
            AgeBand::Adult => AgeBand::Mature,
            AgeBand::Mature => AgeBand::Senior,
            AgeBand::Senior => return None,
        };
        match self.age_band {
            AgeBand::Adult if self.employment == EmploymentStatus::Dependent => {
                self.employment = EmploymentStatus::Unemployed;
                self.household_type = HouseholdType::WorkingAge;
            }
            AgeBand::Senior => {
                self.employment = EmploymentStatus::Retired;
                self.household_type = HouseholdType::Retired;
            }
            _ => {}
        }
        Some((previous, self.age_band))
    }
    #[must_use]
    pub const fn household_type(&self) -> HouseholdType {
        self.household_type
    }
    #[must_use]
    pub const fn education(&self) -> EducationLevel {
        self.education
    }
    pub(crate) const fn set_education(&mut self, education: EducationLevel) {
        self.education = education;
    }

    pub(crate) fn split_training_household(
        &mut self,
        new_id: CohortId,
    ) -> Result<Self, WorldError> {
        self.split_household(new_id, self.region)
    }

    pub(crate) fn split_migrating_household(
        &mut self,
        new_id: CohortId,
        destination: RegionId,
    ) -> Result<Self, WorldError> {
        self.split_household(new_id, destination)
    }

    fn split_household(
        &mut self,
        new_id: CohortId,
        destination: RegionId,
    ) -> Result<Self, WorldError> {
        let people = self.people.people();
        if people <= 1 || self.households <= 1 {
            return Err(WorldError::InvalidCohort(
                "training split requires multiple people and households",
            ));
        }
        let participants = people.div_ceil(self.households);
        let remaining_people = people - participants;
        let split_money = |value: Money| -> Result<Money, WorldError> {
            let share = i128::from(value.minor_units())
                .checked_mul(i128::from(participants))
                .ok_or(WorldError::ArithmeticOverflow("training cohort split"))?
                / i128::from(people);
            Ok(Money::from_minor_units(i64::try_from(share).map_err(
                |_| WorldError::ArithmeticOverflow("training cohort split"),
            )?))
        };
        let annual_income = split_money(self.annual_income)?;
        let liquid_wealth = split_money(self.liquid_wealth)?;
        let debt = split_money(self.debt)?;
        let mut training = Self::new(
            new_id,
            destination,
            self.need_profile,
            Population::new(participants),
            1,
            self.age_band,
            self.household_type,
            self.education,
            self.employment,
            annual_income,
            liquid_wealth,
            debt,
        )?;
        training.years_in_age_band = self.years_in_age_band;
        self.people = Population::new(remaining_people);
        self.households -= 1;
        self.annual_income =
            Money::from_minor_units(self.annual_income.minor_units() - annual_income.minor_units());
        self.liquid_wealth =
            Money::from_minor_units(self.liquid_wealth.minor_units() - liquid_wealth.minor_units());
        self.debt = Money::from_minor_units(self.debt.minor_units() - debt.minor_units());
        Ok(training)
    }
    #[must_use]
    pub const fn employment(&self) -> EmploymentStatus {
        self.employment
    }
    #[must_use]
    pub const fn annual_income(&self) -> Money {
        self.annual_income
    }
    #[must_use]
    pub const fn liquid_wealth(&self) -> Money {
        self.liquid_wealth
    }
    #[must_use]
    pub const fn debt(&self) -> Money {
        self.debt
    }
    pub(crate) fn debit_wealth(&mut self, amount: Money) -> Result<(), WorldError> {
        if amount.minor_units() < 0 || self.liquid_wealth.minor_units() < amount.minor_units() {
            return Err(WorldError::InsufficientHouseholdCash(self.id));
        }
        self.liquid_wealth =
            Money::from_minor_units(self.liquid_wealth.minor_units() - amount.minor_units());
        Ok(())
    }
    pub(crate) fn credit_wealth(&mut self, amount: Money) -> Result<(), WorldError> {
        let value = self
            .liquid_wealth
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("household wealth credit"))?;
        if value < 0 {
            return Err(WorldError::InvalidCohort(
                "wealth credit cannot make wealth negative",
            ));
        }
        self.liquid_wealth = Money::from_minor_units(value);
        Ok(())
    }
    pub(crate) fn borrow_for_survival(&mut self, amount: Money) -> Result<(), WorldError> {
        if amount.minor_units() <= 0 {
            return Err(WorldError::InvalidCohort(
                "survival borrowing must be positive",
            ));
        }
        let wealth = self
            .liquid_wealth
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("survival borrowing wealth"))?;
        let debt = self
            .debt
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("survival borrowing debt"))?;
        self.liquid_wealth = Money::from_minor_units(wealth);
        self.debt = Money::from_minor_units(debt);
        Ok(())
    }
    pub(crate) fn apply_monthly_cashflow(
        &mut self,
        income: Money,
    ) -> Result<HouseholdCashflow, WorldError> {
        let income = income.minor_units();
        let available = self
            .liquid_wealth
            .minor_units()
            .checked_add(income)
            .ok_or(WorldError::ArithmeticOverflow("household monthly income"))?;
        let scheduled = (self.debt.minor_units() / 120).max(0);
        let paid = available.min(scheduled);
        self.liquid_wealth = Money::from_minor_units(available - paid);
        self.debt = Money::from_minor_units(self.debt.minor_units() - paid);
        Ok(HouseholdCashflow {
            cohort: self.id,
            income: Money::from_minor_units(income),
            debt_service: Money::from_minor_units(paid),
            ending_wealth: self.liquid_wealth,
            ending_debt: self.debt,
        })
    }
    pub(crate) const fn set_people(&mut self, value: Population) {
        self.people = value;
    }
    pub(crate) fn apply_excess_deaths(&mut self, survivors: Population) -> Result<(), WorldError> {
        let previous = self.people.people();
        let remaining = survivors.people();
        if remaining > previous {
            return Err(WorldError::InvalidCohort(
                "deaths cannot increase population",
            ));
        }
        if previous == 0 {
            return Ok(());
        }
        self.households = if remaining == 0 {
            0
        } else {
            let scaled = u64::try_from(
                u128::from(self.households) * u128::from(remaining) / u128::from(previous),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("surviving households"))?;
            scaled.clamp(1, remaining)
        };
        self.annual_income = Money::from_minor_units(
            i64::try_from(
                i128::from(self.annual_income.minor_units()) * i128::from(remaining)
                    / i128::from(previous),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("surviving annual income"))?,
        );
        self.people = survivors;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HouseholdCashflow {
    pub cohort: CohortId,
    pub income: Money,
    pub debt_service: Money,
    pub ending_wealth: Money,
    pub ending_debt: Money,
}

impl World {
    /// Registers a cohort after checking identity and region references.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] for a duplicate ID or unknown region.
    pub fn register_household_cohort(&mut self, cohort: HouseholdCohort) -> Result<(), WorldError> {
        if self.cohorts.contains_key(&cohort.id()) {
            return Err(WorldError::DuplicateCohort(cohort.id()));
        }
        if !self.regions.contains_key(&cohort.region()) {
            return Err(WorldError::UnknownRegion(cohort.region()));
        }
        if !self
            .consumption_profiles
            .contains_key(&cohort.need_profile())
        {
            return Err(WorldError::UnknownNeedProfile(cohort.need_profile()));
        }
        self.cohort_health
            .insert(cohort.id(), crate::CohortHealth::default());
        self.events.append(
            self.date,
            DomainEvent::HouseholdCohortRegistered {
                cohort: cohort.id(),
                region: cohort.region(),
                people: cohort.people(),
            },
        );
        self.cohorts.insert(cohort.id(), cohort);
        Ok(())
    }

    #[must_use]
    pub fn household_cohorts(&self) -> &BTreeMap<CohortId, HouseholdCohort> {
        &self.cohorts
    }

    /// Confirms that cohorts are the complete population ledger for every region.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::PopulationAccounting`] when regional and cohort totals differ.
    pub fn validate_population_accounting(&self) -> Result<(), WorldError> {
        for region in self.regions.values() {
            let total = self
                .cohorts
                .values()
                .filter(|c| c.region() == region.id())
                .try_fold(0_u64, |sum, c| {
                    sum.checked_add(c.people().people())
                        .ok_or(WorldError::ArithmeticOverflow("cohort population sum"))
                })?;
            if total != region.population().people() {
                return Err(WorldError::PopulationAccounting {
                    region: region.id(),
                    region_population: region.population(),
                    cohort_population: Population::new(total),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn plan_region_cohort_rescale(
        &self,
        region: RegionId,
        target: Population,
    ) -> Result<Vec<(CohortId, Population)>, WorldError> {
        let ids: Vec<_> = self
            .cohorts
            .values()
            .filter(|c| c.region() == region)
            .map(HouseholdCohort::id)
            .collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let old_total = ids.iter().try_fold(0_u64, |sum, id| {
            sum.checked_add(self.cohorts[id].people().people())
                .ok_or(WorldError::ArithmeticOverflow("cohort rescale sum"))
        })?;
        if old_total == 0 {
            // An extinct region is a legitimate outcome: closing the year
            // with a zero target must be chronicled, not crash the run.
            if target.people() == 0 {
                return Ok(Vec::new());
            }
            return Err(WorldError::InvalidCohort(
                "cannot rescale an all-zero regional cohort ledger",
            ));
        }
        let target_total = target.people();
        let mut rows = Vec::with_capacity(ids.len());
        let mut assigned = 0_u64;
        for id in ids {
            let numerator =
                u128::from(self.cohorts[&id].people().people()) * u128::from(target_total);
            let base = u64::try_from(numerator / u128::from(old_total))
                .map_err(|_| WorldError::ArithmeticOverflow("cohort rescale base"))?;
            let remainder = numerator % u128::from(old_total);
            assigned = assigned
                .checked_add(base)
                .ok_or(WorldError::ArithmeticOverflow("cohort rescale assignment"))?;
            rows.push((id, base, remainder));
        }
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|&a, &b| {
            rows[b]
                .2
                .cmp(&rows[a].2)
                .then_with(|| rows[a].0.cmp(&rows[b].0))
        });
        let leftover = usize::try_from(target_total - assigned)
            .map_err(|_| WorldError::ArithmeticOverflow("cohort rescale remainder"))?;
        for index in order.into_iter().take(leftover) {
            rows[index].1 += 1;
        }
        let changes = rows
            .into_iter()
            .filter_map(|(id, count, _)| {
                let value = Population::new(count);
                (self.cohorts[&id].people() != value).then_some((id, value))
            })
            .collect();
        Ok(changes)
    }
}

fn uncontracted_monthly_income(
    annual_income: Money,
    people: Population,
    contracted_workers: u64,
    functional_capacity: u16,
) -> Result<Money, WorldError> {
    if people.people() == 0 {
        return Ok(Money::default());
    }
    let uncontracted_people = people.people().saturating_sub(contracted_workers);
    let minor_units = i128::from(annual_income.minor_units())
        .checked_mul(i128::from(uncontracted_people))
        .and_then(|value| value.checked_mul(i128::from(functional_capacity)))
        .ok_or(WorldError::ArithmeticOverflow(
            "uncontracted household income",
        ))?
        / i128::from(people.people())
        / 12
        / i128::from(BasisPoints::MAX);
    Ok(Money::from_minor_units(
        i64::try_from(minor_units)
            .map_err(|_| WorldError::ArithmeticOverflow("uncontracted household income"))?,
    ))
}

impl World {
    /// Applies monthly household income and debt service in canonical cohort order.
    /// # Errors
    /// Returns an error on arithmetic overflow before later cohorts are processed.
    pub fn execute_monthly_household_cashflows(
        &mut self,
    ) -> Result<Vec<HouseholdCashflow>, WorldError> {
        if self.last_household_cashflow_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "household cashflow",
                date: self.date,
            });
        }
        let mut updated = self.cohorts.clone();
        let mut rows = Vec::new();
        for cohort in updated.values_mut() {
            let contracted_workers = self
                .employment_agreements
                .values()
                .filter(|agreement| agreement.active() && agreement.cohort() == cohort.id())
                .try_fold(0_u64, |total, agreement| {
                    total
                        .checked_add(agreement.workers())
                        .ok_or(WorldError::ArithmeticOverflow("contracted cohort workers"))
                })?;
            let capacity = self
                .cohort_health
                .get(&cohort.id())
                .map_or(BasisPoints::MAX, |health| {
                    health.functional_capacity().get()
                });
            let fallback = uncontracted_monthly_income(
                cohort.annual_income(),
                cohort.people(),
                contracted_workers,
                capacity,
            )?;
            rows.push(cohort.apply_monthly_cashflow(fallback)?);
        }
        self.cohorts = updated;
        self.last_household_cashflow_date = Some(self.date);
        for row in &rows {
            self.events.append(
                self.date,
                crate::DomainEvent::HouseholdCashflowApplied {
                    cohort: row.cohort,
                    income: row.income,
                    debt_service: row.debt_service,
                    ending_wealth: row.ending_wealth,
                    ending_debt: row.ending_debt,
                },
            );
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis, Good, GoodId,
        Money, NeedProfileId, NeedTier, QuantityMilli, Region, SimDate, WorldSeed,
    };

    use super::*;

    fn cohort(id: u32, people: u64) -> HouseholdCohort {
        HouseholdCohort::new(
            CohortId::new(id),
            RegionId::new(1),
            NeedProfileId::new(1),
            Population::new(people),
            people / 2,
            AgeBand::Adult,
            HouseholdType::WorkingAge,
            EducationLevel::Secondary,
            EmploymentStatus::Employed,
            Money::from_minor_units(i64::try_from(people).expect("test population fits") * 100),
            Money::from_minor_units(0),
            Money::from_minor_units(0),
        )
        .expect("valid cohort")
    }

    fn world(region_population: u64) -> World {
        let mut world = World::new(
            WorldSeed::new(1),
            SimDate::new(2025, 1).expect("valid date"),
        );
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country registration");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good registration");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Test",
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
                    Population::new(region_population),
                    Money::from_minor_units(1_000_000),
                )
                .expect("region"),
            )
            .expect("region registration");
        world
    }

    #[test]
    fn accounting_rejects_mismatched_regional_population() {
        let mut world = world(1_001);
        world
            .register_household_cohort(cohort(1, 600))
            .expect("cohort");
        world
            .register_household_cohort(cohort(2, 400))
            .expect("cohort");
        assert!(matches!(
            world.validate_population_accounting(),
            Err(WorldError::PopulationAccounting { region, .. }) if region == RegionId::new(1)
        ));
    }

    #[test]
    fn largest_remainder_rescale_is_exact_and_deterministic() {
        let mut world = world(1_000);
        world
            .register_household_cohort(cohort(1, 600))
            .expect("cohort");
        world
            .register_household_cohort(cohort(2, 400))
            .expect("cohort");
        world
            .validate_population_accounting()
            .expect("balanced ledger");
        let plan = world
            .plan_region_cohort_rescale(RegionId::new(1), Population::new(1_003))
            .expect("rescale plan");
        assert_eq!(
            plan,
            vec![
                (CohortId::new(1), Population::new(602)),
                (CohortId::new(2), Population::new(401)),
            ]
        );
    }

    #[test]
    fn partial_employment_preserves_uncontracted_income() {
        assert_eq!(
            uncontracted_monthly_income(
                Money::from_minor_units(1_200),
                Population::new(100),
                25,
                BasisPoints::MAX,
            )
            .expect("partial income")
            .minor_units(),
            75
        );
        assert_eq!(
            uncontracted_monthly_income(
                Money::from_minor_units(1_200),
                Population::new(100),
                100,
                BasisPoints::MAX,
            )
            .expect("fully contracted income"),
            Money::default()
        );
    }

    #[test]
    fn lifecycle_ages_cohort_without_changing_real_stocks() {
        let mut cohort = HouseholdCohort::new(
            CohortId::new(1),
            RegionId::new(1),
            NeedProfileId::new(1),
            Population::new(20),
            10,
            AgeBand::Mature,
            HouseholdType::WorkingAge,
            EducationLevel::Secondary,
            EmploymentStatus::Employed,
            Money::from_minor_units(2_400),
            Money::from_minor_units(500),
            Money::from_minor_units(200),
        )
        .expect("cohort");
        cohort.years_in_age_band = 16;
        let stocks = (
            cohort.people(),
            cohort.households(),
            cohort.annual_income(),
            cohort.liquid_wealth(),
            cohort.debt(),
        );
        assert_eq!(
            cohort.advance_lifecycle_year(),
            Some((AgeBand::Mature, AgeBand::Senior))
        );
        assert_eq!(cohort.employment(), EmploymentStatus::Retired);
        assert_eq!(cohort.household_type(), HouseholdType::Retired);
        assert_eq!(
            stocks,
            (
                cohort.people(),
                cohort.households(),
                cohort.annual_income(),
                cohort.liquid_wealth(),
                cohort.debt(),
            )
        );
    }

    #[test]
    fn impossible_household_counts_are_rejected() {
        let result = HouseholdCohort::new(
            CohortId::new(1),
            RegionId::new(1),
            NeedProfileId::new(1),
            Population::new(10),
            11,
            AgeBand::Adult,
            HouseholdType::WorkingAge,
            EducationLevel::Secondary,
            EmploymentStatus::Employed,
            Money::from_minor_units(0),
            Money::from_minor_units(0),
            Money::from_minor_units(0),
        );
        assert!(matches!(result, Err(WorldError::InvalidCohort(_))));
    }
}
