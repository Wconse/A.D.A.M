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
    months_at_current_firm: u8,
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
            months_at_current_firm: 0,
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
    #[must_use]
    pub const fn months_at_current_firm(&self) -> u8 {
        self.months_at_current_firm
    }
    pub(crate) fn set_workers(&mut self, workers: u64) {
        if self.workers == 0 && workers > 0 {
            self.months_at_current_firm = 0;
        }
        self.workers = workers;
        self.active = workers > 0;
    }

    pub(crate) fn advance_tenure(&mut self) {
        if self.active {
            self.months_at_current_firm = self.months_at_current_firm.saturating_add(1);
        }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmploymentSwitch {
    pub from_firm: FirmId,
    pub to_firm: FirmId,
    pub cohort: CohortId,
    pub previous_wage: Money,
    pub offered_wage: Money,
    pub minimum_education: crate::EducationLevel,
    pub labor_market_adjustment_basis_points: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmploymentRetention {
    pub firm: FirmId,
    pub competing_firm: FirmId,
    pub cohort: CohortId,
    pub previous_wage: Money,
    pub retained_wage: Money,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalSkillLaborMarketObservation {
    pub region: crate::RegionId,
    pub minimum_education: crate::EducationLevel,
    pub qualified_available_workers: u64,
    pub vacancies: u64,
    pub unemployment_pressure_months: u8,
    pub vacancy_pressure_months: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegionalOccupationLaborMarketObservation {
    pub region: crate::RegionId,
    pub skill: crate::SkillId,
    pub qualified_available_workers: u64,
    pub vacancies: u64,
    pub unemployment_pressure_months: u8,
    pub vacancy_pressure_months: u8,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum TrainingSponsor {
    Household,
    Firm(FirmId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkforceTraining {
    pub cohort: CohortId,
    pub previous_education: crate::EducationLevel,
    pub target_education: crate::EducationLevel,
    pub months_remaining: u8,
    pub tuition_paid: Money,
    pub sponsor: TrainingSponsor,
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

    /// Configures an optional occupation-specific skill for one recipe.
    /// # Errors
    /// Returns an unknown-recipe error without mutation.
    pub fn set_recipe_required_skill(
        &mut self,
        recipe: crate::RecipeId,
        skill: crate::SkillId,
    ) -> Result<(), WorldError> {
        if !self.production_recipes.contains_key(&recipe) {
            return Err(WorldError::UnknownRecipe(recipe));
        }
        self.recipe_required_skill.insert(recipe, skill);
        Ok(())
    }

    /// Records bounded proficiency for one cohort and content-defined skill.
    /// # Errors
    /// Rejects unknown cohorts and zero proficiency.
    pub fn set_cohort_skill(
        &mut self,
        cohort: CohortId,
        skill: crate::SkillId,
        proficiency: u8,
    ) -> Result<(), WorldError> {
        if !self.cohorts.contains_key(&cohort) {
            return Err(WorldError::UnknownCohort(cohort));
        }
        if proficiency == 0 {
            return Err(WorldError::InvalidEmployment(
                "cohort skill proficiency must be positive",
            ));
        }
        self.cohort_skills.insert((cohort, skill), proficiency);
        Ok(())
    }

    fn cohort_has_recipe_skill(&self, cohort: CohortId, recipe: crate::RecipeId) -> bool {
        self.recipe_required_skill.get(&recipe).is_none_or(|skill| {
            self.cohort_skills
                .get(&(cohort, *skill))
                .is_some_and(|proficiency| *proficiency > 0)
        })
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
            let Some(cohort) = self.select_qualified_unallocated_cohort(
                definition.region(),
                definition.recipe(),
                minimum,
            ) else {
                continue;
            };
            let (wage, labor_market_adjustment_basis_points) =
                self.competitive_labor_bid(firm, cohort)?;
            // A firm walks away from a bid worth more than the work it buys.
            // The vacancy stays open and the labor market keeps its pressure,
            // because the firm, not the world, would pay for the bad hire.
            if self
                .labor_value_ceiling(firm)?
                .is_some_and(|ceiling| wage.minor_units() > ceiling.minor_units())
            {
                continue;
            }
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
        next.progress_workforce_training()?;
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
        let (switches, retentions) =
            next.execute_observed_job_switches(&matches, &mut offer_stats)?;
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
                .filter(|(cohort_id, cohort)| {
                    cohort.region() == region
                        && cohort.employment() == crate::EmploymentStatus::Unemployed
                        && !next.workforce_training.contains_key(cohort_id)
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
                    .count()
                    .saturating_add(
                        switches
                            .iter()
                            .filter(|switched| next.firms[&switched.to_firm].region() == region)
                            .count(),
                    ),
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
            let skill_levels: BTreeSet<_> = next
                .firms
                .values()
                .filter(|firm| firm.region() == region)
                .filter_map(|firm| next.recipe_minimum_education.get(&firm.recipe()).copied())
                .collect();
            for minimum_education in skill_levels {
                let qualified_available_workers = next
                    .cohorts
                    .iter()
                    .filter(|(cohort_id, cohort)| {
                        cohort.region() == region
                            && cohort.employment() == crate::EmploymentStatus::Unemployed
                            && cohort.education() >= minimum_education
                            && !next.workforce_training.contains_key(cohort_id)
                    })
                    .map(|(cohort, _)| next.available_cohort_workers(*cohort))
                    .sum();
                let skill_vacancies = next
                    .firms
                    .iter()
                    .filter(|(_, firm)| {
                        firm.region() == region
                            && next
                                .recipe_minimum_education
                                .get(&firm.recipe())
                                .is_some_and(|level| *level == minimum_education)
                    })
                    .try_fold(0_u64, |sum, (firm, _)| {
                        sum.checked_add(next.firm_labor_vacancy(*firm)?)
                            .ok_or(WorldError::ArithmeticOverflow("skill labor vacancies"))
                    })?;
                let key = (region, minimum_education);
                let previous = next.regional_skill_labor_market.get(&key).copied();
                let skill_unemployment_pressure_months =
                    if qualified_available_workers > skill_vacancies {
                        previous.map_or(1, |row| row.unemployment_pressure_months.saturating_add(1))
                    } else {
                        0
                    };
                let skill_vacancy_pressure_months = if skill_vacancies > qualified_available_workers
                {
                    previous.map_or(1, |row| row.vacancy_pressure_months.saturating_add(1))
                } else {
                    0
                };
                let skill_observation = RegionalSkillLaborMarketObservation {
                    region,
                    minimum_education,
                    qualified_available_workers,
                    vacancies: skill_vacancies,
                    unemployment_pressure_months: skill_unemployment_pressure_months,
                    vacancy_pressure_months: skill_vacancy_pressure_months,
                };
                next.regional_skill_labor_market
                    .insert(key, skill_observation);
                next.events.append(
                    next.date,
                    crate::DomainEvent::RegionalSkillLaborMarketObserved {
                        region,
                        minimum_education,
                        qualified_available_workers,
                        vacancies: skill_vacancies,
                        unemployment_pressure_months: skill_unemployment_pressure_months,
                        vacancy_pressure_months: skill_vacancy_pressure_months,
                    },
                );
            }
        }
        next.enroll_observed_workforce_training()?;
        for agreement in next.employment_agreements.values_mut() {
            agreement.advance_tenure();
        }
        next.last_labor_market_date = Some(next.date);
        next.events.append(
            next.date,
            crate::DomainEvent::ObservedLaborMatchingCompleted {
                offers: u64::try_from(
                    offers_count
                        .saturating_add(switches.len())
                        .saturating_add(retentions.len()),
                )
                .map_err(|_| WorldError::ArithmeticOverflow("labor offers"))?,
                matches: u64::try_from(matches.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("labor matches"))?,
                switches: u64::try_from(switches.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("labor switches"))?,
                retentions: u64::try_from(retentions.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("labor retentions"))?,
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
            || !self.cohort_has_recipe_skill(matched.cohort, firm.recipe())
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

    fn progress_workforce_training(&mut self) -> Result<(), WorldError> {
        let cohorts: Vec<_> = self.workforce_training.keys().copied().collect();
        for cohort in cohorts {
            let training = self.workforce_training[&cohort];
            if training.months_remaining > 1 {
                self.workforce_training
                    .get_mut(&cohort)
                    .ok_or(WorldError::UnknownCohort(cohort))?
                    .months_remaining -= 1;
                continue;
            }
            self.cohorts
                .get_mut(&cohort)
                .ok_or(WorldError::UnknownCohort(cohort))?
                .set_education(training.target_education);
            self.workforce_training.remove(&cohort);
            self.events.append(
                self.date,
                crate::DomainEvent::WorkforceTrainingCompleted {
                    cohort,
                    previous_education: training.previous_education,
                    new_education: training.target_education,
                },
            );
        }
        Ok(())
    }

    fn partition_training_cohort(&mut self, source: CohortId) -> Result<CohortId, WorldError> {
        let definition = self
            .cohorts
            .get(&source)
            .ok_or(WorldError::UnknownCohort(source))?;
        if definition.people().people() <= 1 || definition.households() <= 1 {
            return Ok(source);
        }
        let next_id = self
            .cohorts
            .keys()
            .next_back()
            .map_or(1_u32, |id| id.get().saturating_add(1));
        if next_id == u32::MAX && self.cohorts.contains_key(&CohortId::new(next_id)) {
            return Err(WorldError::ArithmeticOverflow("training cohort identity"));
        }
        let training_id = CohortId::new(next_id);
        let training = self
            .cohorts
            .get_mut(&source)
            .ok_or(WorldError::UnknownCohort(source))?
            .split_training_household(training_id)?;
        self.events.append(
            self.date,
            crate::DomainEvent::HouseholdCohortSplitForTraining {
                source_cohort: source,
                training_cohort: training_id,
                people: training.people().people(),
                households: training.households(),
                annual_income: training.annual_income(),
                liquid_wealth: training.liquid_wealth(),
                debt: training.debt(),
            },
        );
        self.cohorts.insert(training_id, training);
        Ok(training_id)
    }

    fn select_training_sponsor(
        &self,
        shortage: RegionalSkillLaborMarketObservation,
        tuition: Money,
    ) -> Result<Option<FirmId>, WorldError> {
        for (firm_id, firm) in &self.firms {
            if firm.region() != shortage.region
                || self
                    .recipe_minimum_education
                    .get(&firm.recipe())
                    .is_none_or(|level| *level != shortage.minimum_education)
                || self.firm_labor_vacancy(*firm_id)? == 0
            {
                continue;
            }
            if self.training_sponsorship_is_forecast_solvent(*firm_id, tuition)? {
                return Ok(Some(*firm_id));
            }
        }
        Ok(None)
    }

    fn training_sponsorship_is_forecast_solvent(
        &self,
        firm: FirmId,
        tuition: Money,
    ) -> Result<bool, WorldError> {
        if self.is_firm_insolvent(firm) {
            return Ok(false);
        }
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let Some(expectations) = self.firm_expectations.get(&firm).copied() else {
            return Ok(false);
        };
        if expectations.horizon_months() < 3
            || definition.cash().minor_units() < tuition.minor_units()
            || self
                .employment_agreements
                .values()
                .any(|agreement| agreement.firm() == firm && agreement.arrears().minor_units() > 0)
        {
            return Ok(false);
        }
        let payroll = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm && agreement.active())
            .try_fold(0_i128, |sum, agreement| {
                sum.checked_add(
                    i128::from(agreement.wage().minor_units()) * i128::from(agreement.workers()),
                )
                .ok_or(WorldError::ArithmeticOverflow("training sponsor payroll"))
            })?
            .checked_mul(i128::from(expectations.horizon_months()))
            .ok_or(WorldError::ArithmeticOverflow(
                "training sponsor payroll horizon",
            ))?;
        let available_after_operations = i128::from(definition.cash().minor_units())
            .checked_add(i128::from(
                expectations.expected_sales_revenue().minor_units(),
            ))
            .and_then(|value| {
                value.checked_add(i128::from(expectations.expected_financing().minor_units()))
            })
            .and_then(|value| {
                value.checked_sub(i128::from(
                    expectations.expected_input_costs().minor_units(),
                ))
            })
            .and_then(|value| value.checked_sub(payroll))
            .ok_or(WorldError::ArithmeticOverflow(
                "training sponsor available cash",
            ))?;
        Ok(available_after_operations >= i128::from(tuition.minor_units()))
    }

    #[allow(clippy::too_many_lines)]
    fn enroll_observed_workforce_training(&mut self) -> Result<(), WorldError> {
        let shortages: Vec<_> = self
            .regional_skill_labor_market
            .values()
            .copied()
            .filter(|row| row.vacancy_pressure_months >= 6 && row.vacancies > 0)
            .collect();
        let mut enrolled_regions = BTreeSet::new();
        for shortage in shortages {
            if enrolled_regions.contains(&shortage.region) {
                continue;
            }
            let mut candidates = Vec::new();
            for (cohort_id, cohort) in &self.cohorts {
                if cohort.region() != shortage.region
                    || cohort.employment() != crate::EmploymentStatus::Unemployed
                    || cohort.education() >= shortage.minimum_education
                    || self.workforce_training.contains_key(cohort_id)
                    || self.available_cohort_workers(*cohort_id) == 0
                {
                    continue;
                }
                let Some(target_education) = Self::next_education_level(cohort.education()) else {
                    continue;
                };
                if target_education > shortage.minimum_education {
                    continue;
                }
                let people = cohort.people().people();
                let participants = if cohort.households() > 1 {
                    people.div_ceil(cohort.households())
                } else {
                    people
                };
                let participant_income = i128::from(cohort.annual_income().minor_units())
                    .checked_mul(i128::from(participants))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "training participant income",
                    ))?
                    / i128::from(people.max(1));
                let participant_wealth = i128::from(cohort.liquid_wealth().minor_units())
                    .checked_mul(i128::from(participants))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "training participant wealth",
                    ))?
                    / i128::from(people.max(1));
                let tuition = Money::from_minor_units(
                    i64::try_from((participant_income / 12).max(1))
                        .map_err(|_| WorldError::ArithmeticOverflow("training tuition"))?,
                );
                let sponsor = if participant_wealth >= i128::from(tuition.minor_units()) {
                    TrainingSponsor::Household
                } else if let Some(firm) = self.select_training_sponsor(shortage, tuition)? {
                    TrainingSponsor::Firm(firm)
                } else {
                    continue;
                };
                candidates.push((
                    cohort.education(),
                    std::cmp::Reverse(*cohort_id),
                    *cohort_id,
                    target_education,
                    tuition,
                    sponsor,
                ));
            }
            let Some((previous_education, _, cohort, target_education, tuition_paid, sponsor)) =
                candidates.into_iter().max()
            else {
                continue;
            };
            let cohort = self.partition_training_cohort(cohort)?;
            match sponsor {
                TrainingSponsor::Household => self
                    .cohorts
                    .get_mut(&cohort)
                    .ok_or(WorldError::UnknownCohort(cohort))?
                    .debit_wealth(tuition_paid)?,
                TrainingSponsor::Firm(firm) => self
                    .firms
                    .get_mut(&firm)
                    .ok_or(WorldError::UnknownFirm(firm))?
                    .debit_cash(tuition_paid)?,
            }
            self.workforce_training.insert(
                cohort,
                WorkforceTraining {
                    cohort,
                    previous_education,
                    target_education,
                    months_remaining: 3,
                    tuition_paid,
                    sponsor,
                },
            );
            self.events.append(
                self.date,
                crate::DomainEvent::WorkforceTrainingStarted {
                    cohort,
                    previous_education,
                    target_education,
                    months: 3,
                    tuition_paid,
                    sponsoring_firm: match sponsor {
                        TrainingSponsor::Household => None,
                        TrainingSponsor::Firm(firm) => Some(firm),
                    },
                },
            );
            enrolled_regions.insert(shortage.region);
        }
        Ok(())
    }

    const fn next_education_level(
        education: crate::EducationLevel,
    ) -> Option<crate::EducationLevel> {
        match education {
            crate::EducationLevel::None => Some(crate::EducationLevel::Basic),
            crate::EducationLevel::Basic => Some(crate::EducationLevel::Secondary),
            crate::EducationLevel::Secondary => Some(crate::EducationLevel::Vocational),
            crate::EducationLevel::Vocational => Some(crate::EducationLevel::Tertiary),
            crate::EducationLevel::Tertiary => None,
        }
    }

    fn execute_observed_job_switches(
        &mut self,
        matches: &[EmploymentMatch],
        offer_stats: &mut BTreeMap<crate::RegionId, (u64, i128)>,
    ) -> Result<(Vec<EmploymentSwitch>, Vec<EmploymentRetention>), WorldError> {
        let mut touched_firms: BTreeSet<_> = matches.iter().map(|matched| matched.firm).collect();
        let target_firms: Vec<_> = self.firms.keys().copied().collect();
        let mut completed_switches = Vec::new();
        let mut completed_retentions = Vec::new();
        for to_firm in target_firms {
            if touched_firms.contains(&to_firm)
                || self.is_firm_insolvent(to_firm)
                || self.firm_labor_vacancy(to_firm)? == 0
            {
                continue;
            }
            let target = &self.firms[&to_firm];
            let Some(minimum_education) =
                self.recipe_minimum_education.get(&target.recipe()).copied()
            else {
                continue;
            };
            let mut candidates = Vec::new();
            for agreement in self.employment_agreements.values().filter(|agreement| {
                agreement.active()
                    && agreement.workers() > 0
                    && agreement.months_at_current_firm() >= 3
                    && agreement.firm() != to_firm
                    && !touched_firms.contains(&agreement.firm())
            }) {
                let cohort = &self.cohorts[&agreement.cohort()];
                if cohort.region() != target.region()
                    || cohort.education() < minimum_education
                    || !self.cohort_has_recipe_skill(agreement.cohort(), target.recipe())
                {
                    continue;
                }
                let (offered_wage, labor_market_adjustment_basis_points) =
                    self.competitive_labor_bid(to_firm, agreement.cohort())?;
                if target.cash().minor_units() < offered_wage.minor_units()
                    || i128::from(offered_wage.minor_units()) * 100
                        < i128::from(agreement.wage().minor_units()) * 110
                {
                    continue;
                }
                candidates.push(EmploymentSwitch {
                    from_firm: agreement.firm(),
                    to_firm,
                    cohort: agreement.cohort(),
                    previous_wage: agreement.wage(),
                    offered_wage,
                    minimum_education,
                    labor_market_adjustment_basis_points,
                });
            }
            candidates.sort_by(|first, second| {
                second
                    .offered_wage
                    .cmp(&first.offered_wage)
                    .then_with(|| first.previous_wage.cmp(&second.previous_wage))
                    .then_with(|| first.from_firm.cmp(&second.from_firm))
                    .then_with(|| first.cohort.cmp(&second.cohort))
            });
            let Some(switched) = candidates.into_iter().next() else {
                continue;
            };
            let region = target.region();
            let stats = offer_stats.entry(region).or_default();
            stats.0 = stats.0.saturating_add(1);
            stats.1 = stats
                .1
                .checked_add(i128::from(switched.offered_wage.minor_units()))
                .ok_or(WorldError::ArithmeticOverflow(
                    "regional switch offer wages",
                ))?;
            if let Some(retained) = self.plan_employment_retention(switched)? {
                crate::WorldCommand::RetainEmploymentWorker(retained).apply(self)?;
                touched_firms.insert(retained.firm);
                touched_firms.insert(retained.competing_firm);
                completed_retentions.push(retained);
                continue;
            }
            crate::WorldCommand::SwitchEmploymentWorker(switched).apply(self)?;
            touched_firms.insert(switched.from_firm);
            touched_firms.insert(switched.to_firm);
            completed_switches.push(switched);
        }
        Ok((completed_switches, completed_retentions))
    }

    fn plan_employment_retention(
        &self,
        switched: EmploymentSwitch,
    ) -> Result<Option<EmploymentRetention>, WorldError> {
        if !self.retention_is_forecast_solvent(
            switched.from_firm,
            switched.cohort,
            switched.offered_wage,
        )? {
            return Ok(None);
        }
        Ok(Some(EmploymentRetention {
            firm: switched.from_firm,
            competing_firm: switched.to_firm,
            cohort: switched.cohort,
            previous_wage: switched.previous_wage,
            retained_wage: switched.offered_wage,
            minimum_education: switched.minimum_education,
            labor_market_adjustment_basis_points: switched.labor_market_adjustment_basis_points,
        }))
    }

    fn retention_is_forecast_solvent(
        &self,
        firm: FirmId,
        cohort: CohortId,
        retained_wage: Money,
    ) -> Result<bool, WorldError> {
        if self.is_firm_insolvent(firm) {
            return Ok(false);
        }
        let Some(expectations) = self.firm_expectations.get(&firm).copied() else {
            return Ok(false);
        };
        if expectations.horizon_months() < 3
            || self
                .employment_agreements
                .values()
                .any(|agreement| agreement.firm() == firm && agreement.arrears().minor_units() > 0)
        {
            return Ok(false);
        }
        let monthly_payroll = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm && agreement.active())
            .try_fold(0_i128, |sum, agreement| {
                let wage = if agreement.cohort() == cohort {
                    retained_wage
                } else {
                    agreement.wage()
                };
                sum.checked_add(i128::from(wage.minor_units()) * i128::from(agreement.workers()))
                    .ok_or(WorldError::ArithmeticOverflow("retention forecast payroll"))
            })?;
        let obligation = monthly_payroll
            .checked_mul(i128::from(expectations.horizon_months()))
            .ok_or(WorldError::ArithmeticOverflow("retention forecast horizon"))?;
        let cash = self
            .firms
            .get(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?
            .cash();
        let available = i128::from(cash.minor_units())
            .checked_add(i128::from(
                expectations.expected_sales_revenue().minor_units(),
            ))
            .and_then(|value| {
                value.checked_add(i128::from(expectations.expected_financing().minor_units()))
            })
            .and_then(|value| {
                value.checked_sub(i128::from(
                    expectations.expected_input_costs().minor_units(),
                ))
            })
            .ok_or(WorldError::ArithmeticOverflow(
                "retention forecast available cash",
            ))?;
        Ok(available >= obligation)
    }

    /// Retains a worker by matching a materially better competing offer.
    /// # Errors
    /// Rejects stale offers or counteroffers that fail forward payroll coverage.
    pub fn retain_employment_worker(
        &mut self,
        retained: EmploymentRetention,
    ) -> Result<(), WorldError> {
        let source_key = (retained.firm, retained.cohort);
        let agreement =
            self.employment_agreements
                .get(&source_key)
                .ok_or(WorldError::InvalidEmployment(
                    "retention requires an active source agreement",
                ))?;
        let competitor = self
            .firms
            .get(&retained.competing_firm)
            .ok_or(WorldError::UnknownFirm(retained.competing_firm))?;
        let minimum = self
            .recipe_minimum_education
            .get(&competitor.recipe())
            .copied()
            .ok_or(WorldError::InvalidEmployment(
                "retention requires a configured competing vacancy",
            ))?;
        let (expected_wage, expected_adjustment) =
            self.competitive_labor_bid(retained.competing_firm, retained.cohort)?;
        if retained.firm == retained.competing_firm
            || self.firm_labor_vacancy(retained.competing_firm)? == 0
            || !agreement.active()
            || agreement.workers() == 0
            || agreement.months_at_current_firm() < 3
            || agreement.wage() != retained.previous_wage
            || retained.minimum_education != minimum
            || retained.retained_wage != expected_wage
            || retained.labor_market_adjustment_basis_points != expected_adjustment
            || i128::from(retained.retained_wage.minor_units()) * 100
                < i128::from(retained.previous_wage.minor_units()) * 110
            || !self.retention_is_forecast_solvent(
                retained.firm,
                retained.cohort,
                retained.retained_wage,
            )?
        {
            return Err(WorldError::InvalidEmployment(
                "retention requires a current materially better offer and solvent forecast",
            ));
        }
        let mut next = self.clone();
        next.employment_agreements
            .get_mut(&source_key)
            .ok_or(WorldError::InvalidEmployment(
                "validated retention agreement disappeared",
            ))?
            .set_wage(retained.retained_wage);
        next.events.append(
            next.date,
            crate::DomainEvent::EmploymentRetained {
                firm: retained.firm,
                competing_firm: retained.competing_firm,
                cohort: retained.cohort,
                previous_wage: retained.previous_wage,
                retained_wage: retained.retained_wage,
                minimum_education: retained.minimum_education,
                labor_market_adjustment_basis_points: retained.labor_market_adjustment_basis_points,
            },
        );
        *self = next;
        Ok(())
    }

    /// Moves one worker from an existing agreement to a materially better funded offer.
    /// # Errors
    /// Rejects stale, underqualified, underfunded, or sub-threshold switches atomically.
    pub fn switch_employment_worker(
        &mut self,
        switched: EmploymentSwitch,
    ) -> Result<(), WorldError> {
        if switched.from_firm == switched.to_firm || self.firm_labor_vacancy(switched.to_firm)? == 0
        {
            return Err(WorldError::InvalidEmployment(
                "employment switch requires a distinct firm with a vacancy",
            ));
        }
        let source = self
            .firms
            .get(&switched.from_firm)
            .ok_or(WorldError::UnknownFirm(switched.from_firm))?;
        let target = self
            .firms
            .get(&switched.to_firm)
            .ok_or(WorldError::UnknownFirm(switched.to_firm))?;
        let minimum = self
            .recipe_minimum_education
            .get(&target.recipe())
            .copied()
            .ok_or(WorldError::InvalidEmployment(
                "employment switch requires an explicitly configured target recipe",
            ))?;
        let cohort = self
            .cohorts
            .get(&switched.cohort)
            .ok_or(WorldError::UnknownCohort(switched.cohort))?;
        let source_key = (switched.from_firm, switched.cohort);
        let source_agreement =
            self.employment_agreements
                .get(&source_key)
                .ok_or(WorldError::InvalidEmployment(
                    "employment switch source agreement is missing",
                ))?;
        let (expected_wage, expected_adjustment) =
            self.competitive_labor_bid(switched.to_firm, switched.cohort)?;
        if source.region() != target.region()
            || cohort.region() != target.region()
            || cohort.education() < minimum
            || switched.minimum_education != minimum
            || !source_agreement.active()
            || source_agreement.workers() == 0
            || source_agreement.months_at_current_firm() < 3
            || source_agreement.wage() != switched.previous_wage
            || switched.offered_wage != expected_wage
            || switched.labor_market_adjustment_basis_points != expected_adjustment
            || target.cash().minor_units() < switched.offered_wage.minor_units()
            || i128::from(switched.offered_wage.minor_units()) * 100
                < i128::from(switched.previous_wage.minor_units()) * 110
        {
            return Err(WorldError::InvalidEmployment(
                "employment switch requires a qualified worker and a current 10% better funded bid",
            ));
        }
        let mut next = self.clone();
        let source_workers = next.employment_agreements[&source_key].workers();
        next.employment_agreements
            .get_mut(&source_key)
            .ok_or(WorldError::InvalidEmployment(
                "validated employment switch source disappeared",
            ))?
            .set_workers(source_workers - 1);
        let target_key = (switched.to_firm, switched.cohort);
        if let Some(agreement) = next.employment_agreements.get_mut(&target_key) {
            agreement.set_workers(agreement.workers().saturating_add(1));
            agreement.set_wage(switched.offered_wage);
        } else {
            next.register_employment_agreement(EmploymentAgreement::new(
                switched.to_firm,
                switched.cohort,
                1,
                switched.offered_wage,
            )?)?;
        }
        next.events.append(
            next.date,
            crate::DomainEvent::EmploymentSwitched {
                from_firm: switched.from_firm,
                to_firm: switched.to_firm,
                cohort: switched.cohort,
                previous_wage: switched.previous_wage,
                offered_wage: switched.offered_wage,
                minimum_education: switched.minimum_education,
                labor_market_adjustment_basis_points: switched.labor_market_adjustment_basis_points,
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
        recipe: crate::RecipeId,
        minimum: crate::EducationLevel,
    ) -> Option<CohortId> {
        self.cohorts
            .iter()
            .filter(|(id, cohort)| {
                cohort.region() == region
                    && cohort.employment() == crate::EmploymentStatus::Unemployed
                    && cohort.education() >= minimum
                    && self.cohort_has_recipe_skill(**id, recipe)
                    && !self.workforce_training.contains_key(id)
                    && self.available_cohort_workers(**id) > 0
            })
            .max_by_key(|(id, cohort)| (cohort.education(), std::cmp::Reverse(**id)))
            .map(|(id, _)| *id)
    }

    /// The most a firm can pay a worker each month before the hire loses money.
    ///
    /// A worker is worth what the batches they staff can fetch, net of the
    /// materials those batches consume. This is the employer's side of the wage
    /// bargain, and until now it was missing: the bid was built entirely from
    /// what the worker used to earn and how urgently the firm wanted staff, so
    /// nothing stopped a firm from hiring itself into permanent loss.
    ///
    /// Returns `None` when the good or an input has no observed local price, in
    /// which case there is no evidence to judge the hire by.
    fn labor_value_ceiling(&self, firm: FirmId) -> Result<Option<Money>, WorldError> {
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let Some(recipe) = self.production_recipes.get(&definition.recipe()) else {
            return Ok(None);
        };
        let region = definition.region();
        let Some(price) = self
            .regional_prices
            .get(&(region, recipe.output_good()))
            .copied()
        else {
            return Ok(None);
        };
        let revenue =
            crate::firm_entry::quantity_value(price, recipe.output_per_batch())?.minor_units();
        let mut materials = 0_i64;
        for input in recipe.inputs() {
            let Some(input_price) = self.regional_prices.get(&(region, input.good())).copied()
            else {
                return Ok(None);
            };
            materials = materials
                .checked_add(
                    crate::firm_entry::quantity_value(input_price, input.quantity_per_batch())?
                        .minor_units(),
                )
                .ok_or(WorldError::ArithmeticOverflow("labor value materials"))?;
        }
        let margin_per_batch = revenue.saturating_sub(materials);
        if margin_per_batch <= 0 {
            return Ok(Some(Money::default()));
        }
        let labor = recipe.labor_milli_worker_months().max(1);
        let ceiling = i128::from(margin_per_batch)
            .checked_mul(1_000)
            .ok_or(WorldError::ArithmeticOverflow("labor value ceiling"))?
            / i128::from(labor);
        Ok(Some(Money::from_minor_units(
            i64::try_from(ceiling)
                .map_err(|_| WorldError::ArithmeticOverflow("labor value ceiling"))?,
        )))
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
        let pressure = self
            .recipe_minimum_education
            .get(&definition.recipe())
            .and_then(|minimum| {
                self.regional_skill_labor_market
                    .get(&(definition.region(), *minimum))
            });
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

    #[must_use]
    pub fn regional_skill_labor_market(
        &self,
    ) -> &BTreeMap<(crate::RegionId, crate::EducationLevel), RegionalSkillLaborMarketObservation>
    {
        &self.regional_skill_labor_market
    }

    #[must_use]
    pub fn workforce_training(&self) -> &BTreeMap<CohortId, WorkforceTraining> {
        &self.workforce_training
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
        EducationLevel, EmploymentStatus, Firm, FirmExpectationSource, FirmExpectations, Good,
        GoodId, HouseholdCohort, HouseholdType, NeedProfileId, NeedTier, Population,
        ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, SkillId,
        WorldCommand, WorldSeed,
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
    fn a_firm_leaves_a_vacancy_open_when_the_bid_outruns_what_the_work_is_worth() {
        let mut world = labor_market_world();
        // A batch of grain fetches a single minor unit, while the going bid for
        // a worker is built from a cohort earning 1,200 a year.
        world
            .set_regional_price(RegionId::new(1), GoodId::new(1), Money::from_minor_units(1))
            .expect("observed price");
        let matches = world
            .execute_observed_labor_matching()
            .expect("labor matching");
        assert!(
            matches.is_empty(),
            "no firm should hire a worker who costs more than the work they produce"
        );
        assert_eq!(
            world.cohorts[&CohortId::new(1)].employment(),
            crate::EmploymentStatus::Unemployed,
            "the worker stays unemployed rather than being hired into a loss"
        );
    }

    #[test]
    fn the_same_worker_is_hired_once_the_work_covers_the_wage() {
        let mut world = labor_market_world();
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(3_000),
            )
            .expect("observed price");
        let matches = world
            .execute_observed_labor_matching()
            .expect("labor matching");
        assert_eq!(
            matches.len(),
            1,
            "a worker worth more than the bid must still be hired"
        );
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
    fn worker_switches_to_materially_better_offer_and_replays() {
        let mut direct = labor_market_world();
        direct
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(1),
                    CohortId::new(1),
                    1,
                    Money::from_minor_units(100),
                )
                .expect("source agreement"),
            )
            .expect("register source agreement");
        for _ in 0..3 {
            let early_matches = direct
                .execute_observed_labor_matching()
                .expect("tenure-building labor market");
            assert!(early_matches.is_empty());
            assert!(
                !direct
                    .employment_agreements()
                    .contains_key(&(FirmId::new(2), CohortId::new(1)))
            );
            direct.advance_month().expect("tenure month");
        }
        let mut replayed = direct.clone();

        let matches = direct
            .execute_observed_labor_matching()
            .expect("labor market with switching");
        WorldCommand::ExecuteObservedLaborMatching
            .apply(&mut replayed)
            .expect("replayed switching market");

        assert!(matches.is_empty());
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        assert_eq!(
            direct.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            0
        );
        let destination = &direct.employment_agreements()[&(FirmId::new(2), CohortId::new(1))];
        assert_eq!(destination.workers(), 1);
        assert_eq!(destination.wage(), Money::from_minor_units(118));
        assert_eq!(destination.months_at_current_firm(), 1);
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::EmploymentSwitched {
                from_firm,
                to_firm,
                cohort,
                previous_wage,
                offered_wage,
                ..
            } if *from_firm == FirmId::new(1)
                && *to_firm == FirmId::new(2)
                && *cohort == CohortId::new(1)
                && *previous_wage == Money::from_minor_units(100)
                && *offered_wage == Money::from_minor_units(118)
        )));
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::ObservedLaborMatchingCompleted {
                matches: 0,
                switches: 1,
                ..
            }
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
        scarcity.regional_skill_labor_market.insert(
            (RegionId::new(1), EducationLevel::Basic),
            RegionalSkillLaborMarketObservation {
                region: RegionId::new(1),
                minimum_education: EducationLevel::Basic,
                qualified_available_workers: 0,
                vacancies: 1,
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
        surplus.regional_skill_labor_market.insert(
            (RegionId::new(1), EducationLevel::Basic),
            RegionalSkillLaborMarketObservation {
                region: RegionId::new(1),
                minimum_education: EducationLevel::Basic,
                qualified_available_workers: 1,
                vacancies: 0,
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
    fn persistent_skill_shortage_funds_timed_training_and_replays() {
        let mut direct = labor_market_world();
        direct
            .set_recipe_minimum_education(RecipeId::new(1), EducationLevel::Vocational)
            .expect("vocational profile");
        direct
            .cohorts
            .get_mut(&CohortId::new(1))
            .expect("training cohort")
            .credit_wealth(Money::from_minor_units(100))
            .expect("training savings");
        direct.regional_skill_labor_market.insert(
            (RegionId::new(1), EducationLevel::Vocational),
            RegionalSkillLaborMarketObservation {
                region: RegionId::new(1),
                minimum_education: EducationLevel::Vocational,
                qualified_available_workers: 0,
                vacancies: 2,
                unemployment_pressure_months: 0,
                vacancy_pressure_months: 6,
            },
        );
        let mut replayed = direct.clone();

        for month in 0..4 {
            WorldCommand::ExecuteObservedLaborMatching
                .apply(&mut direct)
                .expect("direct training month");
            WorldCommand::ExecuteObservedLaborMatching
                .apply(&mut replayed)
                .expect("replayed training month");
            if month < 3 {
                direct.advance_month().expect("direct training calendar");
                replayed
                    .advance_month()
                    .expect("replayed training calendar");
            }
        }

        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(direct.workforce_training().is_empty());
        let cohort = &direct.cohorts[&CohortId::new(1)];
        assert_eq!(cohort.education(), EducationLevel::Vocational);
        assert_eq!(cohort.liquid_wealth(), Money::default());
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::WorkforceTrainingStarted {
                cohort,
                previous_education: EducationLevel::Secondary,
                target_education: EducationLevel::Vocational,
                months: 3,
                tuition_paid,
                sponsoring_firm: None,
            } if *cohort == CohortId::new(1)
                && *tuition_paid == Money::from_minor_units(100)
        )));
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            crate::DomainEvent::WorkforceTrainingCompleted {
                cohort,
                previous_education: EducationLevel::Secondary,
                new_education: EducationLevel::Vocational,
            } if *cohort == CohortId::new(1)
        )));
        let skill_row = direct
            .regional_skill_labor_market()
            .get(&(RegionId::new(1), EducationLevel::Vocational))
            .expect("vocational evidence");
        assert!(skill_row.vacancy_pressure_months >= 6);
    }

    #[test]
    fn solvent_incumbent_matches_competing_offer() {
        let mut world = labor_market_world();
        world
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(1),
                    CohortId::new(1),
                    1,
                    Money::from_minor_units(100),
                )
                .expect("agreement"),
            )
            .expect("agreement registration");
        for _ in 0..3 {
            world
                .execute_observed_labor_matching()
                .expect("tenure month");
            world.advance_month().expect("calendar month");
        }
        world
            .update_firm_expectations(
                FirmId::new(1),
                FirmExpectations::new(
                    Money::from_minor_units(10_000),
                    Money::default(),
                    Money::default(),
                    3,
                    FirmExpectationSource::Management,
                )
                .expect("expectations"),
            )
            .expect("forecast update");
        world
            .execute_observed_labor_matching()
            .expect("retention market");
        let agreement = &world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))];
        assert_eq!(agreement.wage(), Money::from_minor_units(118));
        assert!(
            !world
                .employment_agreements()
                .contains_key(&(FirmId::new(2), CohortId::new(1)))
        );
        assert!(world.events().events().iter().any(|event| matches!(
            event.event(),
            crate::DomainEvent::EmploymentRetained { firm, retained_wage, .. }
                if *firm == FirmId::new(1)
                    && *retained_wage == Money::from_minor_units(118)
        )));
    }

    #[test]
    fn solvent_firm_sponsors_unfunded_training() {
        let mut world = labor_market_world();
        world
            .set_recipe_minimum_education(RecipeId::new(1), EducationLevel::Vocational)
            .expect("vocational profile");
        world
            .update_firm_expectations(
                FirmId::new(1),
                FirmExpectations::new(
                    Money::from_minor_units(10_000),
                    Money::default(),
                    Money::default(),
                    3,
                    FirmExpectationSource::Management,
                )
                .expect("expectations"),
            )
            .expect("forecast update");
        world.regional_skill_labor_market.insert(
            (RegionId::new(1), EducationLevel::Vocational),
            RegionalSkillLaborMarketObservation {
                region: RegionId::new(1),
                minimum_education: EducationLevel::Vocational,
                qualified_available_workers: 0,
                vacancies: 2,
                unemployment_pressure_months: 0,
                vacancy_pressure_months: 6,
            },
        );
        world
            .execute_observed_labor_matching()
            .expect("sponsored training");
        let training = world.workforce_training()[&CohortId::new(1)];
        assert_eq!(training.sponsor, TrainingSponsor::Firm(FirmId::new(1)));
        assert_eq!(
            world.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(900)
        );
    }

    #[test]
    fn occupation_skill_is_required_in_addition_to_education() {
        let mut unskilled = labor_market_world();
        unskilled
            .set_recipe_required_skill(RecipeId::new(1), SkillId::new(7))
            .expect("skill profile");
        assert!(
            unskilled
                .execute_observed_labor_matching()
                .expect("unskilled market")
                .is_empty()
        );

        let mut skilled = labor_market_world();
        skilled
            .set_recipe_required_skill(RecipeId::new(1), SkillId::new(7))
            .expect("skill profile");
        skilled
            .set_cohort_skill(CohortId::new(1), SkillId::new(7), 1)
            .expect("cohort skill");
        assert_eq!(
            skilled
                .execute_observed_labor_matching()
                .expect("skilled market")
                .len(),
            1
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
