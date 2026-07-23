use crate::{
    ActorId, BasisPoints, BoardVote, CohortId, ContractId, CountryId, FirmExpectationSource,
    FirmId, GoodId, Money, NeedProfileId, Population, PowerNodeId, ProjectId, QuantityMilli,
    RatePpm, RegionId, ResolutionId, ResolutionStatus, RouteId, ShipmentId, SimDate, TerminalId,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DomainEvent {
    FirmOperatingObservationCaptured {
        firm: FirmId,
        sales_revenue: Money,
        produced_batches: u64,
        input_prices: Vec<(GoodId, Money)>,
        market_outcomes: Vec<crate::MarketOfferOutcome>,
    },
    FirmExpectationsUpdated {
        firm: FirmId,
        expected_sales_revenue: Money,
        expected_input_costs: Money,
        expected_financing: Money,
        horizon_months: u16,
        source: FirmExpectationSource,
    },
    CohortExperienceUpdated {
        cohort: CohortId,
        survival_shortage_months: u32,
        unemployment_months: u32,
        debt_distress_months: u32,
    },
    SocialStressUpdated {
        cohort: CohortId,
        health_risk: BasisPoints,
        unrest_pressure: BasisPoints,
    },
    HouseholdCashflowApplied {
        cohort: CohortId,
        income: Money,
        debt_service: Money,
        ending_wealth: Money,
        ending_debt: Money,
    },
    MarketTrade {
        buyer: CohortId,
        seller: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
        spend: Money,
    },
    EmploymentChanged {
        firm: FirmId,
        cohort: CohortId,
        previous_workers: u64,
        current_workers: u64,
    },
    PayrollSettled {
        firm: FirmId,
        cohort: CohortId,
        owed: Money,
        paid: Money,
        arrears: Money,
    },
    ProductionCompleted {
        firm: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
        batches: u64,
    },
    ShipmentQueuedAtTerminal {
        shipment: ShipmentId,
        terminal: TerminalId,
    },
    ShipmentAdmittedToTerminal {
        shipment: ShipmentId,
        terminal: TerminalId,
    },
    FreightContractRegistered {
        contract: ContractId,
        shipper: FirmId,
        carrier: FirmId,
        route: RouteId,
    },
    FreightContractActivated {
        contract: ContractId,
    },
    FreightContractExpired {
        contract: ContractId,
    },
    WorldFounded {
        seed: u64,
    },
    ShipmentStarted {
        shipment: ShipmentId,
        good: GoodId,
        source: FirmId,
        destination: FirmId,
        quantity: QuantityMilli,
        total_cost: Money,
    },
    ShipmentDelivered {
        shipment: ShipmentId,
        good: GoodId,
        destination: FirmId,
        quantity: QuantityMilli,
    },
    BoardResolutionProposed {
        resolution: ResolutionId,
        firm: FirmId,
        proposer: ActorId,
    },
    BoardVoteCast {
        resolution: ResolutionId,
        actor: ActorId,
        vote: BoardVote,
    },
    DividendPaid {
        firm: FirmId,
        amount: Money,
    },
    InvestmentProjectLaunched {
        project: ProjectId,
        firm: FirmId,
        budget: Money,
    },
    InvestmentProjectAdvanced {
        project: ProjectId,
    },
    InvestmentProjectCompleted {
        project: ProjectId,
        firm: FirmId,
    },
    InvestmentCommitted {
        firm: FirmId,
        amount: Money,
    },
    BoardResolutionExecuted {
        resolution: ResolutionId,
    },
    BoardResolutionClosed {
        resolution: ResolutionId,
        status: ResolutionStatus,
    },
    GoodRegistered {
        good: GoodId,
        name: String,
    },
    ConsumptionProfileRegistered {
        profile: NeedProfileId,
        name: String,
    },
    RegionalPriceSet {
        region: RegionId,
        good: GoodId,
        price: Money,
    },
    CountryRegistered {
        country: CountryId,
        name: String,
    },
    RegionRegistered {
        region: RegionId,
        country: CountryId,
        name: String,
    },
    ActorRegistered {
        actor: ActorId,
        home_region: RegionId,
        name: String,
    },
    PowerNodeRegistered {
        node: PowerNodeId,
        country: CountryId,
        name: String,
    },
    InfluenceEstablished {
        actor: ActorId,
        node: PowerNodeId,
        weight: BasisPoints,
    },
    HouseholdCohortRegistered {
        cohort: CohortId,
        region: RegionId,
        people: Population,
    },
    HouseholdCohortPopulationChanged {
        cohort: CohortId,
        people: Population,
    },
    RegionPopulationChanged {
        region: RegionId,
        population: Population,
        rate: RatePpm,
    },
    RegionOutputChanged {
        region: RegionId,
        annual_output: Money,
        rate: RatePpm,
    },
    CountryFiscalYearClosed {
        country: CountryId,
        revenue: Money,
        spending: Money,
        treasury: Money,
        debt: Money,
    },
    CountryPoliticsChanged {
        country: CountryId,
        legitimacy: BasisPoints,
        elite_cohesion: BasisPoints,
    },
    YearAdvanced {
        year: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventEnvelope {
    sequence: u64,
    date: SimDate,
    event: DomainEvent,
}
impl EventEnvelope {
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }
    #[must_use]
    pub const fn event(&self) -> &DomainEvent {
        &self.event
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EventLog {
    events: Vec<EventEnvelope>,
}
impl EventLog {
    pub fn append(&mut self, date: SimDate, event: DomainEvent) {
        let sequence = self.events.len() as u64;
        self.events.push(EventEnvelope {
            sequence,
            date,
            event,
        });
    }
    #[must_use]
    pub fn events(&self) -> &[EventEnvelope] {
        &self.events
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_numbers_are_monotonic() {
        let date = SimDate::new(2025, 1).expect("valid date");
        let mut log = EventLog::default();
        log.append(date, DomainEvent::WorldFounded { seed: 1 });
        log.append(date, DomainEvent::YearAdvanced { year: 2026 });
        assert_eq!(log.events()[0].sequence(), 0);
        assert_eq!(log.events()[1].sequence(), 1);
    }
}
