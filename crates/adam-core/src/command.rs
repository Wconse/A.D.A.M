use crate::{
    ActorId, BasisPoints, BoardResolution, BoardVote, ContractId, FirmExpectations, FirmId,
    FirmPolicy, FirmReorganizationPlan, FreightContract, InvestmentProject, MarketClearing,
    ProjectId, ResolutionId, ShipmentId, ShipmentOrder, TerminalId, World, WorldError,
};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldCommand {
    AdvanceEconomicYear,
    AdvanceMonth,
    ExecuteMonthlyEconomicCycle,
    ExecuteMonthlyCommercialCycle,
    ExecuteObservedFirmManagement,
    ExecuteObservedFirmReorganizations,
    ExecuteObservedFirmLiquidations,
    ExecuteObservedEmergencyRelief,
    SetGovernmentEmergencyPolicy {
        actor: ActorId,
        country: crate::CountryId,
        policy: crate::GovernmentEmergencyPolicy,
    },
    SetCountryHostility {
        first: crate::CountryId,
        second: crate::CountryId,
        active: bool,
    },
    IssueEmergencyReliefDebt {
        actor: ActorId,
        country: crate::CountryId,
        amount: crate::Money,
    },
    FundEmergencyRelief {
        actor: ActorId,
        cohort: crate::CohortId,
        amount: crate::Money,
    },
    SetFirmProductionTarget {
        actor: ActorId,
        firm: FirmId,
        batches: u64,
    },
    ReorganizeFirm(FirmReorganizationPlan),
    CaptureMonthlyFirmObservation {
        firm: FirmId,
    },
    DeriveFirmExpectationsFromObservations {
        firm: FirmId,
        horizon_months: u16,
    },
    UpdateFirmExpectations {
        firm: FirmId,
        expectations: FirmExpectations,
    },
    ResetMonthlyFirmAccounts,
    ChangeEmploymentWorkers {
        firm: FirmId,
        cohort: crate::CohortId,
        workers: u64,
    },
    ExecuteMonthlyPayroll,
    UpdateMonthlyCohortHealth,
    UpdateMonthlyCohortExperience,
    AccumulateMonthlySocialStress,
    DeriveMonthlySocialStress,
    SettleLocalMarket(MarketClearing),
    ExecuteMonthlyHouseholdCashflows,
    ExecuteMonthlyHouseholdCoping,
    ExecuteMonthlyProduction,
    EnqueueTerminalShipment {
        terminal: TerminalId,
        shipment: ShipmentId,
    },
    AdmitTerminalShipments {
        terminal: TerminalId,
    },
    RegisterFreightContract(FreightContract),
    ActivateFreightContract(ContractId),
    StartInventoryShipment {
        source: FirmId,
        destination: FirmId,
        order: ShipmentOrder,
        max_legs: usize,
    },
    AdvanceInventoryShipment {
        shipment: ShipmentId,
        days: u32,
    },
    LaunchInvestmentProject(InvestmentProject),
    AdvanceInvestmentProject(ProjectId),
    ProposeBoardResolution(BoardResolution),
    ExecuteBoardResolution(ResolutionId),
    CastBoardVote {
        resolution: ResolutionId,
        actor: ActorId,
        vote: BoardVote,
    },
    CloseBoardResolution {
        resolution: ResolutionId,
        quorum: BasisPoints,
        approval: BasisPoints,
    },
    AdvanceYears(u32),
    SetMarketingBudget {
        actor: ActorId,
        firm: FirmId,
        value: BasisPoints,
    },
    SetInventoryBuffer {
        actor: ActorId,
        firm: FirmId,
        days: u16,
    },
    SetFirmPolicy {
        actor: ActorId,
        firm: FirmId,
        policy: FirmPolicy,
    },
    RegisterFirm(crate::Firm),
    IssueFirmCredit {
        creditor: ActorId,
        firm: FirmId,
        priority: crate::FirmCreditorPriority,
        principal: crate::Money,
    },
}
impl WorldCommand {
    /// Applies the same deterministic command regardless of player or AI origin.
    /// # Errors
    /// Returns [`WorldError`] if the authoritative transition cannot complete.
    #[allow(clippy::too_many_lines)]
    pub fn apply(&self, world: &mut World) -> Result<(), WorldError> {
        match self {
            Self::AdvanceEconomicYear => world.advance_economic_year().map(|_| ()),
            Self::AdvanceMonth => world.advance_month(),
            Self::ExecuteMonthlyEconomicCycle => world.execute_monthly_economic_cycle().map(|_| ()),
            Self::ExecuteMonthlyCommercialCycle => {
                world.execute_monthly_commercial_cycle().map(|_| ())
            }
            Self::ExecuteObservedFirmManagement => {
                world.execute_observed_firm_management().map(|_| ())
            }
            Self::ExecuteObservedFirmReorganizations => {
                world.execute_observed_firm_reorganizations().map(|_| ())
            }
            Self::ExecuteObservedFirmLiquidations => {
                world.execute_observed_firm_liquidations().map(|_| ())
            }
            Self::ExecuteObservedEmergencyRelief => {
                world.execute_observed_emergency_relief().map(|_| ())
            }
            Self::SetGovernmentEmergencyPolicy {
                actor,
                country,
                policy,
            } => world.set_government_emergency_policy(*actor, *country, *policy),
            Self::SetCountryHostility {
                first,
                second,
                active,
            } => world.set_country_hostility(*first, *second, *active),
            Self::IssueEmergencyReliefDebt {
                actor,
                country,
                amount,
            } => world.issue_emergency_relief_debt(*actor, *country, *amount),
            Self::FundEmergencyRelief {
                actor,
                cohort,
                amount,
            } => world.fund_emergency_relief(*actor, *cohort, *amount),
            Self::SetFirmProductionTarget {
                actor,
                firm,
                batches,
            } => world.set_firm_production_target(*actor, *firm, *batches),
            Self::ReorganizeFirm(plan) => world.reorganize_firm(plan).map(|_| ()),
            Self::CaptureMonthlyFirmObservation { firm } => {
                world.capture_monthly_firm_observation(*firm).map(|_| ())
            }
            Self::DeriveFirmExpectationsFromObservations {
                firm,
                horizon_months,
            } => world
                .derive_firm_expectations_from_observations(*firm, *horizon_months)
                .map(|_| ()),
            Self::UpdateFirmExpectations { firm, expectations } => {
                world.update_firm_expectations(*firm, *expectations)
            }
            Self::ResetMonthlyFirmAccounts => {
                world.reset_monthly_firm_accounts();
                Ok(())
            }
            Self::ChangeEmploymentWorkers {
                firm,
                cohort,
                workers,
            } => world.change_employment_workers(*firm, *cohort, *workers),
            Self::ExecuteMonthlyPayroll => world.execute_monthly_payroll().map(|_| ()),
            Self::UpdateMonthlyCohortHealth => world.update_monthly_cohort_health(),
            Self::UpdateMonthlyCohortExperience => world.update_monthly_cohort_experience(),
            Self::AccumulateMonthlySocialStress => world.accumulate_monthly_social_stress(),
            Self::DeriveMonthlySocialStress => world.derive_monthly_social_stress(),
            Self::SettleLocalMarket(clearing) => world.settle_local_market(clearing),
            Self::ExecuteMonthlyHouseholdCashflows => {
                world.execute_monthly_household_cashflows().map(|_| ())
            }
            Self::ExecuteMonthlyHouseholdCoping => {
                world.execute_monthly_household_coping().map(|_| ())
            }
            Self::ExecuteMonthlyProduction => world.execute_monthly_production().map(|_| ()),
            Self::EnqueueTerminalShipment { terminal, shipment } => {
                world.enqueue_terminal_shipment(*terminal, *shipment)
            }
            Self::AdmitTerminalShipments { terminal } => {
                world.admit_terminal_shipments(*terminal).map(|_| ())
            }
            Self::RegisterFreightContract(contract) => {
                world.register_freight_contract(contract.clone())
            }
            Self::ActivateFreightContract(id) => world.activate_freight_contract(*id),
            Self::StartInventoryShipment {
                source,
                destination,
                order,
                max_legs,
            } => world.start_inventory_shipment(*source, *destination, order, *max_legs),
            Self::AdvanceInventoryShipment { shipment, days } => {
                world.advance_inventory_shipment(*shipment, *days)
            }
            Self::LaunchInvestmentProject(project) => {
                world.launch_investment_project(project.clone())
            }
            Self::AdvanceInvestmentProject(id) => world.advance_investment_project(*id),
            Self::ProposeBoardResolution(value) => world.propose_board_resolution(value.clone()),
            Self::ExecuteBoardResolution(id) => world.execute_board_resolution(*id),
            Self::CastBoardVote {
                resolution,
                actor,
                vote,
            } => world.cast_board_vote(*resolution, *actor, *vote),
            Self::CloseBoardResolution {
                resolution,
                quorum,
                approval,
            } => world
                .close_board_resolution(*resolution, *quorum, *approval)
                .map(|_| ()),
            Self::AdvanceYears(years) => world.advance_years(*years),
            Self::SetMarketingBudget { actor, firm, value } => {
                world.set_marketing_budget(*actor, *firm, *value)
            }
            Self::SetInventoryBuffer { actor, firm, days } => {
                world.set_inventory_buffer(*actor, *firm, *days)
            }
            Self::SetFirmPolicy {
                actor,
                firm,
                policy,
            } => world.set_firm_policy(*actor, *firm, *policy),
            Self::RegisterFirm(firm) => world.register_firm(firm.clone()),
            Self::IssueFirmCredit {
                creditor,
                firm,
                priority,
                principal,
            } => world.issue_firm_credit(*creditor, *firm, *priority, *principal),
        }
    }
}
/// Applies commands strictly in recorded order.
/// # Errors
/// Returns the first transition error and does not apply later commands.
pub fn replay_commands(world: &mut World, commands: &[WorldCommand]) -> Result<(), WorldError> {
    for command in commands {
        command.apply(world)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Country, CountryId, SimDate, WorldSeed};
    #[test]
    fn ordered_commands_match_direct_transition() {
        let mut direct = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        direct
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register");
        let mut replayed = direct.clone();
        direct.advance_years(5).expect("years");
        replay_commands(
            &mut replayed,
            &[WorldCommand::AdvanceYears(2), WorldCommand::AdvanceYears(3)],
        )
        .expect("replay");
        assert_eq!(direct, replayed);
    }
    #[test]
    fn firm_registration_is_replayable_and_atomic() {
        use crate::{
            Firm, Good, GoodId, Money, Population, ProductionRecipe, QuantityMilli, RecipeId,
            Region, RegionId,
        };
        let mut direct = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        direct
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register");
        direct
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(1),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("region");
        direct
            .register_good(Good::new(GoodId::new(1), "Grain").expect("good"))
            .expect("good");
        direct
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Grain recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        let mut replayed = direct.clone();
        let firm = Firm::new(
            FirmId::new(1),
            "Farm",
            RegionId::new(1),
            RecipeId::new(1),
            1,
            1,
            Money::from_minor_units(1),
            std::collections::BTreeMap::new(),
        )
        .expect("firm");
        direct.register_firm(firm.clone()).expect("direct firm");
        replay_commands(&mut replayed, &[WorldCommand::RegisterFirm(firm.clone())])
            .expect("replay firm");
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        let before = replayed.clone();
        assert!(matches!(
            WorldCommand::RegisterFirm(firm).apply(&mut replayed),
            Err(WorldError::DuplicateFirm(_))
        ));
        assert_eq!(replayed, before);
    }
}
