use std::collections::BTreeMap;

use crate::{ActorId, BasisPoints, CountryId, DomainEvent, RegionId, World};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProcurementShortfallCause {
    Other,
    RouteCapacity,
}

/// One deterministic yearly narrative summary derived only from authoritative domain events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronicleEntry {
    pub year: i32,
    pub importance: u16,
    pub text: String,
}

impl World {
    /// Builds a compact yearly chronicle without changing simulation state.
    #[must_use]
    pub fn chronicle(&self) -> Vec<ChronicleEntry> {
        let mut years = BTreeMap::<i32, YearSummary>::new();
        let mut actor_names = BTreeMap::<ActorId, String>::new();
        let mut country_names = BTreeMap::<CountryId, String>::new();
        let mut region_names = BTreeMap::<RegionId, String>::new();
        for envelope in self.events().events() {
            match envelope.event() {
                DomainEvent::CountryRegistered { country, name } => {
                    country_names.insert(*country, name.clone());
                }
                DomainEvent::ActorRegistered { actor, name, .. } => {
                    actor_names.insert(*actor, name.clone());
                }
                DomainEvent::RegionRegistered { region, name, .. } => {
                    region_names.insert(*region, name.clone());
                }
                _ => {}
            }
            let row = years.entry(envelope.date().year()).or_default();
            row.observe(envelope.event());
        }
        years
            .into_iter()
            .filter_map(|(year, summary)| {
                summary.finish(
                    year,
                    &actor_names,
                    &country_names,
                    &region_names,
                    self.firms(),
                    self.goods(),
                )
            })
            .collect()
    }
}

#[derive(Default)]
struct YearSummary {
    months: u64,
    relief_by_actor: BTreeMap<ActorId, i128>,
    target_changes_by_actor: BTreeMap<ActorId, u64>,
    rationing_by_region: BTreeMap<RegionId, u64>,
    reserve_policy_changes: u64,
    reserve_coverage_increases: u64,
    reserve_coverage_decreases: u64,
    reserve_budget_increases: u64,
    reserve_budget_decreases: u64,
    leading_reserve_policy_change: Option<(CountryId, u8, u8, u16, u16)>,
    reserve_priority_changes: u64,
    reserve_priority_increases: u64,
    reserve_priority_decreases: u64,
    reserve_import_reliance_priority_changes: u64,
    leading_reserve_priority_change: Option<(CountryId, RegionId, u16, u16)>,
    reserve_maintenance_entries: u64,
    reserve_maintenance_assessed_minor: i128,
    reserve_maintenance_paid_minor: i128,
    reserve_baseline_spoilage_milli: u128,
    reserve_neglect_spoilage_milli: u128,
    reserve_requirements_reviewed: u64,
    reserve_prioritized_reviews: u64,
    reserve_target_milli: u128,
    reserve_opening_stock_milli: u128,
    reserve_available_supply_milli: u128,
    reserve_remaining_gap_milli: u128,
    reserve_supply_limited_reviews: u64,
    reserve_budget_limited_reviews: u64,
    reserve_procurements: u64,
    reserve_procured_milli: u128,
    reserve_spending_minor: i128,
    reserve_procurement_by_actor: BTreeMap<ActorId, i128>,
    reserve_procurement_by_region: BTreeMap<RegionId, u128>,
    reserve_distributions: u64,
    reserve_distributed_milli: u128,
    reserve_distribution_by_region: BTreeMap<RegionId, u128>,
    hostility_changes: Vec<(CountryId, CountryId, bool)>,
    grievances_by_pair: BTreeMap<(CountryId, CountryId), (u16, u16)>,
    procurement_shortfalls:
        BTreeMap<(crate::FirmId, crate::GoodId, ProcurementShortfallCause), u128>,
    freight_payments: u64,
    freight_revenue_minor: i128,
    route_expansions: u64,
    route_capacity_added_milli: u128,
    route_investment_minor: i128,
    firms_founded: u64,
    firm_entry_capital_cost_minor: i128,
    firm_entry_working_capital_minor: i128,
    firm_entry_jobs: u64,
    leading_firm_entry: Option<(ActorId, crate::FirmId, RegionId, crate::GoodId)>,
    labor_market_observations: u64,
    labor_offers: u64,
    labor_hires: u64,
    labor_available_workers: u64,
    labor_vacancies: u64,
    labor_wage_pressure_increases: u64,
    labor_wage_pressure_decreases: u64,
    labor_wage_adjustment_basis_points: i64,
    maximum_unemployment_pressure_months: u8,
    maximum_vacancy_pressure_months: u8,
    leading_unemployment_region: Option<RegionId>,
    leading_vacancy_region: Option<RegionId>,
    firm_recapitalizations: u64,
    firm_recapitalization_minor: i128,
    autonomous_credit_acceptances: u64,
    autonomous_credit_principal_minor: i128,
    autonomous_credit_searches: u64,
    autonomous_credit_unfunded_searches: u64,
    lender_principal_repaid_minor: i128,
    lender_interest_income_minor: i128,
    lender_realized_loss_minor: i128,
    lender_successful_loans: u64,
    lender_defaulted_loans: u64,
    firm_debt_service_due_minor: i128,
    firm_debt_service_paid_minor: i128,
    firm_interest_charged_minor: i128,
    firm_interest_paid_minor: i128,
    firm_debt_service_attempts: u64,
    firm_overdue_debt_attempts: u64,
    firm_distress_downsizings: u64,
    distress_workers_released: u64,
    firm_insolvencies: u64,
    insolvent_wage_claims_minor: i128,
    insolvent_inventory_milli: u128,
    firm_reorganizations: u64,
    firm_liquidations: u64,
    liquidation_claims_paid_minor: i128,
    liquidation_claims_written_off_minor: i128,
    liquidation_creditor_paid_minor: i128,
    liquidation_creditor_written_off_minor: i128,
    liquidation_inventory_sold_milli: u128,
    liquidation_inventory_sale_proceeds_minor: i128,
    liquidation_inventory_written_off_milli: u128,
    liquidation_capacity_sold_batches: u64,
    liquidation_capacity_sale_proceeds_minor: i128,
    liquidation_capacity_written_off_batches: u64,
    liquidation_owner_distribution_minor: i128,
    reorganization_contributions_minor: i128,
    reorganization_claims_paid_minor: i128,
    reorganized_workers: u64,

    production_milli: u128,
    traded_milli: u128,
    household_borrowing_minor: i128,
    relief_minor: i128,
    relief_debt_minor: i128,
    excess_deaths: u64,
    minimum_survival_bps: Option<u16>,
    rationing_actions: u64,
    rationed_requested_milli: u128,
    rationed_available_milli: u128,
    politics_changes: u64,
    fiscal_revenue_minor: i128,
    fiscal_spending_minor: i128,
    closing_debt_minor: i128,
    closed_countries: u64,
    debt_interest_minor: i128,
    indebted_countries: u64,
    debt_restructuring_count: u64,
    debt_principal_written_off_minor: i128,
    measured_regions: u64,
    final_consumption_minor: i128,
    inventory_change_minor: i128,
    measured_output_minor: i128,
    completed: bool,
}

