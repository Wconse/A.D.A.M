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
        let mut program_names = BTreeMap::<crate::ProgramId, String>::new();
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
                DomainEvent::GovernmentProgramDeclared { program, name, .. } => {
                    program_names.insert(*program, name.clone());
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
                    &program_names,
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
    program_events: u64,
    programs_declared: u64,
    programs_cancelled: u64,
    program_appropriated_minor: i128,
    program_delivered_minor: i128,
    program_carryover_minor: i128,
    program_beneficiary_outcomes: u64,
    program_underfulfilled_outcomes: u64,
    program_excluded_outcomes: u64,
    program_material_consumed_milli: u128,
    program_temporary_workers: u64,
    program_wages_minor: i128,
    leading_program_execution: Option<(crate::ProgramId, i64, i64, u16)>,
    leading_program_loser: Option<(
        crate::ProgramId,
        RegionId,
        u16,
        crate::ProgramRegionalOutcomeKind,
    )>,
    leading_program_consequence: Option<(CountryId, i32, i32, i32, i32)>,
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
    labor_switches: u64,
    labor_switch_wage_gain_minor: i128,
    skill_labor_observations: u64,
    maximum_skill_vacancy_pressure_months: u8,
    leading_skill_shortage: Option<(RegionId, crate::EducationLevel)>,
    workforce_training_started: u64,
    workforce_training_completed: u64,
    workforce_training_tuition_minor: i128,
    labor_wage_adjustment_basis_points: i64,
    maximum_unemployment_pressure_months: u8,
    maximum_vacancy_pressure_months: u8,
    leading_unemployment_region: Option<RegionId>,
    leading_vacancy_region: Option<RegionId>,
    migrated_households: u64,
    migrated_people: u64,
    migration_fees_minor: i128,
    leading_migration: Option<(RegionId, RegionId, u64, u16)>,
    housing_projects_started: u64,
    housing_dwellings_started: u64,
    housing_committed_minor: i128,
    housing_projects_completed: u64,
    housing_dwellings_completed: u64,
    housing_capacity_after_completion: u64,
    leading_housing_region: Option<RegionId>,
    regional_social_pressure_observations: u64,
    leading_social_pressure: Option<(RegionId, u16, u16, u16, u16)>,
    country_legitimacy_pressure_observations: u64,
    leading_legitimacy_pressure: Option<(CountryId, u16, i32)>,
    leading_regional_interest:
        Option<(RegionId, crate::RegionalPolicyPriority, u16, u16, u8, bool)>,
    regional_net_contributors: u64,
    regional_net_beneficiaries: u64,
    regional_net_transfers_minor: i128,
    leading_policy_outcome: Option<(RegionId, i64)>,
    leading_regional_confidence: Option<(CountryId, u16, i32)>,
    explicit_service_allocations: u64,
    autonomous_service_allocations: u64,
    leading_service_allocation: Option<(RegionId, crate::ServiceAllocationSource, u16, i64)>,
    leading_service_allocation_influence: Option<(
        ActorId,
        RegionId,
        crate::ServiceAllocationInfluenceKind,
        u16,
        u16,
    )>,
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
    unprofitable_agreement_endings: u64,
    unprofitable_workers_released: u64,
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
        if self.observe_government_program_event(event)
            || self.observe_procurement_event(event)
            || self.observe_public_reserve_event(event)
            || self.observe_freight_event(event)
            || self.observe_route_expansion_event(event)
            || self.observe_firm_entry_event(event)
            || self.observe_labor_market_event(event)
            || self.observe_settlement_event(event)
            || self.observe_social_pressure_event(event)
            || self.observe_regional_interest_event(event)
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

    fn observe_service_allocation_event(&mut self, event: &DomainEvent) -> bool {
        if let DomainEvent::RegionalServiceAllocationInfluenceApplied {
            actor,
            region,
            kind,
            weight,
            score_bonus,
            ..
        } = event
        {
            let candidate = (*actor, *region, *kind, weight.get(), *score_bonus);
            if self
                .leading_service_allocation_influence
                .is_none_or(|current| (candidate.4, candidate.3) > (current.4, current.3))
            {
                self.leading_service_allocation_influence = Some(candidate);
            }
            return true;
        }
        let DomainEvent::RegionalServiceBudgetAllocated {
            region,
            source,
            share,
            service_budget,
            ..
        } = event
        else {
            return false;
        };
        match source {
            crate::ServiceAllocationSource::AutonomousPrudent => {
                self.autonomous_service_allocations =
                    self.autonomous_service_allocations.saturating_add(1);
            }
            crate::ServiceAllocationSource::ExplicitPoliticalDecision => {
                self.explicit_service_allocations =
                    self.explicit_service_allocations.saturating_add(1);
            }
        }
        let candidate = (*region, *source, share.get(), service_budget.minor_units());
        let rank = |allocation: (RegionId, crate::ServiceAllocationSource, u16, i64)| {
            let explicit =
                u8::from(allocation.1 == crate::ServiceAllocationSource::ExplicitPoliticalDecision);
            let funded = u8::from(allocation.3 > 0);
            if explicit == 1 {
                (explicit, funded, i64::from(allocation.2), allocation.3)
            } else {
                (explicit, funded, allocation.3, i64::from(allocation.2))
            }
        };
        if self
            .leading_service_allocation
            .is_none_or(|current| rank(candidate) > rank(current))
        {
            self.leading_service_allocation = Some(candidate);
        }
        true
    }

    fn observe_regional_interest_event(&mut self, event: &DomainEvent) -> bool {
        if self.observe_service_allocation_event(event) {
            return true;
        }
        match event {
            DomainEvent::RegionalPolicyOutcomeRecorded {
                region,
                net_transfer,
                fiscal_position,
                ..
            } => {
                match fiscal_position {
                    crate::RegionalFiscalPosition::NetContributor => {
                        self.regional_net_contributors =
                            self.regional_net_contributors.saturating_add(1);
                    }
                    crate::RegionalFiscalPosition::Balanced => {}
                    crate::RegionalFiscalPosition::NetBeneficiary => {
                        self.regional_net_beneficiaries =
                            self.regional_net_beneficiaries.saturating_add(1);
                    }
                }
                self.regional_net_transfers_minor += i128::from(net_transfer.minor_units());
                let candidate = (*region, net_transfer.minor_units());
                if self
                    .leading_policy_outcome
                    .is_none_or(|current| candidate.1.unsigned_abs() > current.1.unsigned_abs())
                {
                    self.leading_policy_outcome = Some(candidate);
                }
            }
            DomainEvent::RegionalInterestUpdated {
                region,
                previous_priority,
                priority,
                priority_pressure,
                satisfaction,
                years_persistent,
            } => {
                let candidate = (
                    *region,
                    *priority,
                    priority_pressure.get(),
                    satisfaction.get(),
                    *years_persistent,
                    previous_priority != priority,
                );
                if self.leading_regional_interest.is_none_or(|current| {
                    candidate.3 < current.3 || (candidate.3 == current.3 && candidate.2 > current.2)
                }) {
                    self.leading_regional_interest = Some(candidate);
                }
            }
            DomainEvent::CountryRegionalConfidenceApplied {
                country,
                population_weighted_confidence,
                legitimacy_effect,
            } => {
                let candidate = (
                    *country,
                    population_weighted_confidence.get(),
                    *legitimacy_effect,
                );
                if self
                    .leading_regional_confidence
                    .is_none_or(|current| candidate.1.abs_diff(5_000) > current.1.abs_diff(5_000))
                {
                    self.leading_regional_confidence = Some(candidate);
                }
            }
            _ => return false,
        }
        true
    }

    fn observe_social_pressure_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::RegionalSocialPressureUpdated {
                region,
                chronic_unemployment,
                livelihood_stress,
                public_service_shortfall,
                combined,
            } => {
                self.regional_social_pressure_observations =
                    self.regional_social_pressure_observations.saturating_add(1);
                let candidate = (
                    *region,
                    chronic_unemployment.get(),
                    livelihood_stress.get(),
                    public_service_shortfall.get(),
                    combined.get(),
                );
                if self
                    .leading_social_pressure
                    .is_none_or(|current| candidate.4 > current.4)
                {
                    self.leading_social_pressure = Some(candidate);
                }
            }
            DomainEvent::CountryLegitimacyPressureApplied {
                country,
                population_weighted_pressure,
                legitimacy_effect,
            } => {
                self.country_legitimacy_pressure_observations = self
                    .country_legitimacy_pressure_observations
                    .saturating_add(1);
                let candidate = (
                    *country,
                    population_weighted_pressure.get(),
                    *legitimacy_effect,
                );
                if self.leading_legitimacy_pressure.is_none_or(|current| {
                    candidate.1 > current.1 || (candidate.1 == current.1 && candidate.0 < current.0)
                }) {
                    self.leading_legitimacy_pressure = Some(candidate);
                }
            }
            _ => return false,
        }
        true
    }

    fn observe_settlement_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::HouseholdMigrated {
                from_region,
                to_region,
                people,
                households,
                relocation_fee,
                destination_housing_pressure_basis_points,
                ..
            } => {
                self.migrated_households = self.migrated_households.saturating_add(*households);
                self.migrated_people = self.migrated_people.saturating_add(people.people());
                self.migration_fees_minor += i128::from(relocation_fee.minor_units());
                self.leading_migration.get_or_insert((
                    *from_region,
                    *to_region,
                    people.people(),
                    *destination_housing_pressure_basis_points,
                ));
            }
            DomainEvent::RegionalHousingConstructionStarted {
                region,
                dwellings,
                committed_cost,
                ..
            } => {
                self.housing_projects_started = self.housing_projects_started.saturating_add(1);
                self.housing_dwellings_started =
                    self.housing_dwellings_started.saturating_add(*dwellings);
                self.housing_committed_minor += i128::from(committed_cost.minor_units());
                self.leading_housing_region.get_or_insert(*region);
            }
            DomainEvent::RegionalHousingConstructionCompleted {
                region,
                dwellings,
                dwelling_capacity,
                ..
            } => {
                self.housing_projects_completed = self.housing_projects_completed.saturating_add(1);
                self.housing_dwellings_completed =
                    self.housing_dwellings_completed.saturating_add(*dwellings);
                self.housing_capacity_after_completion = self
                    .housing_capacity_after_completion
                    .saturating_add(*dwelling_capacity);
                self.leading_housing_region.get_or_insert(*region);
            }
            _ => return false,
        }
        true
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
            DomainEvent::EmploymentEndedAsUnprofitable {
                workers_released, ..
            } => {
                self.unprofitable_agreement_endings =
                    self.unprofitable_agreement_endings.saturating_add(1);
                self.unprofitable_workers_released = self
                    .unprofitable_workers_released
                    .saturating_add(*workers_released);
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
            DomainEvent::EmploymentSwitched {
                previous_wage,
                offered_wage,
                ..
            } => {
                self.labor_switches = self.labor_switches.saturating_add(1);
                self.labor_switch_wage_gain_minor += i128::from(
                    offered_wage
                        .minor_units()
                        .saturating_sub(previous_wage.minor_units()),
                );
            }
            DomainEvent::RegionalSkillLaborMarketObserved {
                region,
                minimum_education,
                vacancy_pressure_months,
                ..
            } => {
                self.skill_labor_observations = self.skill_labor_observations.saturating_add(1);
                if *vacancy_pressure_months > self.maximum_skill_vacancy_pressure_months {
                    self.maximum_skill_vacancy_pressure_months = *vacancy_pressure_months;
                    self.leading_skill_shortage = Some((*region, *minimum_education));
                }
            }
            DomainEvent::WorkforceTrainingStarted { tuition_paid, .. } => {
                self.workforce_training_started = self.workforce_training_started.saturating_add(1);
                self.workforce_training_tuition_minor += i128::from(tuition_paid.minor_units());
            }
            DomainEvent::WorkforceTrainingCompleted { .. } => {
                self.workforce_training_completed =
                    self.workforce_training_completed.saturating_add(1);
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

    #[allow(clippy::too_many_lines)]
    fn observe_government_program_event(&mut self, event: &DomainEvent) -> bool {
        match event {
            DomainEvent::GovernmentProgramDeclared { .. } => {
                self.program_events += 1;
                self.programs_declared += 1;
            }
            DomainEvent::GovernmentProgramAppropriated { appropriated, .. } => {
                self.program_events += 1;
                self.program_appropriated_minor += i128::from(appropriated.minor_units());
            }
            DomainEvent::GovernmentProgramCancelled { .. } => {
                self.program_events += 1;
                self.programs_cancelled += 1;
            }
            DomainEvent::GovernmentProgramExecuted {
                program,
                delivered,
                remaining_carryover,
                years_delayed,
                ..
            } => {
                self.program_events += 1;
                self.program_delivered_minor += i128::from(delivered.minor_units());
                self.program_carryover_minor += i128::from(remaining_carryover.minor_units());
                let candidate = (
                    *program,
                    delivered.minor_units(),
                    remaining_carryover.minor_units(),
                    *years_delayed,
                );
                if self
                    .leading_program_execution
                    .is_none_or(|current| candidate.2 > current.2)
                {
                    self.leading_program_execution = Some(candidate);
                }
            }
            DomainEvent::GovernmentProgramRegionalOutcomeRecorded {
                program,
                region,
                fulfillment,
                outcome,
                ..
            } => {
                self.program_events += 1;
                match outcome {
                    crate::ProgramRegionalOutcomeKind::Beneficiary => {
                        self.program_beneficiary_outcomes += 1;
                    }
                    crate::ProgramRegionalOutcomeKind::Underfulfilled => {
                        self.program_underfulfilled_outcomes += 1;
                    }
                    crate::ProgramRegionalOutcomeKind::Excluded => {
                        self.program_excluded_outcomes += 1;
                    }
                }
                let candidate = (*program, *region, fulfillment.get(), *outcome);
                if self
                    .leading_program_loser
                    .is_none_or(|current| candidate.2 < current.2)
                {
                    self.leading_program_loser = Some(candidate);
                }
            }
            DomainEvent::GovernmentProgramMaterialConsumed { quantity, .. } => {
                self.program_events += 1;
                self.program_material_consumed_milli += u128::from(quantity.get());
            }
            DomainEvent::GovernmentProgramTemporaryLaborEmployed { workers, wages, .. } => {
                self.program_events += 1;
                self.program_temporary_workers =
                    self.program_temporary_workers.saturating_add(*workers);
                self.program_wages_minor += i128::from(wages.minor_units());
            }
            DomainEvent::GovernmentProgramPoliticalConsequencesApplied {
                country,
                regional_average_shift,
                polarization,
                legitimacy_shift,
                elite_cohesion_shift,
                ..
            } => {
                self.program_events += 1;
                let candidate = (
                    *country,
                    *regional_average_shift,
                    *polarization,
                    *legitimacy_shift,
                    *elite_cohesion_shift,
                );
                if self
                    .leading_program_consequence
                    .is_none_or(|current| candidate.2 > current.2)
                {
                    self.leading_program_consequence = Some(candidate);
                }
            }
            DomainEvent::GovernmentProgramPoliticalInfluenceApplied { .. }
            | DomainEvent::GovernmentProgramRegionalDelivery { .. } => {
                self.program_events += 1;
            }
            _ => return false,
        }
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

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn finish(
        self,
        year: i32,
        actor_names: &BTreeMap<ActorId, String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
        program_names: &BTreeMap<crate::ProgramId, String>,
        firms: &BTreeMap<crate::FirmId, crate::Firm>,
        goods: &BTreeMap<crate::GoodId, crate::Good>,
    ) -> Option<ChronicleEntry> {
        if self.months == 0 && !self.completed && self.program_events == 0 {
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
        self.push_program_narration(&mut sentences, country_names, region_names, program_names);
        self.push_conflict_narration(&mut sentences, country_names);
        self.push_procurement_shortfall_narration(&mut sentences, firms, goods);
        self.push_firm_entry_narration(&mut sentences, actor_names, region_names, firms, goods);
        self.push_labor_market_narration(&mut sentences, region_names);
        self.push_social_pressure_narration(&mut sentences, country_names, region_names);
        self.push_regional_interest_narration(
            &mut sentences,
            country_names,
            actor_names,
            region_names,
        );
        self.push_settlement_narration(&mut sentences, region_names);
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

    fn push_regional_interest_narration(
        &self,
        sentences: &mut Vec<String>,
        country_names: &BTreeMap<CountryId, String>,
        actor_names: &BTreeMap<ActorId, String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if let Some((region, priority, pressure, satisfaction, years, changed)) =
            self.leading_regional_interest
        {
            let region_name = region_names
                .get(&region)
                .map_or_else(|| region.to_string(), Clone::clone);
            let priority_name = match priority {
                crate::RegionalPolicyPriority::Employment => "employment",
                crate::RegionalPolicyPriority::HouseholdSecurity => "household security",
                crate::RegionalPolicyPriority::PublicServices => "public services",
                crate::RegionalPolicyPriority::Stability => "stability",
            };
            let transition = if changed { "adopted" } else { "retained" };
            sentences.push(format!(
                "{region_name} {transition} {priority_name} as its leading policy interest at {pressure} basis points; regional satisfaction stood at {satisfaction} after {years} persistent years."
            ));
        }
        if self.regional_net_contributors > 0 || self.regional_net_beneficiaries > 0 {
            let leading = self.leading_policy_outcome.map_or_else(
                || "no leading region".to_owned(),
                |(region, net)| {
                    let name = region_names
                        .get(&region)
                        .map_or_else(|| region.to_string(), Clone::clone);
                    format!("{name} recorded the largest net position at {net}")
                },
            );
            sentences.push(format!(
                "Annual service allocation left {} regions as net fiscal beneficiaries and {} as net contributors; aggregate regional net transfers were {} minor currency units, and {leading}.",
                self.regional_net_beneficiaries,
                self.regional_net_contributors,
                self.regional_net_transfers_minor
            ));
        }
        if let Some((actor, region, kind, weight, score_bonus)) =
            self.leading_service_allocation_influence
        {
            let actor_name = actor_names
                .get(&actor)
                .map_or_else(|| actor.to_string(), Clone::clone);
            let region_name = region_names
                .get(&region)
                .map_or_else(|| region.to_string(), Clone::clone);
            let mechanism = match kind {
                crate::ServiceAllocationInfluenceKind::OfficeHolder => {
                    "control of the political office"
                }
                crate::ServiceAllocationInfluenceKind::InfluenceEdge => {
                    "influence over the political office"
                }
            };
            sentences.push(format!(
                "{actor_name}'s {mechanism}, measured at {weight} basis points, added {score_bonus} decision points toward {region_name} in the autonomous service allocation."
            ));
        }
        if let Some((region, source, share, budget)) = self.leading_service_allocation {
            let region_name = region_names
                .get(&region)
                .map_or_else(|| region.to_string(), Clone::clone);
            let source_text = match source {
                crate::ServiceAllocationSource::AutonomousPrudent => {
                    "The autonomous government allocation"
                }
                crate::ServiceAllocationSource::ExplicitPoliticalDecision => {
                    "An explicit political decision"
                }
            };
            sentences.push(format!(
                "{source_text} directed {}.{:02}% of the public-service budget, or {budget} minor currency units, to {region_name}.",
                share / 100,
                share % 100
            ));
        }
        if let Some((country, confidence, effect)) = self.leading_regional_confidence {
            let country_name = country_names
                .get(&country)
                .map_or_else(|| country.to_string(), Clone::clone);
            let direction = if effect < 0 { "reduced" } else { "supported" };
            sentences.push(format!(
                "Population-weighted regional confidence of {confidence} basis points {direction} legitimacy in {country_name} by {} basis points.",
                effect.unsigned_abs()
            ));
        }
    }

    fn push_social_pressure_narration(
        &self,
        sentences: &mut Vec<String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if let Some((region, unemployment, livelihood, services, combined)) =
            self.leading_social_pressure
        {
            let name = region_names
                .get(&region)
                .map_or_else(|| region.to_string(), Clone::clone);
            sentences.push(format!(
                "Material social pressure peaked at {combined} basis points in {name}: chronic unemployment contributed {unemployment}, livelihood stress {livelihood}, and public-service shortfall {services}."
            ));
        }
        if let Some((country, pressure, effect)) = self.leading_legitimacy_pressure {
            let name = country_names
                .get(&country)
                .map_or_else(|| country.to_string(), Clone::clone);
            let direction = if effect < 0 { "reduced" } else { "supported" };
            sentences.push(format!(
                "Population-weighted pressure of {pressure} basis points {direction} legitimacy in {name} by {} basis points.",
                effect.unsigned_abs()
            ));
        }
    }

    fn push_settlement_narration(
        &self,
        sentences: &mut Vec<String>,
        region_names: &BTreeMap<RegionId, String>,
    ) {
        if self.migrated_households > 0 {
            let route = self.leading_migration.map_or_else(
                || "between regions".to_owned(),
                |(from, to, _, pressure)| {
                    let from_name = region_names
                        .get(&from)
                        .map_or_else(|| from.to_string(), Clone::clone);
                    let to_name = region_names
                        .get(&to)
                        .map_or_else(|| to.to_string(), Clone::clone);
                    format!(
                        "from {from_name} to {to_name}, where projected housing pressure reached {pressure} basis points"
                    )
                },
            );
            sentences.push(format!(
                "{} households containing {} people moved {route} and paid {} minor currency units in relocation fees.",
                self.migrated_households,
                self.migrated_people,
                self.migration_fees_minor
            ));
        }
        if self.housing_projects_started > 0 {
            let place = self.leading_housing_region.map_or_else(
                || "pressured regions".to_owned(),
                |region| {
                    region_names
                        .get(&region)
                        .map_or_else(|| region.to_string(), Clone::clone)
                },
            );
            sentences.push(format!(
                "Public authorities committed {} minor currency units to start {} housing projects for {} dwellings, led by {place}.",
                self.housing_committed_minor,
                self.housing_projects_started,
                self.housing_dwellings_started
            ));
        }
        if self.housing_projects_completed > 0 {
            sentences.push(format!(
                "{} public housing projects completed {} dwellings; their regions reported {} combined dwellings of capacity after completion.",
                self.housing_projects_completed,
                self.housing_dwellings_completed,
                self.housing_capacity_after_completion
            ));
        }
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
        if self.labor_switches > 0 {
            sentences.push(format!(
                "Workers voluntarily changed employers {} times for materially better offers, gaining {} minor currency units of monthly wage across those moves.",
                self.labor_switches, self.labor_switch_wage_gain_minor
            ));
        }
        if let Some((region, education)) = self.leading_skill_shortage {
            let name = region_names
                .get(&region)
                .map_or_else(|| format!("region {}", region.get()), Clone::clone);
            sentences.push(format!(
                "Qualification-specific markets recorded {} observations; the longest {:?}-qualified shortage reached {} months in {}.",
                self.skill_labor_observations,
                education,
                self.maximum_skill_vacancy_pressure_months,
                name
            ));
        }
        if self.workforce_training_started > 0 || self.workforce_training_completed > 0 {
            sentences.push(format!(
                "Households started {} workforce training programs and completed {}, paying {} minor currency units of tuition.",
                self.workforce_training_started,
                self.workforce_training_completed,
                self.workforce_training_tuition_minor
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

    fn push_program_narration(
        &self,
        sentences: &mut Vec<String>,
        country_names: &BTreeMap<CountryId, String>,
        region_names: &BTreeMap<RegionId, String>,
        program_names: &BTreeMap<crate::ProgramId, String>,
    ) {
        if self.programs_declared > 0 || self.program_appropriated_minor > 0 {
            sentences.push(format!(
                "Governments declared {} programs and appropriated {} minor currency units; {} programs were cancelled.",
                self.programs_declared, self.program_appropriated_minor, self.programs_cancelled
            ));
        }
        if let Some((program, delivered, carryover, delayed)) = self.leading_program_execution {
            let name = program_names
                .get(&program)
                .map_or("an established program", String::as_str);
            sentences.push(format!(
                "The program '{name}' delivered {delivered} minor currency units, left {carryover} in carryover, and had accumulated {delayed} delayed years."
            ));
        }
        if self.program_material_consumed_milli > 0 || self.program_temporary_workers > 0 {
            sentences.push(format!(
                "Program execution consumed {} milli-units of public materials and employed {} temporary workers for {} minor currency units of wages.",
                self.program_material_consumed_milli, self.program_temporary_workers, self.program_wages_minor
            ));
        }
        if self.program_beneficiary_outcomes
            + self.program_underfulfilled_outcomes
            + self.program_excluded_outcomes
            > 0
        {
            sentences.push(format!(
                "Regional program outcomes recorded {} beneficiaries, {} underfulfilled promises, and {} exclusions.",
                self.program_beneficiary_outcomes, self.program_underfulfilled_outcomes, self.program_excluded_outcomes
            ));
        }
        if let Some((program, region, fulfillment, outcome)) = self.leading_program_loser {
            let name = program_names
                .get(&program)
                .map_or("an established program", String::as_str);
            let region = region_names
                .get(&region)
                .map_or("an unnamed region", String::as_str);
            sentences.push(format!(
                "In '{name}', {region} was the clearest regional loser at {}.{:02}% fulfillment ({outcome:?}).",
                fulfillment / 100, fulfillment % 100
            ));
        }
        if let Some((country, average, polarization, legitimacy, cohesion)) =
            self.leading_program_consequence
        {
            let country = country_names
                .get(&country)
                .map_or("an unnamed country", String::as_str);
            sentences.push(format!(
                "Program memory in {country} produced an average political shift of {average}, polarization of {polarization}, legitimacy change {legitimacy}, and elite-cohesion change {cohesion}."
            ));
        }
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
        if self.unprofitable_agreement_endings > 0 {
            sentences.push(format!(
                "Employers ended {} standing agreements covering {} workers once the wage outran what the work was worth and the employer's own cash could no longer cover the gap.",
                self.unprofitable_agreement_endings, self.unprofitable_workers_released
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
        } else if self
            .leading_service_allocation_influence
            .is_some_and(|influence| influence.4 >= 2_500)
        {
            69
        } else if self.leading_service_allocation.is_some_and(|allocation| {
            allocation.1 == crate::ServiceAllocationSource::ExplicitPoliticalDecision
                && allocation.2 >= 9_000
        }) {
            71
        } else if self
            .leading_regional_interest
            .is_some_and(|interest| interest.3 < 3_000)
        {
            74
        } else if self
            .leading_social_pressure
            .is_some_and(|pressure| pressure.4 >= 7_000)
        {
            77
        } else if self
            .leading_social_pressure
            .is_some_and(|pressure| pressure.4 >= 4_000)
        {
            69
        } else if self.migrated_households > 0
            || self.housing_projects_started > 0
            || self.housing_projects_completed > 0
        {
            68
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
            DomainEvent::EmploymentSwitched {
                from_firm: crate::FirmId::new(1),
                to_firm: crate::FirmId::new(2),
                cohort: CohortId::new(1),
                previous_wage: Money::from_minor_units(100),
                offered_wage: Money::from_minor_units(113),
                minimum_education: crate::EducationLevel::Basic,
                labor_market_adjustment_basis_points: 0,
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
        assert!(entry.text.contains(
            "Workers voluntarily changed employers 1 times for materially better offers"
        ));
        assert!(entry.text.contains("gaining 13 minor currency units"));
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

    #[test]
    fn chronicle_narrates_migration_and_housing_response() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        for (region, name) in [(RegionId::new(1), "Oldtown"), (RegionId::new(2), "Newport")] {
            world.events.append(
                date,
                DomainEvent::RegionRegistered {
                    region,
                    country: CountryId::new(1),
                    name: name.to_owned(),
                },
            );
        }
        world.events.append(
            date,
            DomainEvent::HouseholdMigrated {
                source_cohort: CohortId::new(1),
                migrant_cohort: CohortId::new(2),
                from_region: RegionId::new(1),
                to_region: RegionId::new(2),
                people: Population::new(3),
                households: 1,
                liquid_wealth: Money::from_minor_units(90),
                debt: Money::from_minor_units(4),
                relocation_fee: Money::from_minor_units(10),
                destination_vacancy_months: 7,
                service_advantage_basis_points: 500,
                destination_housing_pressure_basis_points: 9_500,
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalHousingConstructionStarted {
                region: RegionId::new(2),
                dwellings: 2,
                committed_cost: Money::from_minor_units(240),
                housing_pressure_basis_points: 9_500,
                years_remaining: 2,
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalHousingConstructionCompleted {
                region: RegionId::new(2),
                dwellings: 2,
                committed_cost: Money::from_minor_units(240),
                dwelling_capacity: 22,
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
        assert_eq!(chronicle[0].importance, 68);
        assert!(chronicle[0].text.contains("from Oldtown to Newport"));
        assert!(chronicle[0].text.contains("paid 10"));
        assert!(chronicle[0].text.contains("committed 240"));
        assert!(chronicle[0].text.contains("completed 2 dwellings"));
    }

    #[test]
    fn chronicle_explains_material_pressure_and_legitimacy_effect() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::CountryRegistered {
                country: CountryId::new(1),
                name: "Commonwealth".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: RegionId::new(1),
                country: CountryId::new(1),
                name: "Iron Valley".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalSocialPressureUpdated {
                region: RegionId::new(1),
                chronic_unemployment: BasisPoints::new(8_000).expect("pressure"),
                livelihood_stress: BasisPoints::new(7_000).expect("pressure"),
                public_service_shortfall: BasisPoints::new(5_000).expect("pressure"),
                combined: BasisPoints::new(7_000).expect("pressure"),
            },
        );
        world.events.append(
            date,
            DomainEvent::CountryLegitimacyPressureApplied {
                country: CountryId::new(1),
                population_weighted_pressure: BasisPoints::new(7_000).expect("pressure"),
                legitimacy_effect: -200,
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
        assert_eq!(chronicle[0].importance, 77);
        assert!(
            chronicle[0]
                .text
                .contains("peaked at 7000 basis points in Iron Valley")
        );
        assert!(
            chronicle[0]
                .text
                .contains("reduced legitimacy in Commonwealth by 200")
        );
    }

    #[test]
    fn chronicle_names_regional_interests_fiscal_incidence_and_confidence() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::CountryRegistered {
                country: CountryId::new(1),
                name: "Union".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: RegionId::new(1),
                country: CountryId::new(1),
                name: "Harbor".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalPolicyOutcomeRecorded {
                region: RegionId::new(1),
                taxes_paid: Money::from_minor_units(90),
                service_allocation: Money::from_minor_units(40),
                net_transfer: Money::from_minor_units(-50),
                fiscal_position: crate::RegionalFiscalPosition::NetContributor,
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalInterestUpdated {
                region: RegionId::new(1),
                previous_priority: crate::RegionalPolicyPriority::Stability,
                priority: crate::RegionalPolicyPriority::Employment,
                priority_pressure: BasisPoints::new(6_000).expect("pressure"),
                satisfaction: BasisPoints::new(2_900).expect("satisfaction"),
                years_persistent: 1,
            },
        );
        world.events.append(
            date,
            DomainEvent::CountryRegionalConfidenceApplied {
                country: CountryId::new(1),
                population_weighted_confidence: BasisPoints::new(4_000).expect("confidence"),
                legitimacy_effect: -40,
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
        assert_eq!(chronicle[0].importance, 74);
        assert!(chronicle[0].text.contains("Harbor adopted employment"));
        assert!(chronicle[0].text.contains("net contributors"));
        assert!(
            chronicle[0]
                .text
                .contains("reduced legitimacy in Union by 40")
        );
    }

    #[test]
    fn chronicle_exposes_unrestricted_explicit_service_concentration() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(7), date);
        world.events.append(
            date,
            DomainEvent::RegionRegistered {
                region: RegionId::new(1),
                country: CountryId::new(1),
                name: "Capital".to_owned(),
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalServiceBudgetAllocated {
                country: CountryId::new(1),
                region: RegionId::new(1),
                source: crate::ServiceAllocationSource::ExplicitPoliticalDecision,
                share: BasisPoints::FULL,
                service_budget: Money::from_minor_units(500),
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
        assert_eq!(chronicle[0].importance, 71);
        assert!(
            chronicle[0]
                .text
                .contains("explicit political decision directed 100.00%")
        );
        assert!(
            chronicle[0]
                .text
                .contains("500 minor currency units, to Capital")
        );
    }

    #[test]
    fn chronicle_names_actor_whose_influence_biases_service_allocation() {
        let date = SimDate::new(2025, 1).expect("date");
        let mut world = World::new(WorldSeed::new(9), date);
        world
            .register_country(crate::Country::new(CountryId::new(1), "Union").expect("country"))
            .expect("country");
        world
            .register_region(
                crate::Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Harbor",
                    crate::Population::new(100),
                    Money::from_minor_units(1_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_actor(
                crate::Actor::new(ActorId::new(2), "Industrial patron", RegionId::new(1), 1975)
                    .expect("actor"),
            )
            .expect("actor");
        world.events.append(
            date,
            DomainEvent::RegionalServiceAllocationInfluenceApplied {
                country: CountryId::new(1),
                actor: ActorId::new(2),
                office: crate::PowerNodeId::new(1),
                region: RegionId::new(1),
                kind: crate::ServiceAllocationInfluenceKind::InfluenceEdge,
                weight: BasisPoints::new(8_000).expect("weight"),
                score_bonus: 4_000,
            },
        );
        world.events.append(
            date,
            DomainEvent::RegionalServiceBudgetAllocated {
                country: CountryId::new(1),
                region: RegionId::new(1),
                source: crate::ServiceAllocationSource::AutonomousPrudent,
                share: BasisPoints::FULL,
                service_budget: Money::from_minor_units(500),
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
        assert_eq!(chronicle[0].importance, 69);
        assert!(
            chronicle[0]
                .text
                .contains("Industrial patron's influence over the political office")
        );
        assert!(
            chronicle[0]
                .text
                .contains("added 4000 decision points toward Harbor")
        );
    }
}
