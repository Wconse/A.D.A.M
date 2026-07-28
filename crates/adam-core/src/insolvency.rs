use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ActorId, CohortId, DomainEvent, FirmId, GoodId, Money, QuantityMilli, SimDate, World,
    WorldError,
};

const ADMINISTRATION_LIMIT_MONTHS: i64 = 12;

/// Contractual liquidation rank for an actor-funded firm loan.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum FirmCreditorPriority {
    Secured,
    Unsecured,
}

/// Outstanding principal owed by a firm to an actor creditor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmCreditorClaim {
    firm: FirmId,
    creditor: ActorId,
    priority: FirmCreditorPriority,
    principal: Money,
}

impl FirmCreditorClaim {
    #[must_use]
    pub const fn firm(self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn creditor(self) -> ActorId {
        self.creditor
    }
    #[must_use]
    pub const fn priority(self) -> FirmCreditorPriority {
        self.priority
    }
    #[must_use]
    pub const fn principal(self) -> Money {
        self.principal
    }
}

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
    liquidated_on: Option<SimDate>,
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
    #[must_use]
    pub const fn liquidated_on(&self) -> Option<SimDate> {
        self.liquidated_on
    }
}

impl World {
    /// Transfers real actor cash into an operating firm and records an equal creditor claim.
    ///
    /// # Errors
    /// Rejects unknown references, non-positive principal, insufficient creditor cash,
    /// duplicate claims at the same priority, and lending to an insolvent firm.
    pub fn issue_firm_credit(
        &mut self,
        creditor: ActorId,
        firm: FirmId,
        priority: FirmCreditorPriority,
        principal: Money,
    ) -> Result<(), WorldError> {
        if !self.actors().contains_key(&creditor) {
            return Err(WorldError::UnknownActor(creditor));
        }
        if !self.firms.contains_key(&firm) {
            return Err(WorldError::UnknownFirm(firm));
        }
        if self.is_firm_insolvent(firm) {
            return Err(WorldError::InvalidFirmCredit(
                "insolvent firms cannot take new credit",
            ));
        }
        if principal.minor_units() <= 0 {
            return Err(WorldError::InvalidFirmCredit(
                "credit principal must be positive",
            ));
        }
        let key = (firm, priority, creditor);
        if self.firm_creditor_claims.contains_key(&key) {
            return Err(WorldError::InvalidFirmCredit(
                "creditor already has a claim at this priority",
            ));
        }
        let creditor_cash = self.actor_cash.get(&creditor).copied().unwrap_or_default();
        if creditor_cash.minor_units() < principal.minor_units() {
            return Err(WorldError::InvalidFirmCredit(
                "creditor lacks the principal",
            ));
        }
        let mut next = self.clone();
        next.actor_cash.insert(
            creditor,
            Money::from_minor_units(creditor_cash.minor_units() - principal.minor_units()),
        );
        next.firms
            .get_mut(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?
            .apply_cash_delta(principal)?;
        next.firm_creditor_claims.insert(
            key,
            FirmCreditorClaim {
                firm,
                creditor,
                priority,
                principal,
            },
        );
        next.events.append(
            next.date,
            DomainEvent::FirmCreditIssued {
                firm,
                creditor,
                priority,
                principal,
            },
        );
        *self = next;
        Ok(())
    }

    #[must_use]
    pub fn firm_creditor_claims(
        &self,
    ) -> &BTreeMap<(FirmId, FirmCreditorPriority, ActorId), FirmCreditorClaim> {
        &self.firm_creditor_claims
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
            liquidated_on: None,
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
        if insolvency.liquidated_on().is_some() {
            return Err(WorldError::InvalidFirmReorganization(
                "a liquidated firm cannot reopen",
            ));
        }
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
        if insolvency.liquidated_on().is_some() {
            return Ok(None);
        }
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

/// A physical inventory transfer from a liquidating estate to a solvent local producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmLiquidationInventorySale {
    pub estate: FirmId,
    pub buyer: FirmId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub proceeds: Money,
}

/// A physical installed-capacity transfer from a liquidating estate to a solvent successor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmLiquidationCapacitySale {
    pub estate: FirmId,
    pub buyer: FirmId,
    pub capacity_batches: u64,
    pub proceeds: Money,
}

/// Terminal resolution of an estate that could not finance a viable reopening.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmLiquidation {
    pub firm: FirmId,
    pub administrator: ActorId,
    pub inventory_sales: Vec<FirmLiquidationInventorySale>,
    pub inventory_sale_proceeds: Money,
    pub capacity_sales: Vec<FirmLiquidationCapacitySale>,
    pub capacity_sale_proceeds: Money,
    pub capacity_written_off: u64,
    pub claims_paid: Money,
    pub claims_written_off: Money,
    pub creditor_claims_paid: Money,
    pub creditor_claims_written_off: Money,
    pub inventory_written_off: BTreeMap<GoodId, QuantityMilli>,
    pub owner_distribution: Money,
}