impl YearSummary {
    fn observe(&mut self, event: &DomainEvent) {
        if self.observe_procurement_event(event)
            || self.observe_public_reserve_event(event)
            || self.observe_freight_event(event)
            || self.observe_route_expansion_event(event)
            || self.observe_firm_entry_event(event)
            || self.observe_labor_market_event(event)
            || self.observe_firm_debt_service_event(event)
            || self.observe_firm_distress_event(event)
            || self.observe_fiscal_event(event)
        {
            return;
        }
        match event {
            DomainEvent::MonthlyEconomicCycleCompleted { .. } => self.months += 1,
            DomainEvent::FirmProductionTargetSet { actor, .. } => {
                *self.target_changes_by_actor.entry(*actor).or_default() += 1;
            }
            DomainEvent::ProductionCompleted { quantity, .. } => {
                self.production_milli += u128::from(quantity.get());
            }
            DomainEvent::MarketTrade { quantity, .. } => {
                self.traded_milli += u128::from(quantity.get());
            }
            DomainEvent::HouseholdSurvivalBorrowed { amount, .. } => {
                self.household_borrowing_minor += i128::from(amount.minor_units());
            }
            DomainEvent::EmergencyReliefFunded { actor, amount, .. } => {
                self.relief_minor += i128::from(amount.minor_units());
                *self.relief_by_actor.entry(*actor).or_default() +=
                    i128::from(amount.minor_units());
            }
            DomainEvent::EmergencyReliefDebtIssued { amount, .. } => {
                self.relief_debt_minor += i128::from(amount.minor_units());
            }
            DomainEvent::CohortHealthUpdated {
                survival_fulfillment,
                excess_deaths,
                ..
            } => {
                self.excess_deaths = self.excess_deaths.saturating_add(*excess_deaths);
                self.minimum_survival_bps = Some(
                    self.minimum_survival_bps
                        .map_or(survival_fulfillment.get(), |current| {
                            current.min(survival_fulfillment.get())
                        }),
                );
            }
            DomainEvent::BilateralHostilityChanged {
                first,
                second,
                active,
            } => self.hostility_changes.push((*first, *second, *active)),
            DomainEvent::BilateralGrievanceChanged {
                aggrieved,
                target,
                level,
            } => {
                let entry = self
                    .grievances_by_pair
                    .entry((*aggrieved, *target))
                    .or_insert((level.get(), level.get()));
                entry.1 = level.get();
            }
            DomainEvent::SurvivalRationingApplied {
                region,
                requested,
                available,
                ..
            } => {
                self.rationing_actions += 1;
                *self.rationing_by_region.entry(*region).or_default() += 1;
                self.rationed_requested_milli += u128::from(requested.get());
                self.rationed_available_milli += u128::from(available.get());
            }
            DomainEvent::RegionalOutputMeasured {
                final_consumption,
                inventory_change,
                annual_output,
                ..
            } => {
                self.measured_regions += 1;
                self.final_consumption_minor += i128::from(final_consumption.minor_units());
                self.inventory_change_minor += i128::from(inventory_change.minor_units());
                self.measured_output_minor += i128::from(annual_output.minor_units());
            }
            DomainEvent::EconomicYearCompleted { .. } => self.completed = true,
            _ => {}
        }
    }

    fn observe_firm_debt_service_event(&mut self, event: &DomainEvent) -> bool {
        let DomainEvent::FirmDebtServiceSettled {
            due,
            paid,
            interest_charged,
            interest_paid,
            overdue,
            ..
        } = event
        else {
            return false;
        };
        self.firm_debt_service_attempts = self.firm_debt_service_attempts.saturating_add(1);
        self.firm_debt_service_due_minor += i128::from(due.minor_units());
        self.firm_debt_service_paid_minor += i128::from(paid.minor_units());
        self.firm_interest_charged_minor += i128::from(interest_charged.minor_units());
        self.firm_interest_paid_minor += i128::from(interest_paid.minor_units());
        if *overdue {
            self.firm_overdue_debt_attempts = self.firm_overdue_debt_attempts.saturating_add(1);
        }
        true
    }

