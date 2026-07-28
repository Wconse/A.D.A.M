use crate::{
    ActorId, BasisPoints, BoardVote, CohortId, ContractId, CountryId, FirmExpectationSource,
    FirmId, GoodId, Money, NeedProfileId, Population, PowerNodeId, ProjectId, QuantityMilli,
    RatePpm, RegionId, ResolutionId, ResolutionStatus, RouteId, ShipmentId, SimDate, TerminalId,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DomainEvent {
    EconomicYearCompleted {
        closed_year: i32,
        monthly_cycles: u8,
    },
    MonthAdvanced {
        date: SimDate,
    },
    MonthlyEconomicCycleCompleted {
        payroll_records: u64,
        household_cashflows: u64,
        market_fills: u64,
    },
    MonthlyCommercialCycleCompleted {
        production_batches: u64,
        seller_offers: u64,
        market_fills: u64,
        firms_observed: u64,
    },
    SurvivalRationingApplied {
        country: CountryId,
        region: RegionId,
        good: GoodId,
        requested: QuantityMilli,
        available: QuantityMilli,
        cohorts: u64,
    },
    GovernmentEmergencyPolicySet {
        actor: ActorId,
        country: CountryId,
        strategy: crate::EmergencyReliefStrategy,
    },
    EmergencyReliefDebtIssued {
        actor: ActorId,
        country: CountryId,
        amount: Money,
    },
    EmergencyReliefFunded {
        actor: ActorId,
        country: CountryId,
        cohort: CohortId,
        amount: Money,
    },
    ObservedEmergencyReliefCompleted {
        exposed_cohorts: u64,
        funded_cohorts: u64,
    },
    ObservedFirmManagementCompleted {
        firms_reviewed: u64,
        targets_changed: u64,
    },
    FirmProductionTargetSet {
        actor: ActorId,
        firm: FirmId,
        previous_batches: Option<u64>,
        target_batches: u64,
    },
    FirmOperatingObservationCaptured {
        firm: FirmId,
        sales_revenue: Money,
        final_sales_revenue: Money,
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
    CohortHealthUpdated {
        cohort: CohortId,
        survival_fulfillment: BasisPoints,
        functional_capacity: BasisPoints,
        excess_deaths: u64,
        survivors: Population,
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
    HouseholdSurvivalBorrowed {
        cohort: CohortId,
        amount: Money,
        ending_wealth: Money,
        ending_debt: Money,
    },
    HouseholdCashflowApplied {
        cohort: CohortId,
        income: Money,
        debt_service: Money,
        ending_wealth: Money,
        ending_debt: Money,
    },
    FirmSalesTaxPaid {
        firm: FirmId,
        country: CountryId,
        taxable_sales: Money,
        liability: Money,
        paid: Money,
    },
    FirmProcurementTrade {
        buyer: FirmId,
        seller: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
        spend: Money,
    },
    /// Input demand that remained unfilled after the monthly B2B offer book
    /// settled. This is journal evidence only: it does not change world state.
    FirmProcurementShortfall {
        buyer: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
    },
    BilateralHostilityChanged {
        first: CountryId,
        second: CountryId,
        active: bool,
    },
    BilateralGrievanceChanged {
        aggrieved: CountryId,
        target: CountryId,
        level: BasisPoints,
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
    RegionalOutputMeasured {
        region: RegionId,
        final_consumption: Money,
        inventory_change: Money,
        annual_output: Money,
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
    PublicDebtInterestCharged {
        country: CountryId,
        opening_debt: Money,
        interest: Money,
    },
    CountryPoliticsChanged {
        country: CountryId,
        legitimacy: BasisPoints,
        elite_cohesion: BasisPoints,
    },
    YearAdvanced {
        year: i32,
    },
    /// Input demand left unfilled specifically because a reachable import
    /// route exhausted its shared monthly capacity while foreign stock remained.
    FirmProcurementRouteCapacityShortfall {
        buyer: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
    },
    /// Freight component of a delivered household import paid to the route carrier.
    MarketFreightPaid {
        buyer: CohortId,
        seller: FirmId,
        carrier: FirmId,
        route: RouteId,
        amount: Money,
    },
    /// Freight component of a delivered B2B import paid to the route carrier.
    FirmProcurementFreightPaid {
        buyer: FirmId,
        seller: FirmId,
        carrier: FirmId,
        route: RouteId,
        amount: Money,
    },
    RouteCapacityExpanded {
        route: RouteId,
        carrier: FirmId,
        previous_capacity: QuantityMilli,
        added_capacity: QuantityMilli,
        cost: Money,
    },
    /// Ordinary firm operations entered insolvency with an auditable asset-and-claim snapshot.
    FirmInsolvencyDeclared {
        firm: FirmId,
        administrator: ActorId,
        cash: Money,
        wage_arrears: Money,
        inventory: Vec<(GoodId, QuantityMilli)>,
    },
    /// A bounded administration ended without a viable plan; worker claims had priority,
    /// residual cash went to owners, and unsold inventory was explicitly written off.
    FirmLiquidated {
        firm: FirmId,
        administrator: ActorId,
        claims_paid: Money,
        claims_written_off: Money,
        inventory_written_off: Vec<(GoodId, QuantityMilli)>,
        capacity_written_off: u64,
        owner_distribution: Money,
    },
    /// Insolvency administration paid worker claims and reopened a funded operating plan.
    FirmReorganized {
        firm: FirmId,
        administrator: ActorId,
        sponsor: ActorId,
        contribution: Money,
        claims_paid: Money,
        workers: u64,
        production_target: u64,
        cash_reserve: Money,
    },
    /// Authorized management reduced employment after owners could not finance persistent distress.
    FirmDownsizedForDistress {
        firm: FirmId,
        actor: ActorId,
        previous_workers: u64,
        current_workers: u64,
        wage_arrears: Money,
    },
    /// Owner cash transferred into a distressed firm after persistent wage arrears.
    FirmRecapitalized {
        firm: FirmId,
        owner: ActorId,
        amount: Money,
        wage_arrears: Money,
    },
    /// Sovereign debt reduced through an involuntary restructuring after debt
    /// service became unsustainable relative to output and tax revenue.
    PublicDebtRestructured {
        country: CountryId,
        debt_before: Money,
        debt_after: Money,
        principal_written_off: Money,
    },
    /// A solvent local producer bought physical inventory from a liquidating estate.
    /// Appended to preserve the serialized ordering of earlier journal variants.
    FirmLiquidationInventorySold {
        estate: FirmId,
        buyer: FirmId,
        good: GoodId,
        quantity: QuantityMilli,
        proceeds: Money,
    },
    /// Real creditor cash entered a firm in exchange for a ranked principal claim.
    FirmCreditIssued {
        firm: FirmId,
        creditor: ActorId,
        priority: crate::FirmCreditorPriority,
        principal: Money,
    },
    /// A liquidation paid or explicitly wrote off one ranked creditor claim.
    FirmCreditorClaimSettled {
        firm: FirmId,
        creditor: ActorId,
        priority: crate::FirmCreditorPriority,
        paid: Money,
        written_off: Money,
    },
    /// A solvent compatible successor bought installed production capacity from an estate.
    FirmLiquidationCapacitySold {
        estate: FirmId,
        buyer: FirmId,
        capacity_batches: u64,
        proceeds: Money,
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
