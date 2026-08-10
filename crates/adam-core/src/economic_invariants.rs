use std::collections::BTreeMap;

use crate::{
    ActorId, CohortId, ContractId, ContractStatus, CountryId, FirmId, GoodId, Money, MoneyStock,
    ProgramId, QuantityMilli, RegionId, RouteId, ShipmentStatus, SimDate, TerminalId, World,
    WorldError,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EconomicInvariantSeverity {
    HardViolation,
    KnownModelGap,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EconomicInvariantCode {
    NegativeLiquidBalance,
    RegionPopulationMismatch,
    InvalidHouseholdCount,
    LaborOvercommitment,
    UnknownCapacityReservation,
    RouteCapacityExceeded,
    FreightContractCapacityExceeded,
    FreightSpotCapacityExceeded,
    TerminalCapacityExceeded,
    WorkforceLedgerDisagreement,
    PayerlessHouseholdIncome,
    HouseholdDebtHasNoCreditor,
    PublicDebtHasNoCreditor,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EconomicInvariantEntity {
    World,
    Actor(ActorId),
    Cohort(CohortId),
    Contract(ContractId),
    Country(CountryId),
    Firm(FirmId),
    Program(ProgramId),
    Region(RegionId),
    Route(RouteId),
    Terminal(TerminalId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EconomicInvariantFinding {
    pub severity: EconomicInvariantSeverity,
    pub code: EconomicInvariantCode,
    pub entity: EconomicInvariantEntity,
    pub observed: i128,
    pub allowed: i128,
    pub audit_reference: &'static str,
    pub explanation: &'static str,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EconomicClaimsSummary {
    pub household_debt: Money,
    pub public_debt: Money,
    pub wage_arrears: Money,
    pub firm_credit_principal: Money,
    pub firm_credit_interest: Money,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EconomicCommitmentSummary {
    pub investment_commitments: Money,
    pub program_carryover: Money,
    pub program_workers_recorded: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoodStockSummary {
    pub firm_inventory: QuantityMilli,
    pub public_reserve: QuantityMilli,
    pub in_transit: QuantityMilli,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoodTransitionReconciliation {
    pub good: GoodId,
    pub opening: i128,
    pub closing: i128,
    pub authorized_net_change: i128,
    pub unexplained_change: i128,
}

impl GoodTransitionReconciliation {
    #[must_use]
    pub const fn is_reconciled(self) -> bool {
        self.unexplained_change == 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EconomicInvariantReport {
    pub date: SimDate,
    pub money: MoneyStock,
    pub claims: EconomicClaimsSummary,
    pub commitments: EconomicCommitmentSummary,
    pub goods: BTreeMap<GoodId, GoodStockSummary>,
    pub findings: Vec<EconomicInvariantFinding>,
}

impl EconomicInvariantReport {
    #[must_use]
    pub fn is_hard_valid(&self) -> bool {
        self.findings
            .iter()
            .all(|finding| finding.severity != EconomicInvariantSeverity::HardViolation)
    }

    pub fn hard_violations(&self) -> impl Iterator<Item = &EconomicInvariantFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.severity == EconomicInvariantSeverity::HardViolation)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MoneyTransitionReconciliation {
    pub opening: Money,
    pub closing: Money,
    pub authorized_net_issuance: Money,
    pub unexplained_change: Money,
}

impl MoneyTransitionReconciliation {
    #[must_use]
    pub const fn is_reconciled(self) -> bool {
        self.unexplained_change.minor_units() == 0
    }
}

/// Reconciles two liquid-money snapshots against an explicitly authorized net issue or retirement.
///
/// # Errors
/// Returns an arithmetic error when either stock total or their checked difference does not fit.
pub fn reconcile_money_transition(
    opening: &EconomicInvariantReport,
    closing: &EconomicInvariantReport,
    authorized_net_issuance: Money,
) -> Result<MoneyTransitionReconciliation, WorldError> {
    let opening_total = opening.money.total()?;
    let closing_total = closing.money.total()?;
    let unexplained = i128::from(closing_total.minor_units())
        .checked_sub(i128::from(opening_total.minor_units()))
        .and_then(|value| value.checked_sub(i128::from(authorized_net_issuance.minor_units())))
        .ok_or(WorldError::ArithmeticOverflow(
            "economic invariant money transition",
        ))?;
    Ok(MoneyTransitionReconciliation {
        opening: opening_total,
        closing: closing_total,
        authorized_net_issuance,
        unexplained_change: money_from_i128(unexplained, "economic invariant money transition")?,
    })
}

/// Reconciles total physical stock by good against explicitly authorized production,
/// consumption, spoilage, imports, exports, and transformation.
///
/// # Errors
/// Returns an arithmetic error when a physical stock total cannot be represented.
pub fn reconcile_goods_transition(
    opening: &EconomicInvariantReport,
    closing: &EconomicInvariantReport,
    authorized_net_changes: &BTreeMap<GoodId, i128>,
) -> Result<Vec<GoodTransitionReconciliation>, WorldError> {
    let mut goods = opening
        .goods
        .keys()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    goods.extend(closing.goods.keys().copied());
    goods.extend(authorized_net_changes.keys().copied());
    goods
        .into_iter()
        .map(|good| {
            let opening = total_good_stock(opening.goods.get(&good))?;
            let closing = total_good_stock(closing.goods.get(&good))?;
            let authorized_net_change = authorized_net_changes.get(&good).copied().unwrap_or(0);
            let unexplained_change = closing
                .checked_sub(opening)
                .and_then(|change| change.checked_sub(authorized_net_change))
                .ok_or(WorldError::ArithmeticOverflow(
                    "economic invariant goods transition",
                ))?;
            Ok(GoodTransitionReconciliation {
                good,
                opening,
                closing,
                authorized_net_change,
                unexplained_change,
            })
        })
        .collect()
}

impl World {
    /// Builds a deterministic, read-only projection of current economic stocks and violations.
    ///
    /// # Errors
    /// Returns an arithmetic error when a diagnostic total cannot fit its domain type.
    pub fn economic_invariant_report(&self) -> Result<EconomicInvariantReport, WorldError> {
        let money = self.observed_money_stock()?;
        let claims = self.economic_claims_summary()?;
        let commitments = self.economic_commitment_summary()?;
        let goods = self.economic_goods_summary()?;
        let mut findings = Vec::new();

        self.find_negative_liquid_balances(&mut findings);
        self.find_population_and_household_violations(&mut findings)?;
        self.find_labor_violations(&mut findings)?;
        self.find_payerless_income_gaps(&mut findings)?;
        self.find_capacity_violations(&mut findings)?;
        Self::find_known_model_gaps(claims, commitments, &mut findings);
        findings.sort();

        Ok(EconomicInvariantReport {
            date: self.date,
            money,
            claims,
            commitments,
            goods,
            findings,
        })
    }

    fn economic_claims_summary(&self) -> Result<EconomicClaimsSummary, WorldError> {
        let household_debt = self.cohorts.values().try_fold(0_i128, |total, cohort| {
            checked_money_add(total, cohort.debt(), "household debt total")
        })?;
        let public_debt = self.countries.values().try_fold(0_i128, |total, country| {
            checked_money_add(
                total,
                country.indicators().public_debt(),
                "public debt total",
            )
        })?;
        let wage_arrears = self
            .employment_agreements
            .values()
            .try_fold(0_i128, |total, agreement| {
                checked_money_add(total, agreement.arrears(), "wage arrears total")
            })?;
        let (firm_credit_principal, firm_credit_interest) = self
            .firm_creditor_claims
            .values()
            .try_fold((0_i128, 0_i128), |(principal, interest), claim| {
                Ok::<(i128, i128), WorldError>((
                    checked_money_add(principal, claim.principal(), "firm credit principal")?,
                    checked_money_add(interest, claim.accrued_interest(), "firm credit interest")?,
                ))
            })?;
        Ok(EconomicClaimsSummary {
            household_debt: money_from_i128(household_debt, "household debt total")?,
            public_debt: money_from_i128(public_debt, "public debt total")?,
            wage_arrears: money_from_i128(wage_arrears, "wage arrears total")?,
            firm_credit_principal: money_from_i128(firm_credit_principal, "firm credit principal")?,
            firm_credit_interest: money_from_i128(firm_credit_interest, "firm credit interest")?,
        })
    }

    fn economic_commitment_summary(&self) -> Result<EconomicCommitmentSummary, WorldError> {
        let investment_commitments = self
            .committed_investments
            .values()
            .try_fold(0_i128, |total, amount| {
                checked_money_add(total, *amount, "investment commitments")
            })?;
        let program_carryover =
            self.government_programs
                .values()
                .try_fold(0_i128, |total, program| {
                    checked_money_add(
                        total,
                        program.carryover_funding(),
                        "government program carryover",
                    )
                })?;
        let program_workers_recorded =
            self.government_program_labor_usage
                .values()
                .try_fold(0_u64, |total, workers| {
                    total
                        .checked_add(*workers)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "government program labor usage",
                        ))
                })?;
        Ok(EconomicCommitmentSummary {
            investment_commitments: money_from_i128(
                investment_commitments,
                "investment commitments",
            )?,
            program_carryover: money_from_i128(program_carryover, "government program carryover")?,
            program_workers_recorded,
        })
    }

    fn economic_goods_summary(&self) -> Result<BTreeMap<GoodId, GoodStockSummary>, WorldError> {
        let mut goods = BTreeMap::<GoodId, GoodStockSummary>::new();
        for firm in self.firms.values() {
            for (good, quantity) in firm.inventories() {
                let row = goods.entry(*good).or_default();
                row.firm_inventory =
                    checked_quantity_add(row.firm_inventory, *quantity, "firm inventory total")?;
            }
        }
        for ((_, good), quantity) in &self.government_reserves {
            let row = goods.entry(*good).or_default();
            row.public_reserve =
                checked_quantity_add(row.public_reserve, *quantity, "public reserve total")?;
        }
        for shipment in self.inventory_shipments.values() {
            if shipment.status() == ShipmentStatus::Delivered {
                continue;
            }
            let row = goods.entry(shipment.good()).or_default();
            row.in_transit = checked_quantity_add(
                row.in_transit,
                shipment.quantity(),
                "in-transit inventory total",
            )?;
        }
        Ok(goods)
    }

    fn find_negative_liquid_balances(&self, findings: &mut Vec<EconomicInvariantFinding>) {
        for cohort in self.cohorts.values() {
            push_negative_balance(
                findings,
                EconomicInvariantEntity::Cohort(cohort.id()),
                cohort.liquid_wealth(),
            );
        }
        for firm in self.firms.values() {
            push_negative_balance(
                findings,
                EconomicInvariantEntity::Firm(firm.id()),
                firm.cash(),
            );
        }
        for (actor, cash) in &self.actor_cash {
            push_negative_balance(findings, EconomicInvariantEntity::Actor(*actor), *cash);
        }
        for country in self.countries.values() {
            push_negative_balance(
                findings,
                EconomicInvariantEntity::Country(country.id()),
                country.indicators().treasury(),
            );
        }
    }

    fn find_population_and_household_violations(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) -> Result<(), WorldError> {
        for region in self.regions.values() {
            let cohort_population = self
                .cohorts
                .values()
                .filter(|cohort| cohort.region() == region.id())
                .try_fold(0_u64, |total, cohort| {
                    total.checked_add(cohort.people().people()).ok_or(
                        WorldError::ArithmeticOverflow("economic invariant population"),
                    )
                })?;
            if cohort_population != region.population().people() {
                findings.push(EconomicInvariantFinding {
                    severity: EconomicInvariantSeverity::HardViolation,
                    code: EconomicInvariantCode::RegionPopulationMismatch,
                    entity: EconomicInvariantEntity::Region(region.id()),
                    observed: i128::from(cohort_population),
                    allowed: i128::from(region.population().people()),
                    audit_reference: "C-1 / AD-1",
                    explanation: "cohort population does not equal the registered regional stock",
                });
            }
        }
        for cohort in self.cohorts.values() {
            let people = cohort.people().people();
            let households = cohort.households();
            let valid = if people == 0 {
                households == 0
            } else {
                (1..=people).contains(&households)
            };
            if !valid {
                findings.push(EconomicInvariantFinding {
                    severity: EconomicInvariantSeverity::HardViolation,
                    code: EconomicInvariantCode::InvalidHouseholdCount,
                    entity: EconomicInvariantEntity::Cohort(cohort.id()),
                    observed: i128::from(households),
                    allowed: i128::from(people),
                    audit_reference: "AD-1",
                    explanation: "household count is incompatible with cohort population",
                });
            }
        }
        Ok(())
    }

    fn find_labor_violations(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) -> Result<(), WorldError> {
        let mut workers_by_cohort = BTreeMap::<CohortId, u64>::new();
        let mut workers_by_firm = BTreeMap::<FirmId, u64>::new();
        for agreement in self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.active())
        {
            add_workers(
                &mut workers_by_cohort,
                agreement.cohort(),
                agreement.workers(),
                "cohort employment total",
            )?;
            add_workers(
                &mut workers_by_firm,
                agreement.firm(),
                agreement.workers(),
                "firm employment total",
            )?;
        }
        for (cohort, workers) in workers_by_cohort {
            let people = self
                .cohorts
                .get(&cohort)
                .map_or(0, |definition| definition.people().people());
            if workers > people {
                findings.push(EconomicInvariantFinding {
                    severity: EconomicInvariantSeverity::HardViolation,
                    code: EconomicInvariantCode::LaborOvercommitment,
                    entity: EconomicInvariantEntity::Cohort(cohort),
                    observed: i128::from(workers),
                    allowed: i128::from(people),
                    audit_reference: "J-7 / AB-8",
                    explanation: "active employment commits more workers than the cohort contains",
                });
            }
        }
        for firm in self.firms.values() {
            let contracted = workers_by_firm.get(&firm.id()).copied().unwrap_or(0);
            if contracted != firm.workers() {
                findings.push(EconomicInvariantFinding {
                    severity: EconomicInvariantSeverity::KnownModelGap,
                    code: EconomicInvariantCode::WorkforceLedgerDisagreement,
                    entity: EconomicInvariantEntity::Firm(firm.id()),
                    observed: i128::from(contracted),
                    allowed: i128::from(firm.workers()),
                    audit_reference: "J-1 / AC-5",
                    explanation: "employment agreements and the firm's productive workforce disagree",
                });
            }
        }
        Ok(())
    }

    fn find_payerless_income_gaps(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) -> Result<(), WorldError> {
        let mut contracted_workers = BTreeMap::<CohortId, u64>::new();
        for agreement in self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.active())
        {
            add_workers(
                &mut contracted_workers,
                agreement.cohort(),
                agreement.workers(),
                "payerless income contracted workers",
            )?;
        }
        for cohort in self.cohorts.values() {
            let capacity = self
                .cohort_health
                .get(&cohort.id())
                .map_or(crate::BasisPoints::MAX, |health| {
                    health.functional_capacity().get()
                });
            let income = crate::cohort::uncontracted_monthly_income(
                cohort.annual_income(),
                cohort.people(),
                contracted_workers.get(&cohort.id()).copied().unwrap_or(0),
                capacity,
            )?;
            if income.minor_units() > 0 {
                findings.push(EconomicInvariantFinding {
                    severity: EconomicInvariantSeverity::KnownModelGap,
                    code: EconomicInvariantCode::PayerlessHouseholdIncome,
                    entity: EconomicInvariantEntity::Cohort(cohort.id()),
                    observed: i128::from(income.minor_units()),
                    allowed: 0,
                    audit_reference: "A-2 / AD-2",
                    explanation: "configured fallback income is credited without a paying account",
                });
            }
        }
        Ok(())
    }

    fn find_capacity_violations(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) -> Result<(), WorldError> {
        self.find_route_capacity_violations(findings);
        self.find_freight_capacity_violations(findings)?;
        self.find_terminal_capacity_violations(findings);
        Ok(())
    }

    fn find_route_capacity_violations(&self, findings: &mut Vec<EconomicInvariantFinding>) {
        for (route, quantity) in self.route_capacity.reserved() {
            let Some(definition) = self.logistics_routes.get(route) else {
                findings.push(unknown_capacity_finding(EconomicInvariantEntity::Route(
                    *route,
                )));
                continue;
            };
            push_capacity_excess(
                findings,
                EconomicInvariantCode::RouteCapacityExceeded,
                EconomicInvariantEntity::Route(*route),
                quantity.get(),
                definition.capacity().get(),
                "Z-6",
                "route reservations exceed declared route capacity",
            );
        }
    }

    fn find_freight_capacity_violations(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) -> Result<(), WorldError> {
        let mut active_contract_capacity = BTreeMap::<RouteId, u64>::new();
        for contract in self
            .freight_contracts
            .values()
            .filter(|contract| contract.status() == ContractStatus::Active)
        {
            add_workers(
                &mut active_contract_capacity,
                contract.route(),
                contract.reserved_capacity().get(),
                "active freight contract capacity",
            )?;
        }
        for (route, reserved) in &active_contract_capacity {
            let Some(definition) = self.logistics_routes.get(route) else {
                findings.push(unknown_capacity_finding(EconomicInvariantEntity::Route(
                    *route,
                )));
                continue;
            };
            push_capacity_excess(
                findings,
                EconomicInvariantCode::FreightContractCapacityExceeded,
                EconomicInvariantEntity::Route(*route),
                *reserved,
                definition.capacity().get(),
                "Z-5",
                "active freight contracts promise more than the route can carry",
            );
        }
        self.find_freight_pool_usage_violations(findings, &active_contract_capacity);
        Ok(())
    }

    fn find_freight_pool_usage_violations(
        &self,
        findings: &mut Vec<EconomicInvariantFinding>,
        active_contract_capacity: &BTreeMap<RouteId, u64>,
    ) {
        for (contract, used) in self.freight_capacity.contract_used() {
            let Some(definition) = self.freight_contracts.get(contract) else {
                findings.push(unknown_capacity_finding(EconomicInvariantEntity::Contract(
                    *contract,
                )));
                continue;
            };
            push_capacity_excess(
                findings,
                EconomicInvariantCode::FreightContractCapacityExceeded,
                EconomicInvariantEntity::Contract(*contract),
                used.get(),
                definition.reserved_capacity().get(),
                "Z-5",
                "freight contract usage exceeds its reserved capacity",
            );
        }
        for (route, used) in self.freight_capacity.spot_used() {
            let Some(definition) = self.logistics_routes.get(route) else {
                findings.push(unknown_capacity_finding(EconomicInvariantEntity::Route(
                    *route,
                )));
                continue;
            };
            let contracted = active_contract_capacity.get(route).copied().unwrap_or(0);
            let spot_capacity = definition.capacity().get().saturating_sub(contracted);
            push_capacity_excess(
                findings,
                EconomicInvariantCode::FreightSpotCapacityExceeded,
                EconomicInvariantEntity::Route(*route),
                used.get(),
                spot_capacity,
                "Z-5 / Z-6",
                "spot freight usage exceeds uncontracted route capacity",
            );
        }
    }

    fn find_terminal_capacity_violations(&self, findings: &mut Vec<EconomicInvariantFinding>) {
        for (terminal, used) in self.terminal_capacity.used() {
            let Some(definition) = self.terminals.get(terminal) else {
                findings.push(unknown_capacity_finding(EconomicInvariantEntity::Terminal(
                    *terminal,
                )));
                continue;
            };
            push_capacity_excess(
                findings,
                EconomicInvariantCode::TerminalCapacityExceeded,
                EconomicInvariantEntity::Terminal(*terminal),
                used.get(),
                definition.capacity().get(),
                "AC-4",
                "terminal usage exceeds declared terminal capacity",
            );
        }
    }

    fn find_known_model_gaps(
        claims: EconomicClaimsSummary,
        _commitments: EconomicCommitmentSummary,
        findings: &mut Vec<EconomicInvariantFinding>,
    ) {
        push_known_money_gap(
            findings,
            EconomicInvariantCode::HouseholdDebtHasNoCreditor,
            claims.household_debt,
            "A-1 / A-3",
            "household debt has no named creditor asset or payment recipient",
        );
        push_known_money_gap(
            findings,
            EconomicInvariantCode::PublicDebtHasNoCreditor,
            claims.public_debt,
            "P-3 / F-5",
            "public debt has no named creditor asset",
        );
    }
}

fn total_good_stock(row: Option<&GoodStockSummary>) -> Result<i128, WorldError> {
    let Some(row) = row else {
        return Ok(0);
    };
    i128::from(row.firm_inventory.get())
        .checked_add(i128::from(row.public_reserve.get()))
        .and_then(|value| value.checked_add(i128::from(row.in_transit.get())))
        .ok_or(WorldError::ArithmeticOverflow(
            "economic invariant goods stock",
        ))
}

fn checked_money_add(
    total: i128,
    amount: Money,
    context: &'static str,
) -> Result<i128, WorldError> {
    total
        .checked_add(i128::from(amount.minor_units()))
        .ok_or(WorldError::ArithmeticOverflow(context))
}

fn money_from_i128(value: i128, context: &'static str) -> Result<Money, WorldError> {
    i64::try_from(value)
        .map(Money::from_minor_units)
        .map_err(|_| WorldError::ArithmeticOverflow(context))
}

fn checked_quantity_add(
    current: QuantityMilli,
    amount: QuantityMilli,
    context: &'static str,
) -> Result<QuantityMilli, WorldError> {
    current
        .get()
        .checked_add(amount.get())
        .map(QuantityMilli::new)
        .ok_or(WorldError::ArithmeticOverflow(context))
}

fn add_workers<K: Ord>(
    map: &mut BTreeMap<K, u64>,
    key: K,
    workers: u64,
    context: &'static str,
) -> Result<(), WorldError> {
    let total = map.get(&key).copied().unwrap_or(0);
    map.insert(
        key,
        total
            .checked_add(workers)
            .ok_or(WorldError::ArithmeticOverflow(context))?,
    );
    Ok(())
}

fn push_negative_balance(
    findings: &mut Vec<EconomicInvariantFinding>,
    entity: EconomicInvariantEntity,
    balance: Money,
) {
    if balance.minor_units() < 0 {
        findings.push(EconomicInvariantFinding {
            severity: EconomicInvariantSeverity::HardViolation,
            code: EconomicInvariantCode::NegativeLiquidBalance,
            entity,
            observed: i128::from(balance.minor_units()),
            allowed: 0,
            audit_reference: "I-1",
            explanation: "liquid economic balance is negative",
        });
    }
}

fn push_capacity_excess(
    findings: &mut Vec<EconomicInvariantFinding>,
    code: EconomicInvariantCode,
    entity: EconomicInvariantEntity,
    observed: u64,
    allowed: u64,
    audit_reference: &'static str,
    explanation: &'static str,
) {
    if observed > allowed {
        findings.push(EconomicInvariantFinding {
            severity: EconomicInvariantSeverity::HardViolation,
            code,
            entity,
            observed: i128::from(observed),
            allowed: i128::from(allowed),
            audit_reference,
            explanation,
        });
    }
}

fn unknown_capacity_finding(entity: EconomicInvariantEntity) -> EconomicInvariantFinding {
    EconomicInvariantFinding {
        severity: EconomicInvariantSeverity::HardViolation,
        code: EconomicInvariantCode::UnknownCapacityReservation,
        entity,
        observed: 1,
        allowed: 0,
        audit_reference: "H-6",
        explanation: "capacity is reserved against an entity that is not registered",
    }
}

fn push_known_money_gap(
    findings: &mut Vec<EconomicInvariantFinding>,
    code: EconomicInvariantCode,
    amount: Money,
    audit_reference: &'static str,
    explanation: &'static str,
) {
    if amount.minor_units() > 0 {
        findings.push(EconomicInvariantFinding {
            severity: EconomicInvariantSeverity::KnownModelGap,
            code,
            entity: EconomicInvariantEntity::World,
            observed: i128::from(amount.minor_units()),
            allowed: 0,
            audit_reference,
            explanation,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, BasisPoints, ConsumptionProfile, ConsumptionTarget, Country, CountryIndicators,
        DemandBasis, EducationLevel, EmploymentAgreement, EmploymentStatus, Firm, FreightContract,
        HouseholdCohort, HouseholdType, LogisticsRoute, NeedProfileId, NeedTier, Population,
        ProductionRecipe, RecipeId, Region, RouteOperatingCost, ShipmentId, ShipmentOrder,
        TransportMode, WorldSeed,
    };

    fn minimal_world(people: u64, households: u64, debt: Money) -> World {
        let mut world = World::new(
            WorldSeed::new(7),
            SimDate::new(2025, 1).expect("valid date"),
        );
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register country");
        world
            .register_good(crate::Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("register good");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "People",
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
                    "R",
                    Population::new(people),
                    Money::from_minor_units(1_000),
                )
                .expect("region"),
            )
            .expect("register region");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(people),
                    households,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Unemployed,
                    Money::default(),
                    Money::from_minor_units(100),
                    debt,
                )
                .expect("cohort"),
            )
            .expect("register cohort");
        world
    }

    fn has_code(report: &EconomicInvariantReport, code: EconomicInvariantCode) -> bool {
        report.findings.iter().any(|finding| finding.code == code)
    }

    #[test]
    fn valid_minimal_world_has_no_hard_population_or_household_violation() {
        let world = minimal_world(10, 5, Money::default());
        let report = world.economic_invariant_report().expect("report");
        assert!(report.is_hard_valid());
        assert!(!has_code(
            &report,
            EconomicInvariantCode::RegionPopulationMismatch
        ));
        assert!(!has_code(
            &report,
            EconomicInvariantCode::InvalidHouseholdCount
        ));
    }

    #[test]
    fn report_detects_population_and_ghost_household_corruption() {
        let mut world = minimal_world(10, 5, Money::default());
        world
            .cohorts
            .get_mut(&CohortId::new(1))
            .expect("cohort")
            .set_people(Population::new(0));
        let report = world.economic_invariant_report().expect("report");
        assert!(!report.is_hard_valid());
        assert!(has_code(
            &report,
            EconomicInvariantCode::RegionPopulationMismatch
        ));
        assert!(has_code(
            &report,
            EconomicInvariantCode::InvalidHouseholdCount
        ));
    }

    #[test]
    fn report_detects_labor_overcommitment_without_mutating_the_agreement() {
        let mut world = minimal_world(2, 1, Money::default());
        world.employment_agreements.insert(
            (FirmId::new(1), CohortId::new(1)),
            EmploymentAgreement::new(
                FirmId::new(1),
                CohortId::new(1),
                3,
                Money::from_minor_units(10),
            )
            .expect("agreement"),
        );
        let before = world.employment_agreements.clone();
        let report = world.economic_invariant_report().expect("report");
        assert!(has_code(
            &report,
            EconomicInvariantCode::LaborOvercommitment
        ));
        assert_eq!(world.employment_agreements, before);
    }

    #[test]
    fn money_transition_names_an_unexplained_sink_and_authorized_retirement() {
        let mut world = minimal_world(10, 5, Money::default());
        let opening = world.economic_invariant_report().expect("opening report");
        world
            .cohorts
            .get_mut(&CohortId::new(1))
            .expect("cohort")
            .debit_wealth(Money::from_minor_units(1))
            .expect("diagnostic sink");
        let closing = world.economic_invariant_report().expect("closing report");

        let unexplained = reconcile_money_transition(&opening, &closing, Money::default())
            .expect("unexplained transition");
        assert_eq!(unexplained.unexplained_change, Money::from_minor_units(-1));
        assert!(!unexplained.is_reconciled());

        let authorized =
            reconcile_money_transition(&opening, &closing, Money::from_minor_units(-1))
                .expect("authorized retirement");
        assert!(authorized.is_reconciled());
    }

    #[test]
    fn payerless_household_and_public_debt_are_explicit_model_gaps() {
        let mut world = minimal_world(10, 5, Money::from_minor_units(25));
        let country = world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country");
        *country = country.clone().with_indicators(CountryIndicators::new(
            Money::default(),
            Money::from_minor_units(40),
            crate::BasisPoints::HALF,
            crate::BasisPoints::HALF,
        ));
        let report = world.economic_invariant_report().expect("report");
        assert_eq!(report.claims.household_debt, Money::from_minor_units(25));
        assert_eq!(report.claims.public_debt, Money::from_minor_units(40));
        assert!(has_code(
            &report,
            EconomicInvariantCode::HouseholdDebtHasNoCreditor
        ));
        assert!(has_code(
            &report,
            EconomicInvariantCode::PublicDebtHasNoCreditor
        ));
        assert!(report.is_hard_valid());
    }

    #[test]
    fn retired_fallback_income_is_named_as_payerless() {
        let mut world = minimal_world(10, 5, Money::default());
        world.cohorts.insert(
            CohortId::new(1),
            HouseholdCohort::new(
                CohortId::new(1),
                RegionId::new(1),
                NeedProfileId::new(1),
                Population::new(10),
                5,
                AgeBand::Senior,
                HouseholdType::Retired,
                EducationLevel::Secondary,
                EmploymentStatus::Retired,
                Money::from_minor_units(120),
                Money::from_minor_units(100),
                Money::default(),
            )
            .expect("retired cohort"),
        );

        let report = world.economic_invariant_report().expect("report");
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.code == EconomicInvariantCode::PayerlessHouseholdIncome)
            .expect("payerless income finding");
        assert_eq!(
            finding.entity,
            EconomicInvariantEntity::Cohort(CohortId::new(1))
        );
        assert_eq!(finding.observed, 10);
        assert_eq!(finding.severity, EconomicInvariantSeverity::KnownModelGap);

        world
            .execute_monthly_household_cashflows()
            .expect("payerless cashflow");
        let closing = world.economic_invariant_report().expect("closing report");
        let transition = reconcile_money_transition(&report, &closing, Money::default())
            .expect("money transition");
        assert_eq!(transition.unexplained_change, Money::from_minor_units(10));
    }

    #[test]
    fn household_debt_service_is_an_allowlisted_money_sink() {
        let mut world = minimal_world(10, 5, Money::from_minor_units(120));
        let opening = world.economic_invariant_report().expect("opening report");
        let rows = world
            .execute_monthly_household_cashflows()
            .expect("household cashflow");
        assert_eq!(rows[0].debt_service, Money::from_minor_units(1));
        let closing = world.economic_invariant_report().expect("closing report");
        let transition = reconcile_money_transition(&opening, &closing, Money::default())
            .expect("money transition");
        assert_eq!(transition.unexplained_change, Money::from_minor_units(-1));
    }

    #[test]
    fn goods_transition_detects_and_reconciles_duplicated_inventory() {
        let mut world = minimal_world(10, 5, Money::default());
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("register recipe");
        let mut inventory = BTreeMap::new();
        inventory.insert(GoodId::new(1), QuantityMilli::new(100));
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Farm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(100),
                    inventory,
                )
                .expect("firm"),
            )
            .expect("register firm");
        let opening = world.economic_invariant_report().expect("opening report");
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("firm")
            .credit_inventory(GoodId::new(1), QuantityMilli::new(100))
            .expect("duplicate inventory");
        let closing = world.economic_invariant_report().expect("closing report");

        let unexplained = reconcile_goods_transition(&opening, &closing, &BTreeMap::new())
            .expect("goods reconciliation");
        assert_eq!(unexplained.len(), 1);
        assert_eq!(unexplained[0].opening, 100);
        assert_eq!(unexplained[0].closing, 200);
        assert_eq!(unexplained[0].unexplained_change, 100);
        assert!(!unexplained[0].is_reconciled());

        let authorized = BTreeMap::from([(GoodId::new(1), 100)]);
        let reconciled = reconcile_goods_transition(&opening, &closing, &authorized)
            .expect("authorized goods change");
        assert!(reconciled[0].is_reconciled());
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "the fixture names every account and route in the causal freight circuit"
    )]
    fn formal_freight_operating_cost_is_an_allowlisted_money_sink() {
        let mut world = minimal_world(10, 5, Money::default());
        world
            .register_region(
                Region::new(
                    RegionId::new(2),
                    CountryId::new(1),
                    "Port",
                    Population::new(0),
                    Money::default(),
                )
                .expect("port"),
            )
            .expect("register port");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Freight fixture",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("register recipe");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Shipper",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(100),
                    BTreeMap::from([(GoodId::new(1), QuantityMilli::new(1_000))]),
                )
                .expect("shipper"),
            )
            .expect("register shipper");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Carrier",
                    RegionId::new(2),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(100),
                    BTreeMap::new(),
                )
                .expect("carrier"),
            )
            .expect("register carrier");
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(1),
                    RegionId::new(2),
                    TransportMode::Road,
                    QuantityMilli::new(1_000),
                    Money::from_minor_units(10),
                    2,
                    10_000,
                )
                .expect("route")
                .with_carrier(FirmId::new(2)),
            )
            .expect("register route");
        world
            .set_route_operating_cost(
                RouteId::new(1),
                RouteOperatingCost::new(
                    Money::from_minor_units(2),
                    Money::from_minor_units(3),
                    Money::from_minor_units(3),
                )
                .expect("operating cost"),
            )
            .expect("set operating cost");
        let opening = world.economic_invariant_report().expect("opening report");
        world
            .start_inventory_shipment(
                FirmId::new(1),
                FirmId::new(2),
                &ShipmentOrder::new(
                    ShipmentId::new(1),
                    GoodId::new(1),
                    RegionId::new(1),
                    RegionId::new(2),
                    QuantityMilli::new(1_000),
                    Money::from_minor_units(100),
                ),
                1,
            )
            .expect("start shipment");
        let closing = world.economic_invariant_report().expect("closing report");
        let transition = reconcile_money_transition(&opening, &closing, Money::default())
            .expect("money transition");
        assert_eq!(transition.unexplained_change, Money::from_minor_units(-8));
    }

    #[test]
    fn report_detects_contracts_that_overpromise_one_route() {
        let mut world = minimal_world(10, 5, Money::default());
        world
            .register_region(
                Region::new(
                    RegionId::new(2),
                    CountryId::new(1),
                    "Port",
                    Population::new(0),
                    Money::default(),
                )
                .expect("port"),
            )
            .expect("register port");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Freight service",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("register recipe");
        for (id, name) in [(1, "Shipper"), (2, "Carrier")] {
            world
                .register_firm(
                    Firm::new(
                        FirmId::new(id),
                        name,
                        RegionId::new(1),
                        RecipeId::new(1),
                        1,
                        1,
                        Money::from_minor_units(100),
                        BTreeMap::new(),
                    )
                    .expect("firm"),
                )
                .expect("register firm");
        }
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(1),
                    RegionId::new(2),
                    TransportMode::Road,
                    QuantityMilli::new(1_000),
                    Money::from_minor_units(1),
                    1,
                    10_000,
                )
                .expect("route")
                .with_carrier(FirmId::new(2)),
            )
            .expect("register route");
        for id in [1, 2] {
            world
                .register_freight_contract(
                    FreightContract::new(
                        ContractId::new(id),
                        FirmId::new(1),
                        FirmId::new(2),
                        RouteId::new(1),
                        QuantityMilli::new(700),
                        BasisPoints::ZERO,
                        1,
                        12,
                    )
                    .expect("contract"),
                )
                .expect("register contract");
            world
                .activate_freight_contract(ContractId::new(id))
                .expect("activate contract");
        }

        let report = world.economic_invariant_report().expect("report");
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.code == EconomicInvariantCode::FreightContractCapacityExceeded
                    && finding.entity == EconomicInvariantEntity::Route(RouteId::new(1))
            })
            .expect("overcommitment finding");
        assert_eq!(finding.observed, 1_400);
        assert_eq!(finding.allowed, 1_000);
        assert_eq!(finding.severity, EconomicInvariantSeverity::HardViolation);
    }

    #[test]
    fn cloned_worlds_produce_identical_canonically_ordered_reports() {
        let world = minimal_world(10, 5, Money::from_minor_units(25));
        let replayed = world.clone();
        assert_eq!(
            world.economic_invariant_report().expect("direct report"),
            replayed
                .economic_invariant_report()
                .expect("replayed report")
        );
    }
}
