use crate::{CohortId, FirmId, Money, World, WorldError};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmploymentAgreement {
    firm: FirmId,
    cohort: CohortId,
    workers: u64,
    monthly_wage_per_worker: Money,
    arrears: Money,
    active: bool,
}
impl EmploymentAgreement {
    /// Creates a firm-to-cohort wage agreement.
    /// # Errors
    /// Returns an error for zero workers or non-positive wage.
    pub fn new(
        firm: FirmId,
        cohort: CohortId,
        workers: u64,
        wage: Money,
    ) -> Result<Self, WorldError> {
        if workers == 0 || wage.minor_units() <= 0 {
            return Err(WorldError::InvalidEmployment(
                "workers and wage must be positive",
            ));
        }
        Ok(Self {
            firm,
            cohort,
            workers,
            monthly_wage_per_worker: wage,
            arrears: Money::default(),
            active: true,
        })
    }
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn cohort(&self) -> CohortId {
        self.cohort
    }
    #[must_use]
    pub const fn workers(&self) -> u64 {
        self.workers
    }
    #[must_use]
    pub const fn wage(&self) -> Money {
        self.monthly_wage_per_worker
    }
    #[must_use]
    pub const fn arrears(&self) -> Money {
        self.arrears
    }
    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }
    pub(crate) fn set_workers(&mut self, workers: u64) {
        self.workers = workers;
        self.active = workers > 0;
    }

    pub(crate) const fn set_wage(&mut self, wage: Money) {
        self.monthly_wage_per_worker = wage;
    }

    pub(crate) fn settle_arrears(&mut self) -> Money {
        let settled = self.arrears;
        self.arrears = Money::default();
        settled
    }

    pub(crate) fn settle_arrears_up_to(&mut self, available: Money) -> Money {
        let paid = self
            .arrears
            .minor_units()
            .min(available.minor_units().max(0));
        self.arrears = Money::from_minor_units(self.arrears.minor_units() - paid);
        Money::from_minor_units(paid)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PayrollRecord {
    pub firm: FirmId,
    pub cohort: CohortId,
    pub owed: Money,
    pub paid: Money,
    pub arrears: Money,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmploymentMatch {
    pub firm: FirmId,
    pub cohort: CohortId,
    pub wage: Money,
    pub minimum_education: crate::EducationLevel,
    pub labor_market_adjustment_basis_points: i16,
}

/// Persistent monthly evidence for one regional competitive labor market.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalLaborMarketObservation {
    pub region: crate::RegionId,
    pub available_workers: u64,
    pub vacancies: u64,
    pub offers: u64,
    pub hires: u64,
    pub average_offered_wage: Money,
    pub unemployment_pressure_months: u8,
    pub vacancy_pressure_months: u8,
}
impl World {
    /// Configures the minimum education accepted by one production recipe.
    /// # Errors
    /// Returns an unknown-recipe error without mutation.
    pub fn set_recipe_minimum_education(
        &mut self,
        recipe: crate::RecipeId,
        education: crate::EducationLevel,
    ) -> Result<(), WorldError> {
        if !self.production_recipes.contains_key(&recipe) {
            return Err(WorldError::UnknownRecipe(recipe));
        }
        self.recipe_minimum_education.insert(recipe, education);
        Ok(())
    }

    #[must_use]
    pub fn recipe_minimum_education(&self) -> &BTreeMap<crate::RecipeId, crate::EducationLevel> {
        &self.recipe_minimum_education
    }

    /// Matches at most one qualified unallocated worker to each staffed vacancy.
    /// Offers are ranked by wage and then canonical firm identity, so scarce labor goes to
    /// the highest bid while equal worlds remain replay-identical.
    /// # Errors
    /// Returns an error atomically for duplicate monthly execution or an invalid match.
    #[allow(clippy::too_many_lines)]
    pub fn execute_observed_labor_matching(&mut self) -> Result<Vec<EmploymentMatch>, WorldError> {
        if self.last_labor_market_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed labor matching",
                date: self.date,
            });
        }
        let mut offers = Vec::new();
        for firm in self.firms.keys().copied() {
            if self.is_firm_insolvent(firm) || self.firm_labor_vacancy(firm)? == 0 {
                continue;
            }
            let definition = &self.firms[&firm];
            // Labor matching is opt-in per recipe. Legacy content without an explicit
            // qualification profile keeps its pre-market staffing behavior.
            let Some(minimum) = self
                .recipe_minimum_education
                .get(&definition.recipe())
                .copied()
            else {
                continue;
            };
            let Some(cohort) =
                self.select_qualified_unallocated_cohort(definition.region(), minimum)
            else {
                continue;
            };
            let (wage, labor_market_adjustment_basis_points) =
                self.competitive_labor_bid(firm, cohort)?;
            if definition.cash().minor_units() >= wage.minor_units() {
                offers.push(EmploymentMatch {
                    firm,
                    cohort,
                    wage,
                    minimum_education: minimum,
                    labor_market_adjustment_basis_points,
                });
            }
        }
        offers.sort_by(|first, second| {
            second
                .wage
                .cmp(&first.wage)
                .then_with(|| first.firm.cmp(&second.firm))
                .then_with(|| first.cohort.cmp(&second.cohort))
        });
        let offers_count = offers.len();
        let mut offer_stats: BTreeMap<crate::RegionId, (u64, i128)> = BTreeMap::new();
        for offer in &offers {
            let region = self.firms[&offer.firm].region();
            let stats = offer_stats.entry(region).or_default();
            stats.0 = stats.0.saturating_add(1);
            stats.1 = stats
                .1
                .checked_add(i128::from(offer.wage.minor_units()))
                .ok_or(WorldError::ArithmeticOverflow("regional labor offer wages"))?;
        }
        let mut next = self.clone();
        let mut matches = Vec::new();
        for offer in offers {
            if next.firm_labor_vacancy(offer.firm)? == 0
                || next.available_cohort_workers(offer.cohort) == 0
            {
                continue;
            }
            crate::WorldCommand::MatchEmploymentWorker(offer).apply(&mut next)?;
            matches.push(offer);
        }
        let configured_regions: BTreeSet<_> = next
            .firms
            .values()
            .filter(|firm| next.recipe_minimum_education.contains_key(&firm.recipe()))
            .map(crate::Firm::region)
            .collect();
        for region in configured_regions {
            let available_workers = next
                .cohorts
                .iter()
                .filter(|(_, cohort)| {
                    cohort.region() == region
                        && cohort.employment() == crate::EmploymentStatus::Unemployed
                })
                .map(|(cohort, _)| next.available_cohort_workers(*cohort))
                .sum();
            let vacancies = next
                .firms
                .iter()
                .filter(|(_, firm)| {
                    firm.region() == region
                        && next.recipe_minimum_education.contains_key(&firm.recipe())
                })
                .try_fold(0_u64, |sum, (firm, _)| {
                    sum.checked_add(next.firm_labor_vacancy(*firm)?)
                        .ok_or(WorldError::ArithmeticOverflow("regional labor vacancies"))
                })?;
            let (offers, offered_wages) = offer_stats.get(&region).copied().unwrap_or_default();
            let hires = u64::try_from(
                matches
                    .iter()
                    .filter(|matched| next.firms[&matched.firm].region() == region)
                    .count(),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("regional labor hires"))?;
            let average_offered_wage = if offers == 0 {
                Money::default()
            } else {
                Money::from_minor_units(
                    i64::try_from(offered_wages / i128::from(offers))
                        .map_err(|_| WorldError::ArithmeticOverflow("average labor offer wage"))?,
                )
            };
            let previous = next.regional_labor_market.get(&region).copied();
            let unemployment_pressure_months = if available_workers > vacancies {
                previous.map_or(1, |row| row.unemployment_pressure_months.saturating_add(1))
            } else {
                0
            };
            let vacancy_pressure_months = if vacancies > available_workers {
                previous.map_or(1, |row| row.vacancy_pressure_months.saturating_add(1))
            } else {
                0
            };
            let observation = RegionalLaborMarketObservation {
                region,
                available_workers,
                vacancies,
                offers,
                hires,
                average_offered_wage,
                unemployment_pressure_months,
                vacancy_pressure_months,
            };
            next.regional_labor_market.insert(region, observation);
            next.events.append(
                next.date,
                crate::DomainEvent::RegionalLaborMarketObserved {
                    region,
                    available_workers,
                    vacancies,
                    offers,
                    hires,
                    average_offered_wage,
                    unemployment_pressure_months,
                    vacancy_pressure_months,
                },
            );
        }
        next.last_labor_market_date = Some(next.date);
        next.events.append(
            next.date,
            crate::DomainEvent::ObservedLaborMatchingCompleted {
                offers: u64::try_from(offers_count)
                    .map_err(|_| WorldError::ArithmeticOverflow("labor offers"))?,
                matches: u64::try_from(matches.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("labor matches"))?,
            },
        );
        *self = next;
        Ok(matches)
    }

    /// Applies one qualified, funded employment match atomically.
    /// # Errors
    /// Returns an error for a stale vacancy, unavailable worker, insufficient education, or wage.
    pub fn match_employment_worker(&mut self, matched: EmploymentMatch) -> Result<(), WorldError> {
        if matched.wage.minor_units() <= 0 || self.firm_labor_vacancy(matched.firm)? == 0 {
            return Err(WorldError::InvalidEmployment(
                "labor match requires a funded vacancy",
            ));
        }
        let firm = &self.firms[&matched.firm];
        if firm.cash().minor_units() < matched.wage.minor_units() {
            return Err(WorldError::InsufficientFirmCash(matched.firm));
        }
        let minimum = self
            .recipe_minimum_education
            .get(&firm.recipe())
            .copied()
            .ok_or(WorldError::InvalidEmployment(
                "labor match requires an explicitly configured recipe labor profile",
            ))?;
        let cohort = self
            .cohorts
            .get(&matched.cohort)
            .ok_or(WorldError::UnknownCohort(matched.cohort))?;
        if cohort.region() != firm.region()
            || cohort.employment() != crate::EmploymentStatus::Unemployed
            || cohort.education() < minimum
            || matched.minimum_education != minimum
            || self.available_cohort_workers(matched.cohort) == 0
        {
            return Err(WorldError::InvalidEmployment(
                "labor match requires a qualified unallocated local worker",
            ));
        }
        let (expected_wage, expected_adjustment) =
            self.competitive_labor_bid(matched.firm, matched.cohort)?;
        if matched.wage != expected_wage
            || matched.labor_market_adjustment_basis_points != expected_adjustment
        {
            return Err(WorldError::InvalidEmployment(
                "labor match wage must equal the current observed competitive bid",
            ));
        }
        let mut next = self.clone();
        let key = (matched.firm, matched.cohort);
        if let Some(agreement) = next.employment_agreements.get_mut(&key) {
            agreement.set_workers(agreement.workers().saturating_add(1));
            agreement.set_wage(matched.wage);
        } else {
            next.register_employment_agreement(EmploymentAgreement::new(
                matched.firm,
                matched.cohort,
                1,
                matched.wage,
            )?)?;
        }
        next.events.append(
            next.date,
            crate::DomainEvent::EmploymentMatched {
                firm: matched.firm,
                cohort: matched.cohort,
                workers: 1,
                wage: matched.wage,
                minimum_education: matched.minimum_education,
                labor_market_adjustment_basis_points: matched.labor_market_adjustment_basis_points,
            },
        );
        *self = next;
        Ok(())
    }

    fn firm_labor_vacancy(&self, firm: FirmId) -> Result<u64, WorldError> {
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let recipe = self
            .production_recipes
            .get(&definition.recipe())
            .ok_or(WorldError::UnknownRecipe(definition.recipe()))?;
        let target = self
            .firm_production_targets
            .get(&firm)
            .copied()
            .unwrap_or(definition.capacity_batches());
        let required = target
            .checked_mul(recipe.labor_milli_worker_months())
            .ok_or(WorldError::ArithmeticOverflow("labor vacancy requirement"))?
            .div_ceil(1_000)
            .min(definition.workers());
        let staffed: u64 = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm && agreement.active())
            .map(EmploymentAgreement::workers)
            .sum();
        Ok(required.saturating_sub(staffed))
    }

    fn available_cohort_workers(&self, cohort: CohortId) -> u64 {
        let Some(definition) = self.cohorts.get(&cohort) else {
            return 0;
        };
        let allocated: u64 = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.cohort() == cohort && agreement.active())
            .map(EmploymentAgreement::workers)
            .sum();
        definition.people().people().saturating_sub(allocated)
    }

    fn select_qualified_unallocated_cohort(
        &self,
        region: crate::RegionId,
        minimum: crate::EducationLevel,
    ) -> Option<CohortId> {
        self.cohorts
            .iter()
            .filter(|(id, cohort)| {
                cohort.region() == region
                    && cohort.employment() == crate::EmploymentStatus::Unemployed
                    && cohort.education() >= minimum
                    && self.available_cohort_workers(**id) > 0
            })
            .max_by_key(|(id, cohort)| (cohort.education(), std::cmp::Reverse(**id)))
            .map(|(id, _)| *id)
    }

    fn competitive_labor_bid(
        &self,
        firm: FirmId,
        cohort: CohortId,
    ) -> Result<(Money, i16), WorldError> {
        let definition = &self.firms[&firm];
        let target = self
            .firm_production_targets
            .get(&firm)
            .copied()
            .unwrap_or(definition.capacity_batches());
        let capacity = definition.capacity_batches().max(1);
        let urgency = 500_u64.saturating_add(target.saturating_mul(1_500) / capacity);
        let pressure = self.regional_labor_market.get(&definition.region());
        let labor_market_adjustment_basis_points = pressure.map_or(0_i16, |row| {
            if row.vacancy_pressure_months >= 3 {
                i16::from((row.vacancy_pressure_months / 3).min(4)) * 500
            } else if row.unemployment_pressure_months >= 3 {
                -(i16::from((row.unemployment_pressure_months / 3).min(4)) * 250)
            } else {
                0
            }
        });
        let cohort = &self.cohorts[&cohort];
        let people = cohort.people().people().max(1);
        let base = cohort.annual_income().minor_units()
            / i64::try_from(people)
                .map_err(|_| WorldError::ArithmeticOverflow("labor bid population"))?
            / 12;
        let multiplier = i128::from(10_000_u64.saturating_add(urgency))
            .checked_add(i128::from(labor_market_adjustment_basis_points))
            .ok_or(WorldError::ArithmeticOverflow("labor bid pressure"))?;
        let numerator = i128::from(base.max(1))
            .checked_mul(multiplier)
            .ok_or(WorldError::ArithmeticOverflow("competitive labor bid"))?;
        let wage = (numerator + 9_999) / 10_000;
        Ok((
            Money::from_minor_units(
                i64::try_from(wage)
                    .map_err(|_| WorldError::ArithmeticOverflow("competitive labor bid"))?,
            ),
            labor_market_adjustment_basis_points,
        ))
    }
}