impl World {
    #[must_use]
    pub fn is_firm_liquidated(&self, firm: FirmId) -> bool {
        self.firm_insolvencies
            .get(&firm)
            .is_some_and(|insolvency| insolvency.liquidated_on().is_some())
    }

    fn administration_months(&self, insolvency: &FirmInsolvency) -> i64 {
        let current = i64::from(self.date.year()) * 12 + i64::from(self.date.month()) - 1;
        let declared = i64::from(insolvency.declared_on().year()) * 12
            + i64::from(insolvency.declared_on().month())
            - 1;
        current.saturating_sub(declared)
    }

    /// Closes estates that have remained unfunded for twelve complete months.
    /// Reorganization is attempted before this gate in the monthly cycle, so a
    /// newly viable plan always takes precedence over liquidation.
    ///
    /// # Errors
    /// Returns an error atomically for inconsistent claims or arithmetic overflow.
    pub fn execute_observed_firm_liquidations(
        &mut self,
    ) -> Result<Vec<FirmLiquidation>, WorldError> {
        let firms: Vec<_> = self
            .firm_insolvencies
            .iter()
            .filter(|(_, insolvency)| {
                insolvency.liquidated_on().is_none()
                    && self.administration_months(insolvency) >= ADMINISTRATION_LIMIT_MONTHS
            })
            .map(|(firm, _)| *firm)
            .collect();
        let mut next = self.clone();
        let mut completed = Vec::new();
        for firm in firms {
            completed.push(next.liquidate_firm(firm)?);
        }
        *self = next;
        Ok(completed)
    }