    #[allow(clippy::too_many_lines)]
    fn observe_firm_distress_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::LenderCreditHistoryUpdated {
                principal_repaid,
                interest_income,
                realized_loss,
                resolved,
                defaulted,
                ..
            } => {
                self.lender_principal_repaid_minor += i128::from(principal_repaid.minor_units());
                self.lender_interest_income_minor += i128::from(interest_income.minor_units());
                self.lender_realized_loss_minor += i128::from(realized_loss.minor_units());
                if *resolved {
                    if *defaulted {
                        self.lender_defaulted_loans = self.lender_defaulted_loans.saturating_add(1);
                    } else {
                        self.lender_successful_loans =
                            self.lender_successful_loans.saturating_add(1);
                    }
                }
            }
            DomainEvent::FirmCreditOfferAccepted { principal, .. } => {
                self.autonomous_credit_acceptances =
                    self.autonomous_credit_acceptances.saturating_add(1);
                self.autonomous_credit_principal_minor += i128::from(principal.minor_units());
            }
            DomainEvent::ObservedFirmCreditMarketCompleted {
                borrowers_reviewed,
                offers_accepted,
                ..
            } => {
                self.autonomous_credit_searches = self
                    .autonomous_credit_searches
                    .saturating_add(*borrowers_reviewed);
                self.autonomous_credit_unfunded_searches = self
                    .autonomous_credit_unfunded_searches
                    .saturating_add(borrowers_reviewed.saturating_sub(*offers_accepted));
            }
            DomainEvent::FirmRecapitalized { amount, .. } => {
                self.firm_recapitalizations = self.firm_recapitalizations.saturating_add(1);
                self.firm_recapitalization_minor += i128::from(amount.minor_units());
            }
            DomainEvent::FirmDownsizedForDistress {
                previous_workers,
                current_workers,
                ..
            } => {
                self.firm_distress_downsizings = self.firm_distress_downsizings.saturating_add(1);
                self.distress_workers_released = self
                    .distress_workers_released
                    .saturating_add(previous_workers.saturating_sub(*current_workers));
            }
            DomainEvent::FirmInsolvencyDeclared {
                wage_arrears,
                inventory,
                ..
            } => {
                self.firm_insolvencies = self.firm_insolvencies.saturating_add(1);
                self.insolvent_wage_claims_minor += i128::from(wage_arrears.minor_units());
                self.insolvent_inventory_milli = self.insolvent_inventory_milli.saturating_add(
                    inventory
                        .iter()
                        .map(|(_, quantity)| u128::from(quantity.get()))
                        .sum(),
                );
            }
            DomainEvent::FirmCreditorClaimSettled {
                paid, written_off, ..
            } => {
                self.liquidation_creditor_paid_minor += i128::from(paid.minor_units());
                self.liquidation_creditor_written_off_minor +=
                    i128::from(written_off.minor_units());
            }
            DomainEvent::FirmLiquidationInventorySold {
                quantity, proceeds, ..
            } => {
                self.liquidation_inventory_sold_milli = self
                    .liquidation_inventory_sold_milli
                    .saturating_add(u128::from(quantity.get()));
                self.liquidation_inventory_sale_proceeds_minor +=
                    i128::from(proceeds.minor_units());
            }
            DomainEvent::FirmLiquidationCapacitySold {
                capacity_batches,
                proceeds,
                ..
            } => {
                self.liquidation_capacity_sold_batches = self
                    .liquidation_capacity_sold_batches
                    .saturating_add(*capacity_batches);
                self.liquidation_capacity_sale_proceeds_minor += i128::from(proceeds.minor_units());
            }
            DomainEvent::FirmLiquidated {
                claims_paid,
                claims_written_off,
                inventory_written_off,
                capacity_written_off,
                owner_distribution,
                ..
            } => {
                self.firm_liquidations = self.firm_liquidations.saturating_add(1);
                self.liquidation_claims_paid_minor += i128::from(claims_paid.minor_units());
                self.liquidation_claims_written_off_minor +=
                    i128::from(claims_written_off.minor_units());
                self.liquidation_inventory_written_off_milli =
                    self.liquidation_inventory_written_off_milli.saturating_add(
                        inventory_written_off
                            .iter()
                            .map(|(_, quantity)| u128::from(quantity.get()))
                            .sum(),
                    );
                self.liquidation_capacity_written_off_batches = self
                    .liquidation_capacity_written_off_batches
                    .saturating_add(*capacity_written_off);
                self.liquidation_owner_distribution_minor +=
                    i128::from(owner_distribution.minor_units());
            }
            DomainEvent::FirmReorganized {
                contribution,
                claims_paid,
                workers,
                ..
            } => {
                self.firm_reorganizations = self.firm_reorganizations.saturating_add(1);
                self.reorganization_contributions_minor += i128::from(contribution.minor_units());
                self.reorganization_claims_paid_minor += i128::from(claims_paid.minor_units());
                self.reorganized_workers = self.reorganized_workers.saturating_add(*workers);
            }
            _ => return false,
        }
        true
    }

    fn observe_fiscal_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::PublicDebtInterestCharged { interest, .. } => {
                self.indebted_countries += 1;
                self.debt_interest_minor += i128::from(interest.minor_units());
            }
            DomainEvent::PublicDebtRestructured {
                principal_written_off,
                ..
            } => {
                self.debt_restructuring_count += 1;
                self.debt_principal_written_off_minor +=
                    i128::from(principal_written_off.minor_units());
            }
            DomainEvent::CountryFiscalYearClosed {
                revenue,
                spending,
                debt,
                ..
            } => {
                self.fiscal_revenue_minor += i128::from(revenue.minor_units());
                self.fiscal_spending_minor += i128::from(spending.minor_units());
                self.closing_debt_minor += i128::from(debt.minor_units());
                self.closed_countries += 1;
            }
            DomainEvent::CountryPoliticsChanged { .. } => self.politics_changes += 1,
            _ => return false,
        }
        true
    }

    fn observe_firm_entry_event(&mut self, event: &DomainEvent) -> bool {
        let DomainEvent::FirmFoundedFromOpportunity {
            founder,
            firm,
            region,
            good,
            capital_cost,
            working_capital,
            ..
        } = event
        else {
            return false;
        };
        self.firms_founded = self.firms_founded.saturating_add(1);
        self.firm_entry_capital_cost_minor += i128::from(capital_cost.minor_units());
        self.firm_entry_working_capital_minor += i128::from(working_capital.minor_units());
        self.firm_entry_jobs = self.firm_entry_jobs.saturating_add(1);
        self.leading_firm_entry
            .get_or_insert((*founder, *firm, *region, *good));
        true
    }

    fn observe_labor_market_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::EmploymentMatched {
                labor_market_adjustment_basis_points,
                ..
            } => {
                self.labor_wage_adjustment_basis_points +=
                    i64::from(*labor_market_adjustment_basis_points);
                match labor_market_adjustment_basis_points.cmp(&0) {
                    std::cmp::Ordering::Greater => {
                        self.labor_wage_pressure_increases =
                            self.labor_wage_pressure_increases.saturating_add(1);
                    }
                    std::cmp::Ordering::Less => {
                        self.labor_wage_pressure_decreases =
                            self.labor_wage_pressure_decreases.saturating_add(1);
                    }
                    std::cmp::Ordering::Equal => {}
                }
            }
            DomainEvent::RegionalLaborMarketObserved {
                region,
                available_workers,
                vacancies,
                offers,
                hires,
                unemployment_pressure_months,
                vacancy_pressure_months,
                ..
            } => {
                self.labor_market_observations = self.labor_market_observations.saturating_add(1);
                self.labor_offers = self.labor_offers.saturating_add(*offers);
                self.labor_hires = self.labor_hires.saturating_add(*hires);
                self.labor_available_workers = self
                    .labor_available_workers
                    .saturating_add(*available_workers);
                self.labor_vacancies = self.labor_vacancies.saturating_add(*vacancies);
                if *unemployment_pressure_months > self.maximum_unemployment_pressure_months {
                    self.maximum_unemployment_pressure_months = *unemployment_pressure_months;
                    self.leading_unemployment_region = Some(*region);
                }
                if *vacancy_pressure_months > self.maximum_vacancy_pressure_months {
                    self.maximum_vacancy_pressure_months = *vacancy_pressure_months;
                    self.leading_vacancy_region = Some(*region);
                }
            }
            _ => return false,
        }
        true
    }

    fn observe_route_expansion_event(&mut self, event: &DomainEvent) -> bool {
        let DomainEvent::RouteCapacityExpanded {
            added_capacity,
            cost,
            ..
        } = event
        else {
            return false;
        };
        self.route_expansions = self.route_expansions.saturating_add(1);
        self.route_capacity_added_milli = self
            .route_capacity_added_milli
            .saturating_add(u128::from(added_capacity.get()));
        self.route_investment_minor += i128::from(cost.minor_units());
        true
    }

    fn observe_public_reserve_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::GovernmentReservePolicyReviewed {
                country,
                previous_coverage_months,
                new_coverage_months,
                previous_monthly_budget,
                new_monthly_budget,
                ..
            } => self.observe_reserve_policy_change(
                *country,
                *previous_coverage_months,
                *new_coverage_months,
                previous_monthly_budget.get(),
                new_monthly_budget.get(),
            ),
            DomainEvent::GovernmentReservePriorityReviewed {
                country,
                region,
                previous_priority,
                new_priority,
                revision_reason,
                ..
            } => self.observe_reserve_priority_change(
                *country,
                *region,
                previous_priority.get(),
                new_priority.get(),
                *revision_reason,
            ),
            DomainEvent::GovernmentReserveMaintained {
                assessed_cost,
                paid_cost,
                baseline_spoilage,
                neglect_spoilage,
                ..
            } => self.observe_reserve_maintenance(
                *assessed_cost,
                *paid_cost,
                *baseline_spoilage,
                *neglect_spoilage,
            ),
            DomainEvent::GovernmentReserveRequirementReviewed {
                priority,
                target_stock,
                opening_stock,
                available_supply,
                remaining_gap,
                supply_limited,
                budget_limited,
                ..
            } => self.observe_reserve_requirement((
                *priority,
                *target_stock,
                *opening_stock,
                *available_supply,
                *remaining_gap,
                *supply_limited,
                *budget_limited,
            )),
            DomainEvent::GovernmentReserveProcured {
                actor,
                region,
                quantity,
                cost,
                ..
            } => {
                self.reserve_procurements = self.reserve_procurements.saturating_add(1);
                self.reserve_procured_milli = self
                    .reserve_procured_milli
                    .saturating_add(u128::from(quantity.get()));
                self.reserve_spending_minor += i128::from(cost.minor_units());
                let actor_total = self.reserve_procurement_by_actor.entry(*actor).or_default();
                *actor_total += i128::from(cost.minor_units());
                let region_total = self
                    .reserve_procurement_by_region
                    .entry(*region)
                    .or_default();
                *region_total = region_total.saturating_add(u128::from(quantity.get()));
            }
            DomainEvent::GovernmentReserveDistributed {
                region, quantity, ..
            } => {
                self.reserve_distributions = self.reserve_distributions.saturating_add(1);
                self.reserve_distributed_milli = self
                    .reserve_distributed_milli
                    .saturating_add(u128::from(quantity.get()));
                let region_total = self
                    .reserve_distribution_by_region
                    .entry(*region)
                    .or_default();
                *region_total = region_total.saturating_add(u128::from(quantity.get()));
            }
            _ => return false,
        }
        true
    }

    fn observe_freight_event(&mut self, event: &DomainEvent) -> bool {
        let (DomainEvent::MarketFreightPaid { amount, .. }
        | DomainEvent::FirmProcurementFreightPaid { amount, .. }) = event
        else {
            return false;
        };
        self.freight_payments = self.freight_payments.saturating_add(1);
        self.freight_revenue_minor += i128::from(amount.minor_units());
        true
    }

    fn observe_procurement_event(&mut self, event: &DomainEvent) -> bool {
        let (buyer, good, quantity, cause) = match event {
            DomainEvent::FirmProcurementShortfall {
                buyer,
                good,
                quantity,
            } => (*buyer, *good, *quantity, ProcurementShortfallCause::Other),
            DomainEvent::FirmProcurementRouteCapacityShortfall {
                buyer,
                good,
                quantity,
            } => (
                *buyer,
                *good,
                *quantity,
                ProcurementShortfallCause::RouteCapacity,
            ),
            _ => return false,
        };
        self.observe_procurement_shortfall(buyer, good, quantity, cause);
        true
    }

    fn observe_procurement_shortfall(
        &mut self,
        buyer: crate::FirmId,
        good: crate::GoodId,
        quantity: crate::QuantityMilli,
        cause: ProcurementShortfallCause,
    ) {
        let total = self
            .procurement_shortfalls
            .entry((buyer, good, cause))
            .or_default();
        *total = total.saturating_add(u128::from(quantity.get()));
    }

    fn finish(
        self,
        year: i32,
        actor_names: &BTreeMap<ActorId, String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
        firms: &BTreeMap<crate::FirmId, crate::Firm>,
        goods: &BTreeMap<crate::GoodId, crate::Good>,
    ) -> Option<ChronicleEntry> {
        if self.months == 0 && !self.completed {
            return None;
        }
        let mut sentences = Vec::new();
        if let Some(fulfillment) = self.minimum_survival_bps {
            if fulfillment < 10_000 {
                sentences.push(format!(
                    "Survival consumption fell as low as {}.{:02}%.",
                    fulfillment / 100,
                    fulfillment % 100
                ));
            }
        }
        if self.excess_deaths > 0 {
            sentences.push(format!(
                "Insufficient survival consumption was followed by {} excess deaths.",
                self.excess_deaths
            ));
        }
        if self.household_borrowing_minor > 0 {
            sentences.push(format!(
                "Households borrowed {} minor currency units for survival purchases.",
                self.household_borrowing_minor
            ));
        }
        if self.rationing_actions > 0 {
            sentences.push(format!(
                "Officials rationed {} of {} requested milli-units across {} local shortages.",
                self.rationed_available_milli,
                self.rationed_requested_milli,
                self.rationing_actions
            ));
        }
        self.push_public_reserve_narration(
            &mut sentences,
            actor_names,
            country_names,
            region_names,
        );
        self.push_conflict_narration(&mut sentences, country_names);
        self.push_procurement_shortfall_narration(&mut sentences, firms, goods);
        self.push_firm_entry_narration(&mut sentences, actor_names, region_names, firms, goods);
        self.push_labor_market_narration(&mut sentences, region_names);
        self.push_transport_narration(&mut sentences);
        self.push_firm_distress_narration(&mut sentences);
        if self.relief_minor > 0 {
            sentences.push(format!(
                "Political offices transferred {} minor currency units in emergency relief, backed by {} of new public debt.",
                self.relief_minor, self.relief_debt_minor
            ));
        }
        if self.production_milli > 0 || self.traded_milli > 0 {
            sentences.push(format!(
                "Firms produced {} and households bought {} milli-units through local markets.",
                self.production_milli, self.traded_milli
            ));
        }
        if self.measured_regions > 0 {
            sentences.push(format!(
                "Annual accounts measured {} output from {} final consumption and {} inventory change across {} regions.",
                self.measured_output_minor,
                self.final_consumption_minor,
                self.inventory_change_minor,
                self.measured_regions
            ));
        }
        if self.debt_interest_minor > 0 {
            sentences.push(format!(
                "Public debt service charged {} minor currency units of interest across {} indebted countries.",
                self.debt_interest_minor, self.indebted_countries
            ));
        }
        self.push_debt_restructuring_narration(&mut sentences);
        if self.closed_countries > 0 {
            sentences.push(format!(
                "Treasuries collected {} and spent {} minor currency units, closing the year with {} of public debt across {} countries.",
                self.fiscal_revenue_minor,
                self.fiscal_spending_minor,
                self.closing_debt_minor,
                self.closed_countries
            ));
        }
        if self.politics_changes > 0 {
            sentences.push(format!(
                "Political indicators changed in {} countries during annual closure.",
                self.politics_changes
            ));
        }
        self.push_named_attributions(&mut sentences, actor_names, region_names);
        if sentences.is_empty() {
            sentences.push(format!(
                "{} monthly cycles closed without a material event.",
                self.months
            ));
        }
        Some(ChronicleEntry {
            year,
            importance: self.importance(),
            text: sentences.join(" "),
        })
    }

    fn observe_reserve_maintenance(
        &mut self,
        assessed_cost: crate::Money,
        paid_cost: crate::Money,
        baseline_spoilage: crate::QuantityMilli,
        neglect_spoilage: crate::QuantityMilli,
    ) {
        self.reserve_maintenance_entries = self.reserve_maintenance_entries.saturating_add(1);
        self.reserve_maintenance_assessed_minor += i128::from(assessed_cost.minor_units());
        self.reserve_maintenance_paid_minor += i128::from(paid_cost.minor_units());
        self.reserve_baseline_spoilage_milli = self
            .reserve_baseline_spoilage_milli
            .saturating_add(u128::from(baseline_spoilage.get()));
        self.reserve_neglect_spoilage_milli = self
            .reserve_neglect_spoilage_milli
            .saturating_add(u128::from(neglect_spoilage.get()));
    }

    fn observe_reserve_requirement(
        &mut self,
        evidence: (
            BasisPoints,
            crate::QuantityMilli,
            crate::QuantityMilli,
            crate::QuantityMilli,
            crate::QuantityMilli,
            bool,
            bool,
        ),
    ) {
        let (
            priority,
            target_stock,
            opening_stock,
            available_supply,
            remaining_gap,
            supply_limited,
            budget_limited,
        ) = evidence;
        self.reserve_requirements_reviewed = self.reserve_requirements_reviewed.saturating_add(1);
        if priority != BasisPoints::FULL {
            self.reserve_prioritized_reviews = self.reserve_prioritized_reviews.saturating_add(1);
        }
        self.reserve_target_milli = self
            .reserve_target_milli
            .saturating_add(u128::from(target_stock.get()));
        self.reserve_opening_stock_milli = self
            .reserve_opening_stock_milli
            .saturating_add(u128::from(opening_stock.get()));
        self.reserve_available_supply_milli = self
            .reserve_available_supply_milli
            .saturating_add(u128::from(available_supply.get()));
        self.reserve_remaining_gap_milli = self
            .reserve_remaining_gap_milli
            .saturating_add(u128::from(remaining_gap.get()));
        if supply_limited {
            self.reserve_supply_limited_reviews =
                self.reserve_supply_limited_reviews.saturating_add(1);
        }
        if budget_limited {
            self.reserve_budget_limited_reviews =
                self.reserve_budget_limited_reviews.saturating_add(1);
        }
    }

    fn observe_reserve_priority_change(
        &mut self,
        country: CountryId,
        region: RegionId,
        previous: u16,
        new: u16,
        reason: crate::ReservePriorityRevisionReason,
    ) {
        if previous == new {
            return;
        }
        self.reserve_priority_changes = self.reserve_priority_changes.saturating_add(1);
        if new > previous {
            self.reserve_priority_increases = self.reserve_priority_increases.saturating_add(1);
        } else {
            self.reserve_priority_decreases = self.reserve_priority_decreases.saturating_add(1);
        }
        if reason == crate::ReservePriorityRevisionReason::ImportReliance {
            self.reserve_import_reliance_priority_changes = self
                .reserve_import_reliance_priority_changes
                .saturating_add(1);
        }
        self.leading_reserve_priority_change
            .get_or_insert((country, region, previous, new));
    }

    fn observe_reserve_policy_change(
        &mut self,
        country: CountryId,
        previous_coverage: u8,
        new_coverage: u8,
        previous_budget: u16,
        new_budget: u16,
    ) {
        if previous_coverage == new_coverage && previous_budget == new_budget {
            return;
        }
        self.reserve_policy_changes = self.reserve_policy_changes.saturating_add(1);
        match new_coverage.cmp(&previous_coverage) {
            std::cmp::Ordering::Greater => {
                self.reserve_coverage_increases = self.reserve_coverage_increases.saturating_add(1);
            }
            std::cmp::Ordering::Less => {
                self.reserve_coverage_decreases = self.reserve_coverage_decreases.saturating_add(1);
            }
            std::cmp::Ordering::Equal => {}
        }
        match new_budget.cmp(&previous_budget) {
            std::cmp::Ordering::Greater => {
                self.reserve_budget_increases = self.reserve_budget_increases.saturating_add(1);
            }
            std::cmp::Ordering::Less => {
                self.reserve_budget_decreases = self.reserve_budget_decreases.saturating_add(1);
            }
            std::cmp::Ordering::Equal => {}
        }
        self.leading_reserve_policy_change.get_or_insert((
            country,
            previous_coverage,
            new_coverage,
            previous_budget,
            new_budget,
        ));
    }

    fn push_firm_entry_narration(
        &self,
        sentences: &mut Vec<String>,
        actor_names: &BTreeMap<ActorId, String>,
        region_names: &BTreeMap<RegionId, String>,
        firms: &BTreeMap<crate::FirmId, crate::Firm>,
        goods: &BTreeMap<crate::GoodId, crate::Good>,
    ) {
        let Some((founder, firm, region, good)) = self.leading_firm_entry else {
            return;
        };
        let founder_name = actor_names
            .get(&founder)
            .map_or_else(|| format!("actor {}", founder.get()), Clone::clone);
        let firm_name = firms.get(&firm).map_or_else(
            || format!("firm {}", firm.get()),
            |value| value.name().to_owned(),
        );
        let region_name = region_names
            .get(&region)
            .map_or_else(|| format!("region {}", region.get()), Clone::clone);
        let good_name = goods.get(&good).map_or_else(
            || format!("good {}", good.get()),
            |value| value.name().to_owned(),
        );
        sentences.push(format!(
            "Persistent shortages drew {} new local producers, committing {} minor currency units to installed capacity and {} to working capital while opening {} initial jobs. {} founded {} in {} to produce {}.",
            self.firms_founded,
            self.firm_entry_capital_cost_minor,
            self.firm_entry_working_capital_minor,
            self.firm_entry_jobs,
            founder_name,
            firm_name,
            region_name,
            good_name
        ));
    }

    fn push_labor_market_narration(
        &self,
        sentences: &mut Vec<String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if self.labor_market_observations == 0 {
            return;
        }
        sentences.push(format!(
            "Competitive labor markets recorded {} funded offers and {} hires across {} regional monthly observations, with {} residual vacancies and {} available workers observed after matching.",
            self.labor_offers,
            self.labor_hires,
            self.labor_market_observations,
            self.labor_vacancies,
            self.labor_available_workers
        ));
        if self.labor_wage_pressure_increases > 0 || self.labor_wage_pressure_decreases > 0 {
            sentences.push(format!(
                "Persistent labor pressure raised {} accepted wage bids and restrained {}, for a net observed adjustment of {} basis points across hires.",
                self.labor_wage_pressure_increases,
                self.labor_wage_pressure_decreases,
                self.labor_wage_adjustment_basis_points
            ));
        }
        if let Some(region) = self.leading_unemployment_region {
            let name = region_names
                .get(&region)
                .map_or_else(|| format!("region {}", region.get()), Clone::clone);
            sentences.push(format!(
                "The longest observed unemployment pressure reached {} consecutive months in {}.",
                self.maximum_unemployment_pressure_months, name
            ));
        }
        if let Some(region) = self.leading_vacancy_region {
            let name = region_names
                .get(&region)
                .map_or_else(|| format!("region {}", region.get()), Clone::clone);
            sentences.push(format!(
                "The longest observed unfilled-vacancy pressure reached {} consecutive months in {}.",
                self.maximum_vacancy_pressure_months, name
            ));
        }
    }

    fn push_reserve_priority_narration(
        &self,
        sentences: &mut Vec<String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        let Some((country, region, previous, new)) = self.leading_reserve_priority_change else {
            return;
        };
        let country_name = country_names
            .get(&country)
            .map_or_else(|| format!("country {}", country.get()), Clone::clone);
        let region_name = region_names
            .get(&region)
            .map_or_else(|| format!("region {}", region.get()), Clone::clone);
        sentences.push(format!(
            "Reserve priorities adapted {} times after persistent regional evidence: {} targets rose and {} fell; {} revisions followed sustained household import reliance. The leading revision in {}, {}, moved a regional-good priority from {} to {} basis points.",
            self.reserve_priority_changes,
            self.reserve_priority_increases,
            self.reserve_priority_decreases,
            self.reserve_import_reliance_priority_changes,
            country_name,
            region_name,
            previous,
            new
        ));
    }

    fn push_public_reserve_narration(
        &self,
        sentences: &mut Vec<String>,
        actor_names: &BTreeMap<ActorId, String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if let Some((country, previous_coverage, new_coverage, previous_budget, new_budget)) =
            self.leading_reserve_policy_change
        {
            let country_name = country_names
                .get(&country)
                .map_or_else(|| format!("country {}", country.get()), Clone::clone);
            sentences.push(format!(
                "Reserve doctrine adapted {} times from accumulated evidence: coverage rose {} times and fell {}, while procurement authority rose {} times and fell {}. In {}, the leading revision moved coverage from {} to {} months and the monthly treasury ceiling from {} to {} basis points.",
                self.reserve_policy_changes,
                self.reserve_coverage_increases,
                self.reserve_coverage_decreases,
                self.reserve_budget_increases,
                self.reserve_budget_decreases,
                country_name,
                previous_coverage,
                new_coverage,
                previous_budget,
                new_budget
            ));
        }
        self.push_reserve_priority_narration(sentences, country_names, region_names);
        if self.reserve_maintenance_entries > 0 {
            let unpaid = self
                .reserve_maintenance_assessed_minor
                .saturating_sub(self.reserve_maintenance_paid_minor);
            sentences.push(format!(
                "Maintaining carried public reserves consumed {} of {} assessed minor currency units; ordinary spoilage removed {} milli-units and unfunded upkeep destroyed another {}, leaving {} unpaid.",
                self.reserve_maintenance_paid_minor,
                self.reserve_maintenance_assessed_minor,
                self.reserve_baseline_spoilage_milli,
                self.reserve_neglect_spoilage_milli,
                unpaid
            ));
        }
        if self.reserve_requirements_reviewed > 0 {
            sentences.push(format!(
                "Officials reviewed {} observed survival-reserve requirements: coverage targets totaled {} milli-units against {} already stocked, with {} in eligible local supply. {} milli-units remained uncovered; {} reviews were supply-limited and {} budget-limited.",
                self.reserve_requirements_reviewed,
                self.reserve_target_milli,
                self.reserve_opening_stock_milli,
                self.reserve_available_supply_milli,
                self.reserve_remaining_gap_milli,
                self.reserve_supply_limited_reviews,
                self.reserve_budget_limited_reviews
            ));
            if self.reserve_prioritized_reviews > 0 {
                sentences.push(format!(
                    "Differentiated regional-good priorities scaled {} of those targets and placed the most protected needs first against the shared treasury ceiling.",
                    self.reserve_prioritized_reviews
                ));
            }
        }
        if self.reserve_procurements > 0 {
            let mut leading_actor = None;
            for (actor, spending) in &self.reserve_procurement_by_actor {
                if leading_actor.is_none_or(|(_, current)| *spending > current) {
                    leading_actor = Some((*actor, *spending));
                }
            }
            let buyer = leading_actor
                .and_then(|(actor, _)| actor_names.get(&actor))
                .map_or_else(|| "Public buyers".to_owned(), Clone::clone);
            sentences.push(format!(
                "After household markets exposed physical shortages, {} bought {} milli-units from local producers in {} procurements for {} minor currency units.",
                buyer,
                self.reserve_procured_milli,
                self.reserve_procurements,
                self.reserve_spending_minor
            ));
        }
        if self.reserve_distributions > 0 {
            let mut leading_region = None;
            for (region, quantity) in &self.reserve_distribution_by_region {
                if leading_region.is_none_or(|(_, current)| *quantity > current) {
                    leading_region = Some((*region, *quantity));
                }
            }
            let destination = leading_region
                .and_then(|(region, quantity)| {
                    region_names.get(&region).map(|name| {
                        format!("; the largest flow was {quantity} milli-units to {name}")
                    })
                })
                .unwrap_or_default();
            sentences.push(format!(
                "Public reserves released {} milli-units across {} cohort deliveries before grievance, stress, and health consequences{}.",
                self.reserve_distributed_milli, self.reserve_distributions, destination
            ));
        }
    }

    #[allow(clippy::too_many_lines)]
    fn push_firm_distress_narration(&self, sentences: &mut Vec<String>) {
        if self.lender_principal_repaid_minor > 0
            || self.lender_interest_income_minor > 0
            || self.lender_realized_loss_minor > 0
        {
            let successful_label = if self.lender_successful_loans == 1 {
                "loan"
            } else {
                "loans"
            };
            let default_label = if self.lender_defaulted_loans == 1 {
                "default"
            } else {
                "defaults"
            };
            sentences.push(format!(
                "Private lenders recovered {} principal and earned {} interest; {} {} completed successfully while {} {} realized {} of credit losses.",
                self.lender_principal_repaid_minor,
                self.lender_interest_income_minor,
                self.lender_successful_loans,
                successful_label,
                self.lender_defaulted_loans,
                default_label,
                self.lender_realized_loss_minor
            ));
        }
        if self.autonomous_credit_acceptances > 0 {
            sentences.push(format!(
                "Viable cash-constrained firms accepted {} observed credit offers providing {} minor currency units of working capital.",
                self.autonomous_credit_acceptances, self.autonomous_credit_principal_minor
            ));
        }
        if self.autonomous_credit_unfunded_searches > 0 {
            sentences.push(format!(
                "{} of {} viable monthly firm funding searches ended without an acceptable domestic credit offer.",
                self.autonomous_credit_unfunded_searches, self.autonomous_credit_searches
            ));
        }
        if self.firm_debt_service_attempts > 0 {
            let unpaid = self
                .firm_debt_service_due_minor
                .saturating_sub(self.firm_debt_service_paid_minor);
            sentences.push(format!(
                "Operating firms paid {} of {} minor currency units of scheduled debt service, including {} of {} interest; {} remained unpaid across {} overdue attempts.",
                self.firm_debt_service_paid_minor,
                self.firm_debt_service_due_minor,
                self.firm_interest_paid_minor,
                self.firm_interest_charged_minor,
                unpaid,
                self.firm_overdue_debt_attempts
            ));
        }
        if self.firm_recapitalizations > 0 {
            sentences.push(format!(
                "Owners injected {} minor currency units into {} firms after persistent wage arrears.",
                self.firm_recapitalization_minor, self.firm_recapitalizations
            ));
        }
        if self.firm_distress_downsizings > 0 {
            sentences.push(format!(
                "Cashless firms released {} workers across {} distress downsizings while preserving unpaid wage claims.",
                self.distress_workers_released, self.firm_distress_downsizings
            ));
        }
        if self.firm_insolvencies > 0 {
            let firm_word = if self.firm_insolvencies == 1 {
                "firm"
            } else {
                "firms"
            };
            sentences.push(format!(
                "{} {firm_word} entered insolvency administration with {} minor currency units of unpaid wages and {} milli-units of inventory preserved for resolution.",
                self.firm_insolvencies,
                self.insolvent_wage_claims_minor,
                self.insolvent_inventory_milli
            ));
        }
        if self.firm_liquidations > 0 {
            let firm_word = if self.firm_liquidations == 1 {
                "firm"
            } else {
                "firms"
            };
            let verb = if self.firm_liquidations == 1 {
                "was"
            } else {
                "were"
            };
            sentences.push(format!(
                "{} {firm_word} {verb} liquidated after a year without a viable plan: solvent producers bought {} milli-units of estate inventory for {} minor currency units, workers received {}, {} of worker claims were written off, ranked creditors recovered {}, {} of creditor claims were written off, {} milli-units of unsold inventory were written off, compatible successors acquired {} batches of installed capacity for {} minor currency units, {} batches of capacity were retired, and owners received {} residual cash.",
                self.firm_liquidations,
                self.liquidation_inventory_sold_milli,
                self.liquidation_inventory_sale_proceeds_minor,
                self.liquidation_claims_paid_minor,
                self.liquidation_claims_written_off_minor,
                self.liquidation_creditor_paid_minor,
                self.liquidation_creditor_written_off_minor,
                self.liquidation_inventory_written_off_milli,
                self.liquidation_capacity_sold_batches,
                self.liquidation_capacity_sale_proceeds_minor,
                self.liquidation_capacity_written_off_batches,
                self.liquidation_owner_distribution_minor
            ));
        }
        if self.firm_reorganizations > 0 {
            let firm_word = if self.firm_reorganizations == 1 {
                "firm"
            } else {
                "firms"
            };
            sentences.push(format!(
                "{} {firm_word} left insolvency after owners contributed {} minor currency units, paid {} in worker claims, and funded {} returning workers.",
                self.firm_reorganizations,
                self.reorganization_contributions_minor,
                self.reorganization_claims_paid_minor,
                self.reorganized_workers
            ));
        }
    }

    fn push_debt_restructuring_narration(&self, sentences: &mut Vec<String>) {
        if self.debt_restructuring_count == 0 {
            return;
        }
        let restructuring_word = if self.debt_restructuring_count == 1 {
            "restructuring"
        } else {
            "restructurings"
        };
        sentences.push(format!(
            "Sovereign debt crises forced {} {restructuring_word}, writing off {} minor currency units of principal at a political cost.",
            self.debt_restructuring_count, self.debt_principal_written_off_minor
        ));
    }

    fn push_transport_narration(&self, sentences: &mut Vec<String>) {
        if self.freight_payments > 0 {
            let leg_word = if self.freight_payments == 1 {
                "leg"
            } else {
                "legs"
            };
            sentences.push(format!(
                "Imported trade paid {} minor currency units to route carriers across {} freight {leg_word}.",
                self.freight_revenue_minor, self.freight_payments
            ));
        }
        if self.route_expansions > 0 {
            sentences.push(format!(
                "Carriers invested {} minor currency units to add {} milli-units of monthly capacity across {} routes.",
                self.route_investment_minor,
                self.route_capacity_added_milli,
                self.route_expansions
            ));
        }
    }

    fn push_conflict_narration(
        &self,
        sentences: &mut Vec<String>,
        country_names: &BTreeMap<CountryId, String>,
    ) {
        let name = |country: &CountryId| {
            country_names
                .get(country)
                .map_or_else(|| country.to_string(), Clone::clone)
        };
        for ((aggrieved, target), (first_level, last_level)) in &self.grievances_by_pair {
            let aggrieved_name = name(aggrieved);
            let target_name = name(target);
            if *last_level == 0 {
                sentences.push(format!(
                    "{aggrieved_name} resolved its material grievance against {target_name}."
                ));
            } else if last_level >= first_level {
                sentences.push(format!(
                    "{aggrieved_name}'s material grievance against {target_name} deepened to {last_level} basis points."
                ));
            } else {
                sentences.push(format!(
                    "{aggrieved_name}'s material grievance against {target_name} eased to {last_level} basis points."
                ));
            }
        }
        for (first, second, active) in &self.hostility_changes {
            let first_name = name(first);
            let second_name = name(second);
            let verb = if *active { "entered" } else { "ended" };
            sentences.push(format!(
                "{first_name} and {second_name} {verb} bilateral hostility."
            ));
        }
    }

    fn push_procurement_shortfall_narration(
        &self,
        sentences: &mut Vec<String>,
        firms: &BTreeMap<crate::FirmId, crate::Firm>,
        goods: &BTreeMap<crate::GoodId, crate::Good>,
    ) {
        for (&(buyer, good, cause), &quantity) in &self.procurement_shortfalls {
            let buyer_name = firms
                .get(&buyer)
                .map_or_else(|| buyer.to_string(), |firm| firm.name().to_owned());
            let good_name = goods.get(&good).map_or_else(
                || good.to_string(),
                |definition| definition.name().to_owned(),
            );
            let sentence = match cause {
                ProcurementShortfallCause::Other => {
                    format!("{buyer_name} could not procure {quantity} milli-units of {good_name}.")
                }
                ProcurementShortfallCause::RouteCapacity => format!(
                    "Route capacity prevented {buyer_name} from procuring {quantity} milli-units of {good_name}."
                ),
            };
            sentences.push(sentence);
        }
    }

    fn push_named_attributions(
        &self,
        sentences: &mut Vec<String>,
        actor_names: &BTreeMap<ActorId, String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if let Some((actor, funded)) = self
            .relief_by_actor
            .iter()
            .max_by_key(|(actor, funded)| (**funded, std::cmp::Reverse(**actor)))
        {
            if let Some(name) = actor_names.get(actor) {
                sentences.push(format!(
                    "{name} directed {funded} minor currency units of the emergency relief."
                ));
            }
        }
        if let Some((actor, changes)) = self
            .target_changes_by_actor
            .iter()
            .max_by_key(|(actor, changes)| (**changes, std::cmp::Reverse(**actor)))
        {
            if let Some(name) = actor_names.get(actor) {
                sentences.push(format!(
                    "{name} led firm management with {changes} production target adjustments."
                ));
            }
        }
        if let Some((region, actions)) = self
            .rationing_by_region
            .iter()
            .max_by_key(|(region, actions)| (**actions, std::cmp::Reverse(**region)))
        {
            if let Some(name) = region_names.get(region) {
                sentences.push(format!(
                    "Shortages pressed hardest on {name}, with {actions} rationing actions."
                ));
            }
        }
    }

    fn importance(&self) -> u16 {
        if self.excess_deaths > 0 {
            100
        } else if self.minimum_survival_bps.is_some_and(|value| value < 5_000) {
            90
        } else if self.debt_restructuring_count > 0 {
            88
        } else if !self.hostility_changes.is_empty() {
            85
        } else if self.firm_liquidations > 0 {
            82
        } else if self.rationing_actions > 0 {
            80
        } else if self.firm_insolvencies > 0 {
            78
        } else if self.firm_reorganizations > 0 {
            76
        } else if self.reserve_neglect_spoilage_milli > 0 {
            80
        } else if self.reserve_policy_changes > 0 || self.reserve_priority_changes > 0 {
            79
        } else if self.reserve_distributions > 0 || self.reserve_procurements > 0 {
            78
        } else if self.reserve_baseline_spoilage_milli > 0
            || self.reserve_maintenance_paid_minor > 0
        {
            76
        } else if !self.procurement_shortfalls.is_empty() {
            75
        } else if self.firms_founded > 0 {
            73
        } else if self.firm_overdue_debt_attempts > 0 {
            74
        } else if self.firm_distress_downsizings > 0 {
            72
        } else if self.relief_debt_minor > 0 || self.relief_minor > 0 {
            70
        } else if self.household_borrowing_minor > 0 {
            60
        } else if !self.grievances_by_pair.is_empty() {
            50
        } else if self.production_milli > 0 || self.traded_milli > 0 || self.measured_regions > 0 {
            40
        } else {
            10
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Actor, BasisPoints, CohortId, Country, CountryId, Good, GoodId, Money, Population,
        RecipeId, Region, SimDate, WorldSeed,
    };

    use super::*;

    #[test]
    fn chronicle_explains_persistent_regional_labor_pressure() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: RegionId::new(1),
                country: CountryId::new(1),
                name: "North March".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::EmploymentMatched {
                firm: crate::FirmId::new(1),
                cohort: CohortId::new(1),
                workers: 1,
                wage: Money::from_minor_units(125),
                minimum_education: crate::EducationLevel::Basic,
                labor_market_adjustment_basis_points: 500,
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalLaborMarketObserved {
                region: RegionId::new(1),
                available_workers: 0,
                vacancies: 3,
                offers: 2,
                hires: 1,
                average_offered_wage: Money::from_minor_units(120),
                unemployment_pressure_months: 0,
                vacancy_pressure_months: 4,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert!(entry.text.contains("2 funded offers and 1 hires"));
        assert!(entry.text.contains("3 residual vacancies"));
        assert!(
            entry
                .text
                .contains("Persistent labor pressure raised 1 accepted wage bids and restrained 0")
        );
        assert!(
            entry
                .text
                .contains("net observed adjustment of 500 basis points")
        );
        assert!(entry.text.contains(
            "longest observed unfilled-vacancy pressure reached 4 consecutive months in North March"
        ));
    }

    #[test]
    fn chronicle_names_shortage_driven_firm_entry_and_real_capital() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "North March",
                    Population::new(10),
                    Money::from_minor_units(100),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_good(Good::new(GoodId::new(1), "grain").expect("good"))
            .expect("good");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Iris Vale", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world.events.append(
            date,
            DomainEvent::FirmFoundedFromOpportunity {
                founder: ActorId::new(1),
                firm: crate::FirmId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                recipe: RecipeId::new(1),
                cohort: CohortId::new(1),
                wage: Money::from_minor_units(1),
                capital_cost: Money::from_minor_units(20),
                working_capital: Money::from_minor_units(3),
                pressure_months: 3,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 73);
        assert!(
            entry
                .text
                .contains("Persistent shortages drew 1 new local producers")
        );
        assert!(
            entry
                .text
                .contains("20 minor currency units to installed capacity")
        );
        assert!(
            entry
                .text
                .contains("Iris Vale founded firm 1 in North March to produce grain")
        );
    }

    #[test]
    fn chronicle_explains_evidence_driven_reserve_policy_change() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country");
        world.events.append(
            date,
            DomainEvent::GovernmentReservePolicyReviewed {
                country: CountryId::new(1),
                observed_shortage: crate::QuantityMilli::new(1_000),
                opening_stock: crate::QuantityMilli::new(0),
                remaining_gap: crate::QuantityMilli::new(0),
                baseline_spoilage: crate::QuantityMilli::new(0),
                neglect_spoilage: crate::QuantityMilli::new(0),
                preparedness_months: 0,
                budget_gap_months: 0,
                upkeep_stress_months: 0,
                waste_months: 0,
                previous_coverage_months: 1,
                new_coverage_months: 2,
                previous_monthly_budget: crate::BasisPoints::new(4_000).expect("budget"),
                new_monthly_budget: crate::BasisPoints::new(4_000).expect("budget"),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 79);
        assert!(entry.text.contains("Reserve doctrine adapted 1 times"));
        assert!(entry.text.contains("In Aster"));
        assert!(entry.text.contains("coverage from 1 to 2 months"));
        assert!(entry.text.contains("from 4000 to 4000 basis points"));
    }

    #[test]
    fn chronicle_explains_evidence_driven_reserve_priority_change() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country");
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: RegionId::new(1),
                country: CountryId::new(1),
                name: "North March".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::GovernmentReservePriorityReviewed {
                country: CountryId::new(1),
                region: RegionId::new(1),
                good: crate::GoodId::new(1),
                remaining_gap: crate::QuantityMilli::new(500),
                baseline_spoilage: crate::QuantityMilli::new(0),
                imported_quantity: crate::QuantityMilli::new(1_000),
                imported_share: BasisPoints::FULL,
                uncovered_months: 0,
                import_reliance_months: 0,
                idle_spoilage_months: 0,
                previous_priority: BasisPoints::new(5_000).expect("priority"),
                new_priority: BasisPoints::new(5_500).expect("priority"),
                revision_reason: crate::ReservePriorityRevisionReason::ImportReliance,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 79);
        assert!(entry.text.contains("Reserve priorities adapted 1 times"));
        assert!(
            entry
                .text
                .contains("1 revisions followed sustained household import reliance")
        );
        assert!(entry.text.contains("Aster, North March"));
        assert!(entry.text.contains("from 5000 to 5500 basis points"));
    }

    #[test]
    fn chronicle_explains_reserve_storage_cost_and_neglect() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::GovernmentReserveMaintained {
                country: CountryId::new(1),
                region: crate::RegionId::new(1),
                good: crate::GoodId::new(1),
                opening_stock: crate::QuantityMilli::new(1_000),
                reference_value: Money::from_minor_units(10),
                assessed_cost: Money::from_minor_units(4),
                paid_cost: Money::from_minor_units(1),
                baseline_spoilage: crate::QuantityMilli::new(100),
                neglect_spoilage: crate::QuantityMilli::new(169),
                closing_stock: crate::QuantityMilli::new(731),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 80);
        assert!(entry.text.contains("consumed 1 of 4 assessed"));
        assert!(entry.text.contains("ordinary spoilage removed 100"));
        assert!(entry.text.contains("unfunded upkeep destroyed another 169"));
        assert!(entry.text.contains("leaving 3 unpaid"));
    }

    #[test]
    fn chronicle_attributes_physical_reserve_procurement_and_release() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::ActorRegistered {
                actor: crate::ActorId::new(1),
                name: "Mara Voss".to_owned(),
                home_region: crate::RegionId::new(1),
            },
        );
        world.events.append(
            date,
            DomainEvent::GovernmentReserveRequirementReviewed {
                actor: crate::ActorId::new(1),
                country: CountryId::new(1),
                region: crate::RegionId::new(1),
                good: crate::GoodId::new(1),
                observed_shortage: crate::QuantityMilli::new(1_000),
                priority: BasisPoints::FULL,
                target_stock: crate::QuantityMilli::new(2_000),
                opening_stock: crate::QuantityMilli::new(0),
                available_supply: crate::QuantityMilli::new(2_000),
                budget_available: Money::from_minor_units(20),
                purchased: crate::QuantityMilli::new(2_000),
                spending: Money::from_minor_units(20),
                remaining_gap: crate::QuantityMilli::new(0),
                supply_limited: false,
                budget_limited: false,
            },
        );
        world.events.append(
            date,
            DomainEvent::GovernmentReserveProcured {
                actor: crate::ActorId::new(1),
                country: CountryId::new(1),
                region: crate::RegionId::new(1),
                seller: crate::FirmId::new(1),
                good: crate::GoodId::new(1),
                quantity: crate::QuantityMilli::new(2_000),
                cost: Money::from_minor_units(20),
            },
        );
        world.events.append(
            date,
            DomainEvent::GovernmentReserveDistributed {
                country: CountryId::new(1),
                region: crate::RegionId::new(1),
                cohort: CohortId::new(1),
                good: crate::GoodId::new(1),
                quantity: crate::QuantityMilli::new(1_000),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 78);
        assert!(
            entry
                .text
                .contains("reviewed 1 observed survival-reserve requirements")
        );
        assert!(
            entry
                .text
                .contains("coverage targets totaled 2000 milli-units")
        );
        assert!(entry.text.contains("Mara Voss bought 2000 milli-units"));
        assert!(entry.text.contains("for 20 minor currency units"));
        assert!(entry.text.contains("released 1000 milli-units"));
        assert!(
            entry
                .text
                .contains("before grievance, stress, and health consequences")
        );
    }

    #[test]
    fn chronicle_reports_scheduled_firm_debt_service_and_overdue_principal() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::FirmDebtServiceSettled {
                firm: crate::FirmId::new(1),
                creditor: crate::ActorId::new(1),
                priority: crate::FirmCreditorPriority::Secured,
                due: Money::from_minor_units(40),
                paid: Money::from_minor_units(10),
                interest_charged: Money::from_minor_units(2),
                interest_paid: Money::from_minor_units(2),
                remaining_principal: Money::from_minor_units(70),
                remaining_interest: Money::default(),
                overdue: true,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let entry = world.chronicle().pop().expect("chronicle entry");
        assert_eq!(entry.importance, 74);
        assert!(entry.text.contains("paid 10 of 40"));
        assert!(entry.text.contains("including 2 of 2 interest"));
        assert!(
            entry
                .text
                .contains("30 remained unpaid across 1 overdue attempts")
        );
    }

    #[test]
    fn chronicle_connects_material_shortage_coping_relief_and_deaths() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::HouseholdSurvivalBorrowed {
                cohort: CohortId::new(1),
                amount: Money::from_minor_units(100),
                ending_wealth: Money::from_minor_units(100),
                ending_debt: Money::from_minor_units(100),
            },
        );
        world.events.append(
            date,
            DomainEvent::EmergencyReliefFunded {
                actor: crate::ActorId::new(1),
                country: CountryId::new(1),
                cohort: CohortId::new(1),
                amount: Money::from_minor_units(50),
            },
        );
        world.events.append(
            date,
            DomainEvent::CohortHealthUpdated {
                cohort: CohortId::new(1),
                survival_fulfillment: BasisPoints::new(2_500).expect("basis points"),
                functional_capacity: BasisPoints::new(9_000).expect("basis points"),
                excess_deaths: 3,
                survivors: Population::new(97),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert_eq!(chronicle[0].year, 2025);
        assert_eq!(chronicle[0].importance, 100);
        assert!(chronicle[0].text.contains("25.00%"));
        assert!(chronicle[0].text.contains("3 excess deaths"));
        assert!(chronicle[0].text.contains("borrowed 100"));
        assert!(chronicle[0].text.contains("transferred 50"));
    }

    #[test]
    fn chronicle_reports_public_debt_service() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::PublicDebtInterestCharged {
                country: CountryId::new(1),
                opening_debt: Money::from_minor_units(1_000_000),
                interest: Money::from_minor_units(30_000),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(chronicle[0].text.contains("interest"));
        assert!(chronicle[0].text.contains("30000"));
        assert!(chronicle[0].text.contains("1 indebted"));
    }

    #[test]
    fn chronicle_reports_fiscal_closure_totals() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(9), date);
        world.events.append(
            date,
            DomainEvent::CountryFiscalYearClosed {
                country: CountryId::new(1),
                revenue: Money::from_minor_units(500),
                spending: Money::from_minor_units(700),
                treasury: Money::from_minor_units(0),
                debt: Money::from_minor_units(200),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(chronicle[0].text.contains("collected 500"));
        assert!(chronicle[0].text.contains("spent 700"));
        assert!(chronicle[0].text.contains("200 of public debt"));
        assert!(chronicle[0].text.contains("across 1 countries"));
    }

    #[test]
    fn chronicle_records_bilateral_hostility_by_country_name() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        for (country, name) in [(1, "Arcadia"), (2, "Borealia")] {
            world.events.append(
                date,
                DomainEvent::CountryRegistered {
                    country: CountryId::new(country),
                    name: name.to_owned(),
                },
            );
        }
        world.events.append(
            date,
            DomainEvent::BilateralHostilityChanged {
                first: CountryId::new(1),
                second: CountryId::new(2),
                active: true,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(
            chronicle[0]
                .text
                .contains("Arcadia and Borealia entered bilateral hostility")
        );
    }

    #[test]
    fn chronicle_narrates_the_grievance_arc_by_country_name() {
        let escalation = SimDate::new(2025, 1).expect("date");
        let peace = SimDate::new(2026, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), escalation);
        for (country, name) in [(1, "Arcadia"), (2, "Borealia")] {
            world.events.append(
                escalation,
                DomainEvent::CountryRegistered {
                    country: CountryId::new(country),
                    name: name.to_owned(),
                },
            );
        }
        world.events.append(
            escalation,
            DomainEvent::BilateralGrievanceChanged {
                aggrieved: CountryId::new(2),
                target: CountryId::new(1),
                level: BasisPoints::new(7_500).expect("basis points"),
            },
        );
        world.events.append(
            escalation,
            DomainEvent::BilateralHostilityChanged {
                first: CountryId::new(1),
                second: CountryId::new(2),
                active: true,
            },
        );
        world.events.append(
            escalation,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );
        world.events.append(
            peace,
            DomainEvent::BilateralGrievanceChanged {
                aggrieved: CountryId::new(2),
                target: CountryId::new(1),
                level: BasisPoints::new(0).expect("basis points"),
            },
        );
        world.events.append(
            peace,
            DomainEvent::BilateralHostilityChanged {
                first: CountryId::new(1),
                second: CountryId::new(2),
                active: false,
            },
        );
        world.events.append(
            peace,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2026,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 2);
        assert!(chronicle[0].text.contains(
            "Borealia's material grievance against Arcadia deepened to 7500 basis points"
        ));
        assert!(
            chronicle[0]
                .text
                .contains("Arcadia and Borealia entered bilateral hostility")
        );
        assert_eq!(chronicle[0].importance, 85);
        assert!(
            chronicle[1]
                .text
                .contains("Borealia resolved its material grievance against Arcadia")
        );
        assert!(
            chronicle[1]
                .text
                .contains("Arcadia and Borealia ended bilateral hostility")
        );
        assert_eq!(chronicle[1].importance, 85);
    }

    #[test]
    fn chronicle_names_the_most_active_elite_actors_and_regions() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::ActorRegistered {
                actor: crate::ActorId::new(1),
                home_region: crate::RegionId::new(1),
                name: "Mara Voss".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: crate::RegionId::new(1),
                country: CountryId::new(1),
                name: "Northreach".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::EmergencyReliefFunded {
                actor: crate::ActorId::new(1),
                country: CountryId::new(1),
                cohort: CohortId::new(1),
                amount: Money::from_minor_units(50),
            },
        );
        world.events.append(
            date,
            DomainEvent::FirmProductionTargetSet {
                actor: crate::ActorId::new(1),
                firm: crate::FirmId::new(1),
                previous_batches: Some(5),
                target_batches: 2,
            },
        );
        world.events.append(
            date,
            DomainEvent::SurvivalRationingApplied {
                country: CountryId::new(1),
                region: crate::RegionId::new(1),
                good: crate::GoodId::new(1),
                requested: crate::QuantityMilli::new(1_000),
                available: crate::QuantityMilli::new(400),
                cohorts: 1,
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        let text = &chronicle[0].text;
        assert!(text.contains("Mara Voss directed 50 minor currency units"));
        assert!(
            text.contains("Mara Voss led firm management with 1 production target adjustments")
        );
        assert!(text.contains("Shortages pressed hardest on Northreach, with 1 rationing actions"));
    }
    #[test]
    fn chronicle_reports_route_capacity_investment() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::RouteCapacityExpanded {
                route: crate::RouteId::new(1),
                carrier: crate::FirmId::new(1),
                previous_capacity: crate::QuantityMilli::new(1_000),
                added_capacity: crate::QuantityMilli::new(100),
                cost: Money::from_minor_units(12),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(
            chronicle[0]
                .text
                .contains("invested 12 minor currency units")
        );
        assert!(chronicle[0].text.contains("add 100 milli-units"));
        assert!(chronicle[0].text.contains("across 1 routes"));
    }

    #[test]
    fn chronicle_reports_import_freight_revenue() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::FirmProcurementFreightPaid {
                buyer: crate::FirmId::new(1),
                seller: crate::FirmId::new(2),
                carrier: crate::FirmId::new(3),
                route: crate::RouteId::new(1),
                amount: Money::from_minor_units(7),
            },
        );
        world.events.append(
            date,
            DomainEvent::MarketFreightPaid {
                buyer: CohortId::new(1),
                seller: crate::FirmId::new(2),
                carrier: crate::FirmId::new(3),
                route: crate::RouteId::new(1),
                amount: Money::from_minor_units(3),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert!(chronicle[0].text.contains("paid 10 minor currency units"));
        assert!(chronicle[0].text.contains("across 2 freight legs"));
    }

    #[test]
    fn chronicle_reports_sovereign_debt_restructuring() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::PublicDebtRestructured {
                country: CountryId::new(1),
                debt_before: Money::from_minor_units(1_000),
                debt_after: Money::from_minor_units(600),
                principal_written_off: Money::from_minor_units(400),
            },
        );
        world.events.append(
            date,
            DomainEvent::EconomicYearCompleted {
                closed_year: 2025,
                monthly_cycles: 12,
            },
        );

        let chronicle = world.chronicle();
        assert_eq!(chronicle.len(), 1);
        assert_eq!(chronicle[0].importance, 88);
        assert!(chronicle[0].text.contains("1 restructuring"));
        assert!(chronicle[0].text.contains("writing off 400"));
        assert!(chronicle[0].text.contains("political cost"));
    }
}