impl World {
    /// Registers a local employment agreement within firm and cohort worker limits.
    /// # Errors
    /// Returns an error for unknown references, region mismatch, duplicate pair, or over-allocation.
    pub fn register_employment_agreement(
        &mut self,
        agreement: EmploymentAgreement,
    ) -> Result<(), WorldError> {
        let firm = self
            .firms()
            .get(&agreement.firm())
            .ok_or(WorldError::UnknownFirm(agreement.firm()))?;
        let cohort = self
            .cohorts
            .get(&agreement.cohort())
            .ok_or(WorldError::UnknownCohort(agreement.cohort()))?;
        if firm.region() != cohort.region() {
            return Err(WorldError::InvalidEmployment(
                "firm and cohort must share a region",
            ));
        }
        let key = (agreement.firm(), agreement.cohort());
        if self.employment_agreements.contains_key(&key) {
            return Err(WorldError::InvalidEmployment(
                "duplicate employment agreement",
            ));
        }
        let firm_allocated: u64 = self
            .employment_agreements
            .values()
            .filter(|a| a.firm() == agreement.firm())
            .map(EmploymentAgreement::workers)
            .sum();
        let cohort_allocated: u64 = self
            .employment_agreements
            .values()
            .filter(|a| a.cohort() == agreement.cohort())
            .map(EmploymentAgreement::workers)
            .sum();
        if firm_allocated + agreement.workers() > firm.workers()
            || cohort_allocated + agreement.workers() > cohort.people().people()
        {
            return Err(WorldError::InvalidEmployment(
                "employment exceeds workers or cohort population",
            ));
        }
        self.employment_agreements.insert(key, agreement);
        Ok(())
    }
    /// Changes staffed workers in an existing agreement; zero workers terminates active work while preserving arrears.
    /// # Errors
    /// Returns an error for unknown references or worker over-allocation.
    pub fn change_employment_workers(
        &mut self,
        firm: FirmId,
        cohort: CohortId,
        workers: u64,
    ) -> Result<(), WorldError> {
        let key = (firm, cohort);
        if workers > 0 && self.is_firm_insolvent(firm) {
            return Err(WorldError::InvalidFirmReorganization(
                "insolvent employment can resume only through an approved reorganization",
            ));
        }
        let definition = self
            .firms()
            .get(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?;
        let population = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?
            .people()
            .people();
        let firm_other: u64 = self
            .employment_agreements
            .iter()
            .filter(|(key, row)| key.0 == firm && key.1 != cohort && row.active())
            .map(|(_, row)| row.workers())
            .sum();
        let cohort_other: u64 = self
            .employment_agreements
            .iter()
            .filter(|(key, row)| key.1 == cohort && key.0 != firm && row.active())
            .map(|(_, row)| row.workers())
            .sum();
        if firm_other + workers > definition.workers() || cohort_other + workers > population {
            return Err(WorldError::InvalidEmployment(
                "worker change exceeds firm or cohort limit",
            ));
        }
        let agreement =
            self.employment_agreements
                .get_mut(&key)
                .ok_or(WorldError::InvalidEmployment(
                    "employment agreement is missing",
                ))?;
        let previous = agreement.workers();
        agreement.set_workers(workers);
        self.events.append(
            self.date,
            crate::DomainEvent::EmploymentChanged {
                firm,
                cohort,
                previous_workers: previous,
                current_workers: workers,
            },
        );
        Ok(())
    }
    /// Pays wages from firm cash to cohort wealth; unpaid obligations become arrears.
    /// # Errors
    /// Returns an error on arithmetic overflow.
    pub fn execute_monthly_payroll(&mut self) -> Result<Vec<PayrollRecord>, WorldError> {
        if self.last_payroll_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "payroll",
                date: self.date,
            });
        }
        let mut firms = self.firms.clone();
        let mut cohorts = self.cohorts.clone();
        let mut agreements = self.employment_agreements.clone();
        let mut records = Vec::new();
        for agreement in agreements
            .values_mut()
            .filter(|agreement| agreement.active || agreement.arrears().minor_units() > 0)
        {
            let current =
                i128::from(agreement.wage().minor_units()) * i128::from(agreement.workers());
            let current = i64::try_from(current)
                .map_err(|_| WorldError::ArithmeticOverflow("payroll owed"))?;
            let owed = current
                .checked_add(agreement.arrears().minor_units())
                .ok_or(WorldError::ArithmeticOverflow("payroll arrears"))?;
            let firm = firms
                .get_mut(&agreement.firm())
                .ok_or(WorldError::UnknownFirm(agreement.firm()))?;
            let paid = firm.cash().minor_units().min(owed);
            firm.debit_cash(Money::from_minor_units(paid))?;
            cohorts
                .get_mut(&agreement.cohort())
                .ok_or(WorldError::UnknownCohort(agreement.cohort()))?
                .credit_wealth(Money::from_minor_units(paid))?;
            agreement.arrears = Money::from_minor_units(owed - paid);
            records.push(PayrollRecord {
                firm: agreement.firm(),
                cohort: agreement.cohort(),
                owed: Money::from_minor_units(owed),
                paid: Money::from_minor_units(paid),
                arrears: agreement.arrears(),
            });
        }
        self.firms = firms;
        self.cohorts = cohorts;
        self.employment_agreements = agreements;
        for row in &records {
            self.record_firm_payroll(row.firm, row.owed, row.paid, row.arrears)?;
        }
        self.last_payroll_date = Some(self.date);
        for row in &records {
            self.events.append(
                self.date,
                crate::DomainEvent::PayrollSettled {
                    firm: row.firm,
                    cohort: row.cohort,
                    owed: row.owed,
                    paid: row.paid,
                    arrears: row.arrears,
                },
            );
        }
        Ok(records)
    }
    #[must_use]
    pub fn employment_agreements(&self) -> &BTreeMap<(FirmId, CohortId), EmploymentAgreement> {
        &self.employment_agreements
    }

    #[must_use]
    pub fn regional_labor_market(
        &self,
    ) -> &BTreeMap<crate::RegionId, RegionalLaborMarketObservation> {
        &self.regional_labor_market
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
        EducationLevel, EmploymentStatus, Firm, Good, GoodId, HouseholdCohort, HouseholdType,
        NeedProfileId, NeedTier, Population, ProductionRecipe, QuantityMilli, RecipeId, Region,
        RegionId, SimDate, WorldCommand, WorldSeed,
    };

    fn labor_market_world() -> World {
        let mut world = World::new(WorldSeed::new(77), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country registration");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "North March",
                    Population::new(1),
                    Money::from_minor_units(1_200),
                )
                .expect("region"),
            )
            .expect("region registration");
        world
            .register_good(Good::new(GoodId::new(1), "grain").expect("good"))
            .expect("good registration");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "survival",
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
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(1),
                    1,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Unemployed,
                    Money::from_minor_units(1_200),
                    Money::default(),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort registration");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "grain workshop",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe registration");
        world
            .set_recipe_minimum_education(RecipeId::new(1), EducationLevel::Basic)
            .expect("explicit labor profile");
        for (id, capacity) in [(1, 1), (2, 2)] {
            world
                .register_firm(
                    Firm::new(
                        FirmId::new(id),
                        format!("Works {id}"),
                        RegionId::new(1),
                        RecipeId::new(1),
                        1,
                        capacity,
                        Money::from_minor_units(1_000),
                        BTreeMap::new(),
                    )
                    .expect("firm"),
                )
                .expect("firm registration");
            world.firm_production_targets.insert(FirmId::new(id), 1);
        }
        world
    }

    #[test]
    fn scarce_qualified_worker_accepts_highest_ranked_wage_offer_and_replays() {
        let mut direct = labor_market_world();
        let mut replayed = direct.clone();
        WorldCommand::ExecuteObservedLaborMatching
            .apply(&mut direct)
            .expect("labor matching");
        WorldCommand::ExecuteObservedLaborMatching
            .apply(&mut replayed)
            .expect("replayed labor matching");
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        let agreement = &direct.employment_agreements()[&(FirmId::new(1), CohortId::new(1))];
        assert_eq!(agreement.workers(), 1);
        assert_eq!(agreement.wage(), Money::from_minor_units(120));
        let observation = direct.regional_labor_market()[&RegionId::new(1)];
        assert_eq!(observation.available_workers, 0);
        assert_eq!(observation.vacancies, 1);
        assert_eq!(observation.offers, 2);
        assert_eq!(observation.hires, 1);
        assert_eq!(
            observation.average_offered_wage,
            Money::from_minor_units(116)
        );
        assert_eq!(observation.unemployment_pressure_months, 0);
        assert_eq!(observation.vacancy_pressure_months, 1);
        assert!(
            !direct
                .employment_agreements()
                .contains_key(&(FirmId::new(2), CohortId::new(1)))
        );
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::EmploymentMatched {
                firm,
                cohort,
                wage,
                minimum_education: EducationLevel::Basic,
                ..
            } if *firm == FirmId::new(1)
                && *cohort == CohortId::new(1)
                && *wage == Money::from_minor_units(120)
        )));
    }

    #[test]
    fn regional_labor_evidence_retains_persistent_unemployment_pressure() {
        let mut world = labor_market_world();
        world.firm_production_targets.insert(FirmId::new(1), 0);
        world.firm_production_targets.insert(FirmId::new(2), 0);

        world
            .execute_observed_labor_matching()
            .expect("first labor observation");
        let first = world.regional_labor_market()[&RegionId::new(1)];
        assert_eq!(first.available_workers, 1);
        assert_eq!(first.vacancies, 0);
        assert_eq!(first.unemployment_pressure_months, 1);
        assert_eq!(first.vacancy_pressure_months, 0);

        world.advance_month().expect("next month");
        world
            .execute_observed_labor_matching()
            .expect("second labor observation");
        let second = world.regional_labor_market()[&RegionId::new(1)];
        assert_eq!(second.unemployment_pressure_months, 2);
        assert!(world.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::RegionalLaborMarketObserved {
                region,
                available_workers: 1,
                vacancies: 0,
                unemployment_pressure_months: 2,
                ..
            } if *region == RegionId::new(1)
        )));
    }

    #[test]
    fn persistent_regional_pressure_adapts_funded_wage_bids_boundedly() {
        let mut scarcity = labor_market_world();
        scarcity.regional_labor_market.insert(
            RegionId::new(1),
            RegionalLaborMarketObservation {
                region: RegionId::new(1),
                available_workers: 0,
                vacancies: 1,
                offers: 0,
                hires: 0,
                average_offered_wage: Money::default(),
                unemployment_pressure_months: 0,
                vacancy_pressure_months: 3,
            },
        );
        let scarcity_matches = scarcity
            .execute_observed_labor_matching()
            .expect("scarcity-adjusted matching");
        assert_eq!(scarcity_matches[0].wage, Money::from_minor_units(125));
        assert_eq!(
            scarcity_matches[0].labor_market_adjustment_basis_points,
            500
        );

        let mut surplus = labor_market_world();
        surplus.regional_labor_market.insert(
            RegionId::new(1),
            RegionalLaborMarketObservation {
                region: RegionId::new(1),
                available_workers: 1,
                vacancies: 0,
                offers: 0,
                hires: 0,
                average_offered_wage: Money::default(),
                unemployment_pressure_months: 3,
                vacancy_pressure_months: 0,
            },
        );
        let surplus_matches = surplus
            .execute_observed_labor_matching()
            .expect("surplus-adjusted matching");
        assert_eq!(surplus_matches[0].wage, Money::from_minor_units(118));
        assert_eq!(
            surplus_matches[0].labor_market_adjustment_basis_points,
            -250
        );
    }

    #[test]
    fn education_requirement_blocks_unqualified_worker_without_mutation() {
        let mut world = labor_market_world();
        world
            .set_recipe_minimum_education(RecipeId::new(1), EducationLevel::Vocational)
            .expect("education requirement");
        let before = world.clone();
        let matches = world
            .execute_observed_labor_matching()
            .expect("labor matching");
        assert!(matches.is_empty());
        assert!(world.employment_agreements().is_empty());
        assert_eq!(world.firms(), before.firms());
    }
}