    #[allow(clippy::too_many_lines)]
    fn sell_liquidation_inventory(
        &mut self,
        estate_id: FirmId,
    ) -> Result<Vec<FirmLiquidationInventorySale>, WorldError> {
        let estate = self
            .firms
            .get(&estate_id)
            .ok_or(WorldError::UnknownFirm(estate_id))?;
        let region = estate.region();
        let inventory = estate.inventories().clone();
        let buyers: Vec<_> = self
            .firms
            .values()
            .filter(|firm| {
                firm.id() != estate_id
                    && firm.region() == region
                    && !self.is_firm_insolvent(firm.id())
            })
            .map(crate::Firm::id)
            .collect();
        let mut sales = Vec::new();
        for (good, offered) in inventory {
            let Some(unit_price) = self.regional_prices.get(&(region, good)).copied() else {
                continue;
            };
            if unit_price.minor_units() <= 0 {
                continue;
            }
            for buyer_id in &buyers {
                let buyer = &self.firms[buyer_id];
                let recipe = self
                    .production_recipes
                    .get(&buyer.recipe())
                    .ok_or(WorldError::UnknownRecipe(buyer.recipe()))?;
                let Some(input) = recipe.inputs().iter().find(|input| input.good() == good) else {
                    continue;
                };
                let target_batches = self
                    .firm_production_targets
                    .get(buyer_id)
                    .copied()
                    .unwrap_or(buyer.capacity_batches());
                let desired = input
                    .quantity_per_batch()
                    .get()
                    .checked_mul(target_batches)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "liquidation inventory demand",
                    ))?;
                let held = buyer
                    .inventories()
                    .get(&good)
                    .copied()
                    .unwrap_or_default()
                    .get();
                let shortfall = desired.saturating_sub(held);
                let remaining = self.firms[&estate_id]
                    .inventories()
                    .get(&good)
                    .copied()
                    .unwrap_or_default()
                    .get()
                    .min(offered.get());
                if shortfall == 0 || remaining == 0 {
                    continue;
                }
                let buyer_cash = self.firms[buyer_id].cash().minor_units().max(0);
                let affordable = u128::try_from(buyer_cash)
                    .map_err(|_| WorldError::ArithmeticOverflow("liquidation buyer cash"))?
                    .checked_mul(u128::from(QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow("liquidation affordability"))?
                    / u128::try_from(unit_price.minor_units())
                        .map_err(|_| WorldError::ArithmeticOverflow("liquidation unit price"))?;
                let quantity = remaining
                    .min(shortfall)
                    .min(u64::try_from(affordable).unwrap_or(u64::MAX));
                if quantity == 0 {
                    continue;
                }
                let spend_numerator = u128::from(quantity)
                    .checked_mul(u128::try_from(unit_price.minor_units()).map_err(|_| {
                        WorldError::ArithmeticOverflow("liquidation inventory price")
                    })?)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "liquidation inventory price",
                    ))?;
                let spend = spend_numerator.div_ceil(u128::from(QuantityMilli::SCALE));
                let spend = Money::from_minor_units(
                    i64::try_from(spend)
                        .map_err(|_| WorldError::ArithmeticOverflow("liquidation proceeds"))?,
                );
                self.firms
                    .get_mut(buyer_id)
                    .ok_or(WorldError::UnknownFirm(*buyer_id))?
                    .debit_cash(spend)?;
                self.firms
                    .get_mut(&estate_id)
                    .ok_or(WorldError::UnknownFirm(estate_id))?
                    .debit_inventory(good, QuantityMilli::new(quantity))?;
                self.firms
                    .get_mut(buyer_id)
                    .ok_or(WorldError::UnknownFirm(*buyer_id))?
                    .credit_inventory(good, QuantityMilli::new(quantity))?;
                self.firms
                    .get_mut(&estate_id)
                    .ok_or(WorldError::UnknownFirm(estate_id))?
                    .apply_cash_delta(spend)?;
                let sale = FirmLiquidationInventorySale {
                    estate: estate_id,
                    buyer: *buyer_id,
                    good,
                    quantity: QuantityMilli::new(quantity),
                    proceeds: spend,
                };
                self.events.append(
                    self.date,
                    DomainEvent::FirmLiquidationInventorySold {
                        estate: sale.estate,
                        buyer: sale.buyer,
                        good: sale.good,
                        quantity: sale.quantity,
                        proceeds: sale.proceeds,
                    },
                );
                sales.push(sale);
            }
        }
        Ok(sales)
    }

    #[allow(clippy::too_many_lines)]
    fn sell_liquidation_capacity(
        &mut self,
        estate_id: FirmId,
    ) -> Result<Vec<FirmLiquidationCapacitySale>, WorldError> {
        let estate = self
            .firms
            .get(&estate_id)
            .ok_or(WorldError::UnknownFirm(estate_id))?;
        let region = estate.region();
        let recipe_id = estate.recipe();
        let recipe = self
            .production_recipes
            .get(&recipe_id)
            .ok_or(WorldError::UnknownRecipe(recipe_id))?;
        let Some(output_price) = self
            .regional_prices
            .get(&(region, recipe.output_good()))
            .copied()
        else {
            return Ok(Vec::new());
        };
        if output_price.minor_units() <= 0 {
            return Ok(Vec::new());
        }
        // Stage 0 reserve price: one batch of installed capacity is valued at one
        // month of its reference-price gross output. This is observable, recipe-
        // specific, and avoids creating a separate ungrounded capital price.
        let batch_value = u128::from(recipe.output_per_batch().get())
            .checked_mul(
                u128::try_from(output_price.minor_units())
                    .map_err(|_| WorldError::ArithmeticOverflow("liquidation capacity price"))?,
            )
            .ok_or(WorldError::ArithmeticOverflow("liquidation capacity price"))?
            .div_ceil(u128::from(QuantityMilli::SCALE));
        let batch_price = i64::try_from(batch_value.max(1))
            .map_err(|_| WorldError::ArithmeticOverflow("liquidation capacity price"))?;
        let buyers: Vec<_> = self
            .firms
            .values()
            .filter(|firm| {
                firm.id() != estate_id
                    && firm.region() == region
                    && firm.recipe() == recipe_id
                    && !self.is_firm_insolvent(firm.id())
            })
            .map(crate::Firm::id)
            .collect();
        let mut sales = Vec::new();
        for buyer_id in buyers {
            let remaining = self.firms[&estate_id].capacity_batches();
            if remaining == 0 {
                break;
            }
            let buyer = &self.firms[&buyer_id];
            let target = self
                .firm_production_targets
                .get(&buyer_id)
                .copied()
                .unwrap_or(buyer.capacity_batches());
            // Only a producer already choosing its installed ceiling may expand;
            // the auction cannot manufacture demand or override a cautious target.
            if target == 0 || target < buyer.capacity_batches() {
                continue;
            }
            let payroll_reserve = self
                .employment_agreements
                .values()
                .filter(|agreement| agreement.firm() == buyer_id && agreement.active())
                .try_fold(0_i128, |sum, agreement| {
                    let payroll = i128::from(agreement.wage().minor_units())
                        .checked_mul(i128::from(agreement.workers()))
                        .ok_or(WorldError::ArithmeticOverflow(
                            "liquidation capacity payroll reserve",
                        ))?;
                    sum.checked_add(payroll)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "liquidation capacity payroll reserve",
                        ))
                })?;
            let spendable = i128::from(buyer.cash().minor_units())
                .saturating_sub(payroll_reserve)
                .max(0);
            let affordable = u64::try_from(spendable / i128::from(batch_price)).unwrap_or(u64::MAX);
            // A successor may at most double its current installed scale in one
            // administered sale, limiting fire-sale concentration and integration risk.
            let integration_limit = buyer.capacity_batches().max(1);
            let transferred = remaining.min(affordable).min(integration_limit);
            if transferred == 0 {
                continue;
            }
            let proceeds_minor = i128::from(batch_price)
                .checked_mul(i128::from(transferred))
                .ok_or(WorldError::ArithmeticOverflow(
                    "liquidation capacity proceeds",
                ))?;
            let proceeds =
                Money::from_minor_units(i64::try_from(proceeds_minor).map_err(|_| {
                    WorldError::ArithmeticOverflow("liquidation capacity proceeds")
                })?);
            self.firms
                .get_mut(&buyer_id)
                .ok_or(WorldError::UnknownFirm(buyer_id))?
                .debit_cash(proceeds)?;
            self.firms
                .get_mut(&estate_id)
                .ok_or(WorldError::UnknownFirm(estate_id))?
                .remove_capacity(transferred)?;
            self.firms
                .get_mut(&buyer_id)
                .ok_or(WorldError::UnknownFirm(buyer_id))?
                .add_capacity(transferred)?;
            self.firms
                .get_mut(&estate_id)
                .ok_or(WorldError::UnknownFirm(estate_id))?
                .apply_cash_delta(proceeds)?;
            let sale = FirmLiquidationCapacitySale {
                estate: estate_id,
                buyer: buyer_id,
                capacity_batches: transferred,
                proceeds,
            };
            self.events.append(
                self.date,
                DomainEvent::FirmLiquidationCapacitySold {
                    estate: sale.estate,
                    buyer: sale.buyer,
                    capacity_batches: sale.capacity_batches,
                    proceeds: sale.proceeds,
                },
            );
            sales.push(sale);
        }
        Ok(sales)
    }

    fn settle_liquidation_creditors(
        &mut self,
        firm_id: FirmId,
    ) -> Result<(Money, Money), WorldError> {
        let mut total_paid = 0_i64;
        let mut total_written_off = 0_i64;
        for priority in [
            FirmCreditorPriority::Secured,
            FirmCreditorPriority::Unsecured,
        ] {
            let claims: Vec<_> = self
                .firm_creditor_claims
                .iter()
                .filter(|((firm, claim_priority, _), _)| {
                    *firm == firm_id && *claim_priority == priority
                })
                .map(|(key, claim)| (*key, i128::from(claim.principal().minor_units())))
                .collect();
            let class_total = claims.iter().try_fold(0_i128, |sum, (_, amount)| {
                sum.checked_add(*amount)
                    .ok_or(WorldError::ArithmeticOverflow("creditor claims"))
            })?;
            if class_total == 0 {
                continue;
            }
            let available = i128::from(self.firms[&firm_id].cash().minor_units()).max(0);
            let distributable = available.min(class_total);
            let mut allocations = Vec::with_capacity(claims.len());
            let mut allocated = 0_i128;
            for (key, claim) in &claims {
                let numerator = distributable
                    .checked_mul(*claim)
                    .ok_or(WorldError::ArithmeticOverflow("creditor allocation"))?;
                let base = numerator / class_total;
                allocated += base;
                allocations.push((*key, *claim, base, numerator % class_total));
            }
            let mut residual = distributable - allocated;
            let mut order: Vec<_> = (0..allocations.len()).collect();
            order.sort_by_key(|index| {
                (
                    std::cmp::Reverse(allocations[*index].3),
                    allocations[*index].0,
                )
            });
            for index in order {
                if residual == 0 {
                    break;
                }
                allocations[index].2 += 1;
                residual -= 1;
            }
            let class_payment = Money::from_minor_units(
                i64::try_from(distributable)
                    .map_err(|_| WorldError::ArithmeticOverflow("creditor payment"))?,
            );
            self.firms
                .get_mut(&firm_id)
                .ok_or(WorldError::UnknownFirm(firm_id))?
                .debit_cash(class_payment)?;
            for (key, claim, payment, _) in allocations {
                let paid = i64::try_from(payment)
                    .map_err(|_| WorldError::ArithmeticOverflow("creditor payment"))?;
                let written_off = i64::try_from(claim - payment)
                    .map_err(|_| WorldError::ArithmeticOverflow("creditor write-off"))?;
                let creditor = key.2;
                let cash = self.actor_cash.get(&creditor).copied().unwrap_or_default();
                self.actor_cash.insert(
                    creditor,
                    Money::from_minor_units(
                        cash.minor_units()
                            .checked_add(paid)
                            .ok_or(WorldError::ArithmeticOverflow("creditor recovery"))?,
                    ),
                );
                self.firm_creditor_claims.remove(&key);
                self.events.append(
                    self.date,
                    DomainEvent::FirmCreditorClaimSettled {
                        firm: firm_id,
                        creditor,
                        priority,
                        paid: Money::from_minor_units(paid),
                        written_off: Money::from_minor_units(written_off),
                    },
                );
                total_paid = total_paid
                    .checked_add(paid)
                    .ok_or(WorldError::ArithmeticOverflow("creditor payments"))?;
                total_written_off = total_written_off
                    .checked_add(written_off)
                    .ok_or(WorldError::ArithmeticOverflow("creditor write-offs"))?;
            }
        }
        Ok((
            Money::from_minor_units(total_paid),
            Money::from_minor_units(total_written_off),
        ))
    }

    #[allow(clippy::too_many_lines)]
    fn liquidate_firm(&mut self, firm_id: FirmId) -> Result<FirmLiquidation, WorldError> {
        let insolvency = self
            .firm_insolvencies
            .get(&firm_id)
            .ok_or(WorldError::InvalidFirmReorganization(
                "firm is not insolvent",
            ))?
            .clone();
        if insolvency.liquidated_on().is_some() {
            return Err(WorldError::InvalidFirmReorganization(
                "firm estate is already liquidated",
            ));
        }
        if !self.firms.contains_key(&firm_id) {
            return Err(WorldError::UnknownFirm(firm_id));
        }
        let mut next = self.clone();
        let inventory_sales = next.sell_liquidation_inventory(firm_id)?;
        let inventory_sale_proceeds_minor =
            inventory_sales.iter().try_fold(0_i64, |total, sale| {
                total
                    .checked_add(sale.proceeds.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow("liquidation sale proceeds"))
            })?;
        let capacity_sales = next.sell_liquidation_capacity(firm_id)?;
        let capacity_sale_proceeds_minor =
            capacity_sales.iter().try_fold(0_i64, |total, sale| {
                total.checked_add(sale.proceeds.minor_units()).ok_or(
                    WorldError::ArithmeticOverflow("liquidation capacity proceeds"),
                )
            })?;
        let estate_cash = i128::from(next.firms[&firm_id].cash().minor_units()).max(0);
        let claims: Vec<_> = next
            .employment_agreements
            .iter()
            .filter(|(_, agreement)| {
                agreement.firm() == firm_id && agreement.arrears().minor_units() > 0
            })
            .map(|(key, agreement)| (*key, i128::from(agreement.arrears().minor_units())))
            .collect();
        let total_claims = claims.iter().try_fold(0_i128, |total, (_, claim)| {
            total
                .checked_add(*claim)
                .ok_or(WorldError::ArithmeticOverflow("liquidation claims"))
        })?;
        let distributable = estate_cash.min(total_claims);
        let mut payouts = Vec::with_capacity(claims.len());
        let mut allocated = 0_i128;
        for (key, claim) in &claims {
            let numerator =
                distributable
                    .checked_mul(*claim)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "liquidation claim allocation",
                    ))?;
            let base = if total_claims == 0 {
                0
            } else {
                numerator / total_claims
            };
            let remainder = if total_claims == 0 {
                0
            } else {
                numerator % total_claims
            };
            allocated += base;
            payouts.push((*key, base, remainder));
        }
        let mut residual = distributable - allocated;
        let mut priority: Vec<_> = (0..payouts.len()).collect();
        priority.sort_by_key(|index| (std::cmp::Reverse(payouts[*index].2), payouts[*index].0));
        for index in priority {
            if residual == 0 {
                break;
            }
            payouts[index].1 += 1;
            residual -= 1;
        }

        let paid_money = Money::from_minor_units(
            i64::try_from(distributable)
                .map_err(|_| WorldError::ArithmeticOverflow("liquidation claims"))?,
        );
        next.firms
            .get_mut(&firm_id)
            .ok_or(WorldError::UnknownFirm(firm_id))?
            .debit_cash(paid_money)?;
        for (key, payout, _) in payouts {
            if payout == 0 {
                continue;
            }
            let payment = Money::from_minor_units(
                i64::try_from(payout)
                    .map_err(|_| WorldError::ArithmeticOverflow("liquidation claim payment"))?,
            );
            next.cohorts
                .get_mut(&key.1)
                .ok_or(WorldError::UnknownCohort(key.1))?
                .credit_wealth(payment)?;
            let agreement = next.employment_agreements.get_mut(&key).ok_or(
                WorldError::InvalidFirmReorganization("worker claim disappeared"),
            )?;
            let owed = agreement.arrears();
            let paid = agreement.settle_arrears_up_to(payment);
            let arrears = agreement.arrears();
            next.record_firm_payroll(firm_id, owed, paid, arrears)?;
            next.events.append(
                next.date,
                DomainEvent::PayrollSettled {
                    firm: firm_id,
                    cohort: key.1,
                    owed,
                    paid,
                    arrears,
                },
            );
        }
        // Liquidation is terminal: any worker claim not covered by estate cash is
        // explicitly written off by the liquidation event instead of surviving
        // as an unpayable balance on a frozen agreement.
        for (key, _) in &claims {
            next.employment_agreements
                .get_mut(key)
                .ok_or(WorldError::InvalidFirmReorganization(
                    "worker claim disappeared",
                ))?
                .settle_arrears();
        }

        let (creditor_claims_paid, creditor_claims_written_off) =
            next.settle_liquidation_creditors(firm_id)?;

        let owner_pool = next.firms[&firm_id].cash().minor_units().max(0);
        let owners: Vec<_> = next
            .ownership_stakes
            .values()
            .filter(|stake| stake.firm() == firm_id && stake.economic_rights().get() > 0)
            .map(|stake| (stake.owner(), i128::from(stake.economic_rights().get())))
            .collect();
        let total_rights: i128 = owners.iter().map(|(_, rights)| *rights).sum();
        let mut owner_paid = 0_i64;
        if owner_pool > 0 && total_rights > 0 {
            let pool = i128::from(owner_pool);
            let mut distributions = Vec::with_capacity(owners.len());
            let mut allocated = 0_i128;
            for (owner, rights) in owners {
                let numerator = pool
                    .checked_mul(rights)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "liquidation owner distribution",
                    ))?;
                let base = numerator / total_rights;
                allocated += base;
                distributions.push((owner, base, numerator % total_rights));
            }
            let mut residual = pool - allocated;
            let mut priority: Vec<_> = (0..distributions.len()).collect();
            priority.sort_by_key(|index| {
                (
                    std::cmp::Reverse(distributions[*index].2),
                    distributions[*index].0,
                )
            });
            for index in priority {
                if residual == 0 {
                    break;
                }
                distributions[index].1 += 1;
                residual -= 1;
            }
            for (owner, amount, _) in distributions {
                let amount = i64::try_from(amount).map_err(|_| {
                    WorldError::ArithmeticOverflow("liquidation owner distribution")
                })?;
                let current = next.actor_cash.get(&owner).copied().unwrap_or_default();
                let updated = current.minor_units().checked_add(amount).ok_or(
                    WorldError::ArithmeticOverflow("liquidation owner distribution"),
                )?;
                next.actor_cash
                    .insert(owner, Money::from_minor_units(updated));
                owner_paid =
                    owner_paid
                        .checked_add(amount)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "liquidation owner distribution",
                        ))?;
            }
            next.firms
                .get_mut(&firm_id)
                .ok_or(WorldError::UnknownFirm(firm_id))?
                .debit_cash(Money::from_minor_units(owner_paid))?;
        }

        let inventory_written_off = next.firms[&firm_id].inventories().clone();
        for (good, quantity) in &inventory_written_off {
            next.firms
                .get_mut(&firm_id)
                .ok_or(WorldError::UnknownFirm(firm_id))?
                .debit_inventory(*good, *quantity)?;
        }
        let capacity_written_off = next.firms[&firm_id].capacity_batches();
        next.firms
            .get_mut(&firm_id)
            .ok_or(WorldError::UnknownFirm(firm_id))?
            .remove_capacity(capacity_written_off)?;
        next.firm_production_targets.remove(&firm_id);
        next.firm_insolvencies
            .get_mut(&firm_id)
            .ok_or(WorldError::InvalidFirmReorganization(
                "firm estate disappeared",
            ))?
            .liquidated_on = Some(next.date);
        let claims_written_off = total_claims - distributable;
        let result = FirmLiquidation {
            firm: firm_id,
            administrator: insolvency.administrator(),
            inventory_sales,
            inventory_sale_proceeds: Money::from_minor_units(inventory_sale_proceeds_minor),
            capacity_sales,
            capacity_sale_proceeds: Money::from_minor_units(capacity_sale_proceeds_minor),
            capacity_written_off,
            claims_paid: paid_money,
            claims_written_off: Money::from_minor_units(
                i64::try_from(claims_written_off)
                    .map_err(|_| WorldError::ArithmeticOverflow("liquidation claims"))?,
            ),
            creditor_claims_paid,
            creditor_claims_written_off,
            inventory_written_off: inventory_written_off.clone(),
            owner_distribution: Money::from_minor_units(owner_paid),
        };
        next.events.append(
            next.date,
            DomainEvent::FirmLiquidated {
                firm: result.firm,
                administrator: result.administrator,
                claims_paid: result.claims_paid,
                claims_written_off: result.claims_written_off,
                inventory_written_off: inventory_written_off.into_iter().collect(),
                capacity_written_off: result.capacity_written_off,
                owner_distribution: result.owner_distribution,
            },
        );
        *self = next;
        Ok(result)
    }
}
