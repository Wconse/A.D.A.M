use crate::{
    ActorId, BasisPoints, BoardResolution, BoardVote, ContractId, FirmId, FirmPolicy,
    FreightContract, InvestmentProject, MarketClearing, ProjectId, ResolutionId, ShipmentId,
    ShipmentOrder, TerminalId, World, WorldError,
};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldCommand {
    AccumulateMonthlySocialStress,
    DeriveMonthlySocialStress,
    SettleLocalMarket(MarketClearing),
    ExecuteMonthlyHouseholdCashflows,
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
}
impl WorldCommand {
    /// Applies the same deterministic command regardless of player or AI origin.
    /// # Errors
    /// Returns [`WorldError`] if the authoritative transition cannot complete.
    pub fn apply(&self, world: &mut World) -> Result<(), WorldError> {
        match self {
            Self::AccumulateMonthlySocialStress => world.accumulate_monthly_social_stress(),
            Self::DeriveMonthlySocialStress => world.derive_monthly_social_stress(),
            Self::SettleLocalMarket(clearing) => world.settle_local_market(clearing),
            Self::ExecuteMonthlyHouseholdCashflows => {
                world.execute_monthly_household_cashflows().map(|_| ())
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
}
