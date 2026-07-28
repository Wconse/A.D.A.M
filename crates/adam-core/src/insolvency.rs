use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ActorId, CohortId, DomainEvent, FirmId, GoodId, Money, QuantityMilli, SimDate, World,
    WorldError,
};

/// Immutable asset-and-worker-claim snapshot taken when ordinary firm operations
/// enter insolvency administration.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmInsolvency {
    firm: FirmId,
    administrator: ActorId,
    declared_on: SimDate,
    cash_at_declaration: Money,
    wage_arrears: Money,
    inventories_at_declaration: BTreeMap<GoodId, QuantityMilli>,
}

impl FirmInsolvency {
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn administrator(&self) -> ActorId {
        self.administrator
    }
    #[must_use]
    pub const fn declared_on(&self) -> SimDate {
        self.declared_on
    }
    #[must_use]
    pub const fn cash_at_declaration(&self) -> Money {
        self.cash_at_declaration
    }
    #[must_use]
    pub const fn wage_arrears(&self) -> Money {
        self.wage_arrears
    }
    #[must_use]
    pub fn inventories_at_declaration(&self) -> &BTreeMap<GoodId, QuantityMilli> {
        &self.inventories_at_declaration
    }
}

impl World {
    pub(crate) fn declare_firm_insolvent(
        &mut self,
        firm: FirmId,
        administrator: ActorId,
        wage_arrears: Money,
    ) -> Result<(), WorldError> {
        if self.firm_insolvencies.contains_key(&firm) {
            return Ok(());
        }
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let snapshot = FirmInsolvency {
            firm,
            administrator,
            declared_on: self.date,
            cash_at_declaration: definition.cash(),
            wage_arrears,
            inventories_at_declaration: definition.inventories().clone(),
        };
        let inventory = snapshot
            .inventories_at_declaration
            .iter()
            .map(|(good, quantity)| (*good, *quantity))
            .collect();
        self.firm_insolvencies.insert(firm, snapshot.clone());
        self.firm_distress_months.remove(&firm);
        self.events.append(
            self.date,
            DomainEvent::FirmInsolvencyDeclared {
                firm,
                administrator,
                cash: snapshot.cash_at_declaration,
                wage_arrears,
                inventory,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn is_firm_insolvent(&self, firm: FirmId) -> bool {
        self.firm_insolvencies.contains_key(&firm)
    }

    #[must_use]
    pub fn firm_insolvencies(&self) -> &BTreeMap<FirmId, FirmInsolvency> {
        &self.firm_insolvencies
    }
}

/// A funded, administrator-authorized plan for reopening an insolvent firm.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmReorganizationPlan {
    pub firm: FirmId,
    pub administrator: ActorId,
    pub sponsor: ActorId,
    pub contribution: Money,
    pub staffing: Vec<(CohortId, u64)>,
    pub production_target: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmReorganization {
    pub firm: FirmId,
    pub administrator: ActorId,
    pub sponsor: ActorId,
    pub contribution: Money,
    pub claims_paid: Money,
    pub workers: u64,
    pub production_target: u64,
    pub cash_reserve: Money,
}

impl World {
    /// Pays every preserved worker claim and reopens an insolvent firm only when
    /// its proposed staff has a full month of payroll liquidity.
    ///
    /// # Errors
    /// Returns an error without mutation when authority, ownership, claims,
    /// staffing, capacity, or financing is invalid.
    #[allow(clippy::too_many_lines)]
    pub fn reorganize_firm(
        &mut self,
        plan: &FirmReorganizationPlan,
    ) -> Result<FirmReorganization, WorldError> {
        let insolvency =
            self.firm_insolvencies
                .get(&plan.firm)
                .ok_or(WorldError::InvalidFirmReorganization(
                    "firm is not insolvent",
                ))?;
        if insolvency.administrator() != plan.administrator {
            return Err(WorldError::InvalidFirmReorganization(
                "only the appointed administrator may approve reopening",
            ));
        }
        if !self
            .ownership_stakes
            .contains_key(&(plan.firm, plan.sponsor))
        {
            return Err(WorldError::InvalidFirmReorganization(
                "reorganization sponsor must be an economic owner",
            ));
        }
        if plan.contribution.minor_units() < 0 {
            return Err(WorldError::InvalidFirmReorganization(
                "owner contribution cannot be negative",
            ));
        }
        if plan.production_target == 0 {
            return Err(WorldError::InvalidFirmReorganization(
                "reopening requires a positive production target",
            ));
        }
        let firm = self
            .firms
            .get(&plan.firm)
            .ok_or(WorldError::UnknownFirm(plan.firm))?;
        if plan.production_target > firm.capacity_batches() {
            return Err(WorldError::InvalidProductionTarget {
                firm: plan.firm,
                target: plan.production_target,
                capacity: firm.capacity_batches(),
            });
        }
        let sponsor_cash = self
            .actor_cash
            .get(&plan.sponsor)
            .copied()
            .unwrap_or_default();
        if sponsor_cash.minor_units() < plan.contribution.minor_units() {
            return Err(WorldError::InvalidFirmReorganization(
                "sponsor lacks the promised contribution",
            ));
        }

        if self
            .employment_agreements
            .values()
            .any(|agreement| agreement.firm() == plan.firm && agreement.active())
        {
            return Err(WorldError::InvalidFirmReorganization(
                "insolvent firm must have no active workers before reopening",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut staffing = plan.staffing.clone();
        staffing.sort_by_key(|(cohort, _)| *cohort);
        let mut workers = 0_u64;
        let mut monthly_payroll = 0_i128;
        for (cohort, count) in &staffing {
            if *count == 0 || !seen.insert(*cohort) {
                return Err(WorldError::InvalidFirmReorganization(
                    "staffing must contain unique cohorts with positive workers",
                ));
            }
            let agreement = self
                .employment_agreements
                .get(&(plan.firm, *cohort))
                .ok_or(WorldError::InvalidFirmReorganization(
                    "staffing requires an existing employment agreement",
                ))?;
            workers = workers
                .checked_add(*count)
                .ok_or(WorldError::ArithmeticOverflow("reorganization workers"))?;
            let cohort_payroll = i128::from(agreement.wage().minor_units())
                .checked_mul(i128::from(*count))
                .ok_or(WorldError::ArithmeticOverflow("reorganization payroll"))?;
            monthly_payroll = monthly_payroll
                .checked_add(cohort_payroll)
                .ok_or(WorldError::ArithmeticOverflow("reorganization payroll"))?;
        }
        if workers == 0 {
            return Err(WorldError::InvalidFirmReorganization(
                "reopening requires a positive workforce",
            ));
        }
        if workers > firm.workers() {
            return Err(WorldError::InvalidFirmReorganization(
                "staffing exceeds firm worker capacity",
            ));
        }
        let claims = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == plan.firm)
            .try_fold(0_i128, |total, agreement| {
                total
                    .checked_add(i128::from(agreement.arrears().minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow("reorganization claims"))
            })?;
        let funded_cash = i128::from(firm.cash().minor_units())
            .checked_add(i128::from(plan.contribution.minor_units()))
            .ok_or(WorldError::ArithmeticOverflow("reorganization funding"))?;
        let required = claims
            .checked_add(monthly_payroll)
            .ok_or(WorldError::ArithmeticOverflow("reorganization funding"))?;
        if funded_cash < required {
            return Err(WorldError::InvalidFirmReorganization(
                "funding must pay all wage claims and preserve one month of new payroll",
            ));
        }

        let mut next = self.clone();
        next.actor_cash.insert(
            plan.sponsor,
            Money::from_minor_units(sponsor_cash.minor_units() - plan.contribution.minor_units()),
        );
        let current_cash = next.firms[&plan.firm].cash().minor_units();
        let financed_cash = current_cash
            .checked_add(plan.contribution.minor_units())
            .ok_or(WorldError::ArithmeticOverflow(
                "reorganization contribution",
            ))?;
        next.firms
            .get_mut(&plan.firm)
            .ok_or(WorldError::UnknownFirm(plan.firm))?
            .set_cash(Money::from_minor_units(financed_cash));

        let claim_keys: Vec<_> = next
            .employment_agreements
            .iter()
            .filter(|(_, agreement)| {
                agreement.firm() == plan.firm && agreement.arrears().minor_units() > 0
            })
            .map(|(key, _)| *key)
            .collect();
        for key in claim_keys {
            let claim = next.employment_agreements[&key].arrears();
            next.firms
                .get_mut(&plan.firm)
                .ok_or(WorldError::UnknownFirm(plan.firm))?
                .debit_cash(claim)?;
            next.cohorts
                .get_mut(&key.1)
                .ok_or(WorldError::UnknownCohort(key.1))?
                .credit_wealth(claim)?;
            let settled = next
                .employment_agreements
                .get_mut(&key)
                .ok_or(WorldError::InvalidFirmReorganization(
                    "worker claim disappeared",
                ))?
                .settle_arrears();
            next.record_firm_payroll(plan.firm, settled, settled, Money::default())?;
            next.events.append(
                next.date,
                DomainEvent::PayrollSettled {
                    firm: plan.firm,
                    cohort: key.1,
                    owed: settled,
                    paid: settled,
                    arrears: Money::default(),
                },
            );
        }

        next.firm_insolvencies.remove(&plan.firm);
        for (cohort, count) in &staffing {
            next.change_employment_workers(plan.firm, *cohort, *count)?;
        }
        next.set_firm_production_target(plan.administrator, plan.firm, plan.production_target)?;
        let cash_reserve = next.firms[&plan.firm].cash();
        let claims_paid = Money::from_minor_units(
            i64::try_from(claims)
                .map_err(|_| WorldError::ArithmeticOverflow("reorganization claims"))?,
        );
        let result = FirmReorganization {
            firm: plan.firm,
            administrator: plan.administrator,
            sponsor: plan.sponsor,
            contribution: plan.contribution,
            claims_paid,
            workers,
            production_target: plan.production_target,
            cash_reserve,
        };
        next.events.append(
            next.date,
            DomainEvent::FirmReorganized {
                firm: result.firm,
                administrator: result.administrator,
                sponsor: result.sponsor,
                contribution: result.contribution,
                claims_paid: result.claims_paid,
                workers: result.workers,
                production_target: result.production_target,
                cash_reserve: result.cash_reserve,
            },
        );
        *self = next;
        Ok(result)
    }
}

impl World {
    /// Derives a minimum viable one-batch reopening plan from authoritative
    /// claims, wages, labor requirements, ownership, policy, and liquid cash.
    ///
    /// # Errors
    /// Returns an error for inconsistent references or arithmetic overflow.
    #[allow(clippy::too_many_lines)]
    pub fn plan_observed_firm_reorganization(
        &self,
        firm_id: FirmId,
    ) -> Result<Option<FirmReorganizationPlan>, WorldError> {
        let Some(insolvency) = self.firm_insolvencies.get(&firm_id) else {
            return Ok(None);
        };
        let firm = self
            .firms
            .get(&firm_id)
            .ok_or(WorldError::UnknownFirm(firm_id))?;
        if firm.capacity_batches() == 0 {
            return Ok(None);
        }
        let recipe = self
            .production_recipes
            .get(&firm.recipe())
            .ok_or(WorldError::UnknownRecipe(firm.recipe()))?;
        let required_workers = recipe
            .labor_milli_worker_months()
            .div_ceil(QuantityMilli::SCALE)
            .max(1);
        if required_workers > firm.workers() {
            return Ok(None);
        }
        let Some(sponsor) = self
            .ownership_stakes
            .values()
            .filter(|stake| stake.firm() == firm_id)
            .max_by_key(|stake| {
                (
                    stake.economic_rights().get(),
                    std::cmp::Reverse(stake.owner()),
                )
            })
            .map(|stake| stake.owner())
        else {
            return Ok(None);
        };
        let policy = self
            .firm_policies
            .get(&firm_id)
            .ok_or(WorldError::MissingFirmPolicy(firm_id))?;
        if policy.reinvestment().get() == 0 {
            return Ok(None);
        }

        let mut remaining = required_workers;
        let mut staffing = Vec::new();
        let mut monthly_payroll = 0_i128;
        for ((agreement_firm, cohort), agreement) in &self.employment_agreements {
            if *agreement_firm != firm_id || remaining == 0 {
                continue;
            }
            let population = self
                .cohorts
                .get(cohort)
                .ok_or(WorldError::UnknownCohort(*cohort))?
                .people()
                .people();
            let employed_elsewhere: u64 = self
                .employment_agreements
                .values()
                .filter(|row| row.cohort() == *cohort && row.firm() != firm_id && row.active())
                .map(crate::EmploymentAgreement::workers)
                .sum();
            let available = population.saturating_sub(employed_elsewhere);
            let selected = remaining.min(available);
            if selected == 0 {
                continue;
            }
            let payroll = i128::from(agreement.wage().minor_units())
                .checked_mul(i128::from(selected))
                .ok_or(WorldError::ArithmeticOverflow(
                    "observed reorganization payroll",
                ))?;
            monthly_payroll =
                monthly_payroll
                    .checked_add(payroll)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "observed reorganization payroll",
                    ))?;
            staffing.push((*cohort, selected));
            remaining -= selected;
        }
        if remaining > 0 {
            return Ok(None);
        }
        let claims = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm_id)
            .try_fold(0_i128, |total, agreement| {
                total
                    .checked_add(i128::from(agreement.arrears().minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "observed reorganization claims",
                    ))
            })?;
        let required_cash =
            claims
                .checked_add(monthly_payroll)
                .ok_or(WorldError::ArithmeticOverflow(
                    "observed reorganization funding",
                ))?;
        let contribution = required_cash
            .saturating_sub(i128::from(firm.cash().minor_units()))
            .max(0);
        let contribution = i64::try_from(contribution)
            .map_err(|_| WorldError::ArithmeticOverflow("observed reorganization funding"))?;
        let sponsor_cash = self
            .actor_cash
            .get(&sponsor)
            .copied()
            .unwrap_or_default()
            .minor_units();
        let policy_limit = i128::from(sponsor_cash)
            .checked_mul(i128::from(policy.reinvestment().get()))
            .ok_or(WorldError::ArithmeticOverflow(
                "reorganization policy limit",
            ))?
            / 10_000;
        if contribution > sponsor_cash || i128::from(contribution) > policy_limit {
            return Ok(None);
        }
        Ok(Some(FirmReorganizationPlan {
            firm: firm_id,
            administrator: insolvency.administrator(),
            sponsor,
            contribution: Money::from_minor_units(contribution),
            staffing,
            production_target: 1,
        }))
    }

    /// Applies every currently affordable policy-authorized minimum viable
    /// reorganization in stable firm order.
    ///
    /// # Errors
    /// Returns an error atomically if a derived plan cannot be committed.
    pub fn execute_observed_firm_reorganizations(
        &mut self,
    ) -> Result<Vec<FirmReorganization>, WorldError> {
        let firms: Vec<_> = self.firm_insolvencies.keys().copied().collect();
        let mut next = self.clone();
        let mut completed = Vec::new();
        for firm in firms {
            if let Some(plan) = next.plan_observed_firm_reorganization(firm)? {
                completed.push(next.reorganize_firm(&plan)?);
            }
        }
        *self = next;
        Ok(completed)
    }
}
