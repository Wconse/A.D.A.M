use crate::{TerminalCapacityLedger, TerminalId, TerminalQueue};
use std::collections::BTreeMap;
use std::fmt;

use crate::{
    ActorId, BasisPoints, BoardResolution, BoardVote, CohortId, ConsumptionProfile, ContractId,
    CorporateAction, CorporateRole, CountryId, DomainEvent, EventLog, Firm, FirmAppointment,
    FirmId, FirmPolicy, FreightCapacityLedger, FreightContract, Good, GoodId, HouseholdCohort,
    InventoryShipment, InvestmentProject, LogisticsRoute, LogisticsTerminal, Money, NeedProfileId,
    OwnershipStake, Population, PowerNodeId, ProductionRecipe, ProjectId, RecipeId, RegionId,
    ResolutionId, ResolutionStatus, RouteCapacityLedger, RouteId, RouteOperatingCost, ShipmentId,
    SimDate, TimeError, WorldSeed,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CountryIndicators {
    treasury: Money,
    public_debt: Money,
    legitimacy: BasisPoints,
    elite_cohesion: BasisPoints,
}

impl CountryIndicators {
    #[must_use]
    pub const fn new(
        treasury: Money,
        public_debt: Money,
        legitimacy: BasisPoints,
        elite_cohesion: BasisPoints,
    ) -> Self {
        Self {
            treasury,
            public_debt,
            legitimacy,
            elite_cohesion,
        }
    }
    #[must_use]
    pub const fn treasury(self) -> Money {
        self.treasury
    }
    #[must_use]
    pub const fn public_debt(self) -> Money {
        self.public_debt
    }
    #[must_use]
    pub const fn legitimacy(self) -> BasisPoints {
        self.legitimacy
    }
    #[must_use]
    pub const fn elite_cohesion(self) -> BasisPoints {
        self.elite_cohesion
    }
    pub(crate) const fn set_treasury(&mut self, value: Money) {
        self.treasury = value;
    }
    pub(crate) const fn set_public_debt(&mut self, value: Money) {
        self.public_debt = value;
    }
    pub(crate) const fn set_legitimacy(&mut self, value: BasisPoints) {
        self.legitimacy = value;
    }
    pub(crate) const fn set_elite_cohesion(&mut self, value: BasisPoints) {
        self.elite_cohesion = value;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Country {
    id: CountryId,
    name: String,
    indicators: CountryIndicators,
}

impl Country {
    /// Creates a country with a stable identity.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyName`] when the name is empty or whitespace.
    pub fn new(id: CountryId, name: impl Into<String>) -> Result<Self, WorldError> {
        Ok(Self {
            id,
            name: validated_name("country", name.into())?,
            indicators: CountryIndicators::new(
                Money::from_minor_units(0),
                Money::from_minor_units(0),
                BasisPoints::HALF,
                BasisPoints::HALF,
            ),
        })
    }

    #[must_use]
    pub const fn id(&self) -> CountryId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn with_indicators(mut self, indicators: CountryIndicators) -> Self {
        self.indicators = indicators;
        self
    }

    #[must_use]
    pub const fn indicators(&self) -> CountryIndicators {
        self.indicators
    }

    pub(crate) const fn indicators_mut(&mut self) -> &mut CountryIndicators {
        &mut self.indicators
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Region {
    id: RegionId,
    country: CountryId,
    name: String,
    population: Population,
    annual_output: Money,
}

impl Region {
    /// Creates an aggregate simulation region.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyName`] when the name is empty or whitespace.
    pub fn new(
        id: RegionId,
        country: CountryId,
        name: impl Into<String>,
        population: Population,
        annual_output: Money,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            id,
            country,
            name: validated_name("region", name.into())?,
            population,
            annual_output,
        })
    }

    #[must_use]
    pub const fn id(&self) -> RegionId {
        self.id
    }

    #[must_use]
    pub const fn country(&self) -> CountryId {
        self.country
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn population(&self) -> Population {
        self.population
    }

    #[must_use]
    pub const fn annual_output(&self) -> Money {
        self.annual_output
    }

    pub(crate) const fn set_population(&mut self, value: Population) {
        self.population = value;
    }

    pub(crate) const fn set_annual_output(&mut self, value: Money) {
        self.annual_output = value;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Actor {
    id: ActorId,
    name: String,
    home_region: RegionId,
    born_year: i32,
}

impl Actor {
    /// Creates a named human actor.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyName`] when the name is empty or whitespace.
    pub fn new(
        id: ActorId,
        name: impl Into<String>,
        home_region: RegionId,
        born_year: i32,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            id,
            name: validated_name("actor", name.into())?,
            home_region,
            born_year,
        })
    }

    #[must_use]
    pub const fn id(&self) -> ActorId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn home_region(&self) -> RegionId {
        self.home_region
    }

    #[must_use]
    pub const fn born_year(&self) -> i32 {
        self.born_year
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum PowerNodeKind {
    PoliticalOffice,
    Capital,
    MilitaryCommand,
    MediaPlatform,
    CivicOrganization,
}

impl PowerNodeKind {
    const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::PoliticalOffice => 1,
            Self::Capital => 2,
            Self::MilitaryCommand => 3,
            Self::MediaPlatform => 4,
            Self::CivicOrganization => 5,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PowerNode {
    id: PowerNodeId,
    country: CountryId,
    name: String,
    kind: PowerNodeKind,
    holder: Option<ActorId>,
}

impl PowerNode {
    /// Creates a node in the power network.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyName`] when the name is empty or whitespace.
    pub fn new(
        id: PowerNodeId,
        country: CountryId,
        name: impl Into<String>,
        kind: PowerNodeKind,
        holder: Option<ActorId>,
    ) -> Result<Self, WorldError> {
        Ok(Self {
            id,
            country,
            name: validated_name("power node", name.into())?,
            kind,
            holder,
        })
    }

    #[must_use]
    pub const fn id(&self) -> PowerNodeId {
        self.id
    }

    #[must_use]
    pub const fn country(&self) -> CountryId {
        self.country
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn kind(&self) -> PowerNodeKind {
        self.kind
    }

    #[must_use]
    pub const fn holder(&self) -> Option<ActorId> {
        self.holder
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Influence {
    actor: ActorId,
    node: PowerNodeId,
    weight: BasisPoints,
}

impl Influence {
    #[must_use]
    pub const fn new(actor: ActorId, node: PowerNodeId, weight: BasisPoints) -> Self {
        Self {
            actor,
            node,
            weight,
        }
    }

    #[must_use]
    pub const fn actor(self) -> ActorId {
        self.actor
    }

    #[must_use]
    pub const fn node(self) -> PowerNodeId {
        self.node
    }

    #[must_use]
    pub const fn weight(self) -> BasisPoints {
        self.weight
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    DuplicateCountry(CountryId),
    DuplicateGood(GoodId),
    DuplicateFirm(FirmId),
    DuplicateFirmAppointment,
    DuplicateBoardResolution(ResolutionId),
    DuplicateInvestmentProject(ProjectId),
    DuplicateOwnershipStake {
        firm: FirmId,
        owner: ActorId,
    },
    OwnershipExceedsFull(FirmId),
    DuplicateRecipe(RecipeId),
    DuplicateNeedProfile(NeedProfileId),
    DuplicateCohort(CohortId),
    DuplicateRegion(RegionId),
    DuplicateActor(ActorId),
    DuplicatePowerNode(PowerNodeId),
    DuplicateInfluence {
        actor: ActorId,
        node: PowerNodeId,
    },
    UnknownCountry(CountryId),
    UnknownGood(GoodId),
    UnknownFirm(FirmId),
    UnknownRecipe(RecipeId),
    UnknownNeedProfile(NeedProfileId),
    MissingRegionalPrice {
        region: RegionId,
        good: GoodId,
    },
    UnknownRegion(RegionId),
    UnknownActor(ActorId),
    UnknownPowerNode(PowerNodeId),
    EmptyName(&'static str),
    InvalidCohort(&'static str),
    InvalidConsumptionProfile(&'static str),
    InvalidPrice,
    InsufficientHouseholdCash(CohortId),
    UnknownCohort(CohortId),
    InvalidProduction(&'static str),
    InvalidBusinessPolicy(&'static str),
    MissingFirmPolicy(FirmId),
    UnknownBoardResolution(ResolutionId),
    UnknownInvestmentProject(ProjectId),
    InsufficientCommittedInvestment(FirmId),
    InvalidInvestmentProject(&'static str),
    InvalidLogistics(&'static str),
    InvalidFreightContract(&'static str),
    InvalidTerminal(&'static str),
    DuplicateTerminal(TerminalId),
    InsufficientTerminalCapacity(TerminalId),
    UnknownTerminal(TerminalId),
    NoTerminalInRegion(RegionId),
    DuplicateFreightContract(ContractId),
    UnknownFreightContract(ContractId),
    InsufficientContractCapacity(ContractId),
    InsufficientSpotCapacity(RouteId),
    DuplicateLogisticsRoute(RouteId),
    DuplicateShipment(ShipmentId),
    UnknownShipment(ShipmentId),
    InsufficientFirmInventory {
        firm: FirmId,
        good: GoodId,
    },
    NoFeasibleLogisticsRoute(ShipmentId),
    UnknownLogisticsRoute(RouteId),
    InsufficientRouteCapacity(RouteId),
    MissingBoardMandate(ResolutionId),
    BoardResolutionNotApproved(ResolutionId),
    BoardResolutionAlreadyExecuted(ResolutionId),
    InvalidBoardExecution(&'static str),
    InsufficientFirmCash(FirmId),
    UnauthorizedBoardAction(ActorId),
    InvalidBoardVote(&'static str),
    UnauthorizedFirmControl {
        actor: ActorId,
        firm: FirmId,
    },
    PopulationAccounting {
        region: RegionId,
        region_population: Population,
        cohort_population: Population,
    },
    ArithmeticOverflow(&'static str),
    Time(TimeError),
}

impl fmt::Display for WorldError {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCountry(id) => write!(formatter, "country {id} already exists"),
            Self::DuplicateGood(id) => write!(formatter, "good {id} already exists"),
            Self::DuplicateFirm(id) => write!(formatter, "firm {id} already exists"),
            Self::DuplicateFirmAppointment => {
                formatter.write_str("duplicate corporate appointment")
            }
            Self::DuplicateInvestmentProject(id) => {
                write!(formatter, "investment project {id} already exists")
            }
            Self::DuplicateBoardResolution(id) => {
                write!(formatter, "board resolution {id} already exists")
            }
            Self::DuplicateOwnershipStake { firm, owner } => write!(
                formatter,
                "actor {owner} already has an ownership stake in firm {firm}"
            ),
            Self::OwnershipExceedsFull(firm) => {
                write!(formatter, "ownership rights exceed 100% for firm {firm}")
            }
            Self::DuplicateRecipe(id) => write!(formatter, "production recipe {id} already exists"),
            Self::DuplicateNeedProfile(id) => write!(formatter, "need profile {id} already exists"),
            Self::DuplicateCohort(id) => write!(formatter, "cohort {id} already exists"),
            Self::DuplicateRegion(id) => write!(formatter, "region {id} already exists"),
            Self::DuplicateActor(id) => write!(formatter, "actor {id} already exists"),
            Self::DuplicatePowerNode(id) => write!(formatter, "power node {id} already exists"),
            Self::DuplicateInfluence { actor, node } => {
                write!(
                    formatter,
                    "influence from actor {actor} to node {node} already exists"
                )
            }
            Self::UnknownCountry(id) => write!(formatter, "unknown country {id}"),
            Self::UnknownGood(id) => write!(formatter, "unknown good {id}"),
            Self::UnknownFirm(id) => write!(formatter, "unknown firm {id}"),
            Self::UnknownRecipe(id) => write!(formatter, "unknown production recipe {id}"),
            Self::UnknownNeedProfile(id) => write!(formatter, "unknown need profile {id}"),
            Self::MissingRegionalPrice { region, good } => write!(
                formatter,
                "missing price for good {good} in region {region}"
            ),
            Self::UnknownRegion(id) => write!(formatter, "unknown region {id}"),
            Self::UnknownActor(id) => write!(formatter, "unknown actor {id}"),
            Self::UnknownPowerNode(id) => write!(formatter, "unknown power node {id}"),
            Self::EmptyName(kind) => write!(formatter, "{kind} name cannot be empty"),
            Self::InvalidCohort(reason) => write!(formatter, "invalid household cohort: {reason}"),
            Self::InvalidConsumptionProfile(reason) => {
                write!(formatter, "invalid consumption profile: {reason}")
            }
            Self::UnknownCohort(id) => write!(formatter, "unknown cohort {id}"),
            Self::InsufficientHouseholdCash(id) => {
                write!(formatter, "cohort {id} has insufficient cash")
            }
            Self::InvalidPrice => formatter.write_str("regional price must be positive"),
            Self::InvalidProduction(reason) => {
                write!(formatter, "invalid production model: {reason}")
            }
            Self::InvalidBusinessPolicy(reason) => {
                write!(formatter, "invalid business policy: {reason}")
            }
            Self::MissingFirmPolicy(firm) => write!(formatter, "firm {firm} has no policy"),
            Self::UnknownBoardResolution(id) => write!(formatter, "unknown board resolution {id}"),
            Self::UnknownInvestmentProject(id) => {
                write!(formatter, "unknown investment project {id}")
            }
            Self::InsufficientCommittedInvestment(firm) => {
                write!(formatter, "firm {firm} lacks committed investment funds")
            }
            Self::InvalidInvestmentProject(reason) => {
                write!(formatter, "invalid investment project: {reason}")
            }
            Self::InvalidLogistics(reason) => write!(formatter, "invalid logistics: {reason}"),
            Self::InvalidTerminal(reason) => write!(formatter, "invalid terminal: {reason}"),
            Self::DuplicateTerminal(id) => write!(formatter, "terminal {id} already exists"),
            Self::UnknownTerminal(id) => write!(formatter, "unknown terminal {id}"),
            Self::NoTerminalInRegion(id) => {
                write!(formatter, "region {id} has no logistics terminal")
            }
            Self::InsufficientTerminalCapacity(id) => {
                write!(formatter, "terminal {id} has insufficient capacity")
            }
            Self::InvalidFreightContract(reason) => {
                write!(formatter, "invalid freight contract: {reason}")
            }
            Self::DuplicateFreightContract(id) => {
                write!(formatter, "freight contract {id} already exists")
            }
            Self::UnknownFreightContract(id) => write!(formatter, "unknown freight contract {id}"),
            Self::InsufficientContractCapacity(id) => write!(
                formatter,
                "freight contract {id} has insufficient reserved capacity"
            ),
            Self::InsufficientSpotCapacity(id) => {
                write!(formatter, "route {id} has insufficient spot capacity")
            }
            Self::DuplicateLogisticsRoute(id) => {
                write!(formatter, "logistics route {id} already exists")
            }
            Self::DuplicateShipment(id) => write!(formatter, "shipment {id} already exists"),
            Self::UnknownShipment(id) => write!(formatter, "unknown shipment {id}"),
            Self::InsufficientFirmInventory { firm, good } => write!(
                formatter,
                "firm {firm} has insufficient inventory of good {good}"
            ),
            Self::UnknownLogisticsRoute(id) => write!(formatter, "unknown logistics route {id}"),
            Self::InsufficientRouteCapacity(id) => {
                write!(formatter, "route {id} has insufficient capacity")
            }
            Self::NoFeasibleLogisticsRoute(id) => {
                write!(formatter, "shipment {id} has no feasible route")
            }
            Self::MissingBoardMandate(id) => {
                write!(formatter, "board resolution {id} has no executable mandate")
            }
            Self::BoardResolutionNotApproved(id) => {
                write!(formatter, "board resolution {id} is not approved")
            }
            Self::BoardResolutionAlreadyExecuted(id) => {
                write!(formatter, "board resolution {id} was already executed")
            }
            Self::InvalidBoardExecution(reason) => {
                write!(formatter, "invalid board execution: {reason}")
            }
            Self::InsufficientFirmCash(firm) => {
                write!(formatter, "firm {firm} has insufficient cash")
            }
            Self::UnauthorizedBoardAction(actor) => {
                write!(formatter, "actor {actor} cannot act on this board")
            }
            Self::InvalidBoardVote(reason) => write!(formatter, "invalid board vote: {reason}"),
            Self::UnauthorizedFirmControl { actor, firm } => {
                write!(formatter, "actor {actor} cannot control firm {firm}")
            }
            Self::PopulationAccounting {
                region,
                region_population,
                cohort_population,
            } => write!(
                formatter,
                "region {region} population {} differs from cohort total {}",
                region_population.people(),
                cohort_population.people()
            ),
            Self::ArithmeticOverflow(operation) => {
                write!(formatter, "arithmetic overflow during {operation}")
            }
            Self::Time(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<TimeError> for WorldError {
    fn from(value: TimeError) -> Self {
        Self::Time(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct World {
    pub(crate) seed: WorldSeed,
    pub(crate) date: SimDate,
    pub(crate) goods: BTreeMap<GoodId, Good>,
    pub(crate) production_recipes: BTreeMap<RecipeId, ProductionRecipe>,
    pub(crate) firms: BTreeMap<FirmId, Firm>,
    pub(crate) ownership_stakes: BTreeMap<(FirmId, ActorId), OwnershipStake>,
    pub(crate) firm_policies: BTreeMap<FirmId, FirmPolicy>,
    pub(crate) firm_appointments: BTreeMap<(FirmId, ActorId, CorporateRole), FirmAppointment>,
    pub(crate) board_resolutions: BTreeMap<ResolutionId, BoardResolution>,
    pub(crate) actor_cash: BTreeMap<ActorId, Money>,
    pub(crate) committed_investments: BTreeMap<FirmId, Money>,
    pub(crate) investment_projects: BTreeMap<ProjectId, InvestmentProject>,
    pub(crate) logistics_routes: BTreeMap<RouteId, LogisticsRoute>,
    pub(crate) route_capacity: RouteCapacityLedger,
    pub(crate) freight_capacity: FreightCapacityLedger,
    pub(crate) inventory_shipments: BTreeMap<ShipmentId, InventoryShipment>,
    pub(crate) terminals: BTreeMap<TerminalId, LogisticsTerminal>,
    pub(crate) terminal_capacity: TerminalCapacityLedger,
    pub(crate) terminal_queue: TerminalQueue,
    pub(crate) freight_contracts: BTreeMap<ContractId, FreightContract>,
    pub(crate) route_operating_costs: BTreeMap<RouteId, RouteOperatingCost>,
    pub(crate) consumption_profiles: BTreeMap<NeedProfileId, ConsumptionProfile>,
    pub(crate) regional_prices: BTreeMap<(RegionId, GoodId), Money>,
    pub(crate) countries: BTreeMap<CountryId, Country>,
    pub(crate) regions: BTreeMap<RegionId, Region>,
    pub(crate) cohorts: BTreeMap<CohortId, HouseholdCohort>,
    actors: BTreeMap<ActorId, Actor>,
    power_nodes: BTreeMap<PowerNodeId, PowerNode>,
    influences: BTreeMap<(ActorId, PowerNodeId), Influence>,
    pub(crate) events: EventLog,
}

impl World {
    /// Creates an empty world and records its founding event.
    #[must_use]
    pub fn new(seed: WorldSeed, start_date: SimDate) -> Self {
        let mut events = EventLog::default();
        events.append(start_date, DomainEvent::WorldFounded { seed: seed.get() });
        Self {
            seed,
            date: start_date,
            goods: BTreeMap::new(),
            production_recipes: BTreeMap::new(),
            firms: BTreeMap::new(),
            ownership_stakes: BTreeMap::new(),
            firm_policies: BTreeMap::new(),
            firm_appointments: BTreeMap::new(),
            board_resolutions: BTreeMap::new(),
            actor_cash: BTreeMap::new(),
            committed_investments: BTreeMap::new(),
            investment_projects: BTreeMap::new(),
            logistics_routes: BTreeMap::new(),
            route_capacity: RouteCapacityLedger::default(),
            freight_capacity: FreightCapacityLedger::default(),
            inventory_shipments: BTreeMap::new(),
            terminals: BTreeMap::new(),
            terminal_capacity: TerminalCapacityLedger::default(),
            terminal_queue: TerminalQueue::default(),
            freight_contracts: BTreeMap::new(),
            route_operating_costs: BTreeMap::new(),
            consumption_profiles: BTreeMap::new(),
            regional_prices: BTreeMap::new(),
            countries: BTreeMap::new(),
            regions: BTreeMap::new(),
            cohorts: BTreeMap::new(),
            actors: BTreeMap::new(),
            power_nodes: BTreeMap::new(),
            influences: BTreeMap::new(),
            events,
        }
    }

    /// Adds a country and records the successful registration.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::DuplicateCountry`] when the ID is already registered.
    pub fn register_country(&mut self, country: Country) -> Result<(), WorldError> {
        if self.countries.contains_key(&country.id()) {
            return Err(WorldError::DuplicateCountry(country.id()));
        }
        self.events.append(
            self.date,
            DomainEvent::CountryRegistered {
                country: country.id(),
                name: country.name().to_owned(),
            },
        );
        self.countries.insert(country.id(), country);
        Ok(())
    }

    /// Adds a region after validating its country reference.
    ///
    /// # Errors
    ///
    /// Returns a duplicate or unknown-reference [`WorldError`] when validation fails.
    pub fn register_region(&mut self, region: Region) -> Result<(), WorldError> {
        if self.regions.contains_key(&region.id()) {
            return Err(WorldError::DuplicateRegion(region.id()));
        }
        if !self.countries.contains_key(&region.country()) {
            return Err(WorldError::UnknownCountry(region.country()));
        }
        self.events.append(
            self.date,
            DomainEvent::RegionRegistered {
                region: region.id(),
                country: region.country(),
                name: region.name().to_owned(),
            },
        );
        self.regions.insert(region.id(), region);
        Ok(())
    }

    /// Adds a named actor after validating the home region.
    ///
    /// # Errors
    ///
    /// Returns a duplicate or unknown-reference [`WorldError`] when validation fails.
    pub fn register_actor(&mut self, actor: Actor) -> Result<(), WorldError> {
        if self.actors.contains_key(&actor.id()) {
            return Err(WorldError::DuplicateActor(actor.id()));
        }
        if !self.regions.contains_key(&actor.home_region()) {
            return Err(WorldError::UnknownRegion(actor.home_region()));
        }
        self.events.append(
            self.date,
            DomainEvent::ActorRegistered {
                actor: actor.id(),
                home_region: actor.home_region(),
                name: actor.name().to_owned(),
            },
        );
        self.actors.insert(actor.id(), actor);
        Ok(())
    }

    /// Adds a power node after validating country and optional holder references.
    ///
    /// # Errors
    ///
    /// Returns a duplicate or unknown-reference [`WorldError`] when validation fails.
    pub fn register_power_node(&mut self, node: PowerNode) -> Result<(), WorldError> {
        if self.power_nodes.contains_key(&node.id()) {
            return Err(WorldError::DuplicatePowerNode(node.id()));
        }
        if !self.countries.contains_key(&node.country()) {
            return Err(WorldError::UnknownCountry(node.country()));
        }
        if let Some(holder) = node.holder() {
            if !self.actors.contains_key(&holder) {
                return Err(WorldError::UnknownActor(holder));
            }
        }
        self.events.append(
            self.date,
            DomainEvent::PowerNodeRegistered {
                node: node.id(),
                country: node.country(),
                name: node.name().to_owned(),
            },
        );
        self.power_nodes.insert(node.id(), node);
        Ok(())
    }

    /// Adds one weighted actor-to-node relationship.
    ///
    /// # Errors
    ///
    /// Returns a duplicate or unknown-reference [`WorldError`] when validation fails.
    pub fn establish_influence(&mut self, influence: Influence) -> Result<(), WorldError> {
        let key = (influence.actor(), influence.node());
        if self.influences.contains_key(&key) {
            return Err(WorldError::DuplicateInfluence {
                actor: influence.actor(),
                node: influence.node(),
            });
        }
        if !self.actors.contains_key(&influence.actor()) {
            return Err(WorldError::UnknownActor(influence.actor()));
        }
        if !self.power_nodes.contains_key(&influence.node()) {
            return Err(WorldError::UnknownPowerNode(influence.node()));
        }
        self.events.append(
            self.date,
            DomainEvent::InfluenceEstablished {
                actor: influence.actor(),
                node: influence.node(),
                weight: influence.weight(),
            },
        );
        self.influences.insert(key, influence);
        Ok(())
    }

    #[must_use]
    pub const fn seed(&self) -> WorldSeed {
        self.seed
    }

    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }

    #[must_use]
    pub fn countries(&self) -> &BTreeMap<CountryId, Country> {
        &self.countries
    }

    #[must_use]
    pub fn regions(&self) -> &BTreeMap<RegionId, Region> {
        &self.regions
    }

    #[must_use]
    pub fn actors(&self) -> &BTreeMap<ActorId, Actor> {
        &self.actors
    }

    #[must_use]
    pub fn power_nodes(&self) -> &BTreeMap<PowerNodeId, PowerNode> {
        &self.power_nodes
    }

    #[must_use]
    pub fn influences(&self) -> &BTreeMap<(ActorId, PowerNodeId), Influence> {
        &self.influences
    }

    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.events
    }

    fn write_terminal_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.terminals.len() as u64);
        for (id, terminal) in &self.terminals {
            hash.write_u32(id.get());
            hash.write_u32(terminal.region().get());
            hash.write_u32(terminal.operator().get());
            hash.write_u64(terminal.capacity().get());
            hash.write_i64(terminal.storage_cost().minor_units());
            hash.write_u16(terminal.handling_days());
        }
        hash.write_u64(self.terminal_capacity.used().len() as u64);
        for (id, q) in self.terminal_capacity.used() {
            hash.write_u32(id.get());
            hash.write_u64(q.get());
        }
        hash.write_u64(self.terminal_queue.waiting().len() as u64);
        for (id, entries) in self.terminal_queue.waiting() {
            hash.write_u32(id.get());
            hash.write_u64(entries.len() as u64);
            for entry in entries {
                hash.write_u32(entry.shipment().get());
                hash.write_u64(entry.quantity().get());
            }
        }
    }

    fn write_freight_contract_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.freight_contracts.len() as u64);
        for (id, c) in &self.freight_contracts {
            hash.write_u32(id.get());
            hash.write_u32(c.shipper().get());
            hash.write_u32(c.carrier().get());
            hash.write_u32(c.route().get());
            hash.write_u64(c.reserved_capacity().get());
            hash.write_u16(c.discount().get());
            hash.write_u32(c.start_month());
            hash.write_u32(c.end_month());
            hash.write_u8(match c.status() {
                crate::ContractStatus::Proposed => 1,
                crate::ContractStatus::Active => 2,
                crate::ContractStatus::Expired => 3,
                crate::ContractStatus::Terminated => 4,
            });
        }
        hash.write_u64(self.route_operating_costs.len() as u64);
        for (route, cost) in &self.route_operating_costs {
            hash.write_u32(route.get());
            hash.write_i64(cost.total_per_unit().minor_units());
        }
        hash.write_u64(self.freight_capacity.contract_used().len() as u64);
        for (id, q) in self.freight_capacity.contract_used() {
            hash.write_u32(id.get());
            hash.write_u64(q.get());
        }
        hash.write_u64(self.freight_capacity.spot_used().len() as u64);
        for (id, q) in self.freight_capacity.spot_used() {
            hash.write_u32(id.get());
            hash.write_u64(q.get());
        }
    }

    fn write_logistics_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.logistics_routes.len() as u64);
        for (id, route) in &self.logistics_routes {
            hash.write_u32(id.get());
            hash.write_u32(route.origin().get());
            hash.write_u32(route.destination().get());
            hash.write_u8(match route.mode() {
                crate::TransportMode::Road => 1,
                crate::TransportMode::Rail => 2,
                crate::TransportMode::Sea => 3,
                crate::TransportMode::Air => 4,
            });
            hash.write_u64(route.capacity().get());
            hash.write_i64(route.cost_per_unit().minor_units());
            hash.write_u16(route.transit_days());
            hash.write_u16(route.reliability_bps());
            hash.write_u32(route.carrier().map_or(0, FirmId::get));
        }
        hash.write_u64(self.route_capacity.reserved().len() as u64);
        for (id, quantity) in self.route_capacity.reserved() {
            hash.write_u32(id.get());
            hash.write_u64(quantity.get());
        }
        hash.write_u64(self.inventory_shipments.len() as u64);
        for (id, shipment) in &self.inventory_shipments {
            hash.write_u32(id.get());
            hash.write_u32(shipment.good().get());
            hash.write_u32(shipment.source().get());
            hash.write_u32(shipment.destination().get());
            hash.write_u64(shipment.quantity().get());
            hash.write_i64(shipment.total_cost().minor_units());
            hash.write_u32(shipment.remaining_days());
            hash.write_u8(match shipment.status() {
                crate::ShipmentStatus::Planned => 1,
                crate::ShipmentStatus::Reserved => 2,
                crate::ShipmentStatus::InTransit => 3,
                crate::ShipmentStatus::Delivered => 4,
                crate::ShipmentStatus::Cancelled => 5,
            });
            hash.write_u64(shipment.routes().len() as u64);
            for (route, contract) in shipment.routes().iter().zip(shipment.capacity_contracts()) {
                hash.write_u32(route.get());
                hash.write_u32(contract.map_or(0, ContractId::get));
            }
            hash.write_u8(match shipment.phase() {
                crate::IntermodalPhase::Transit => 1,
                crate::IntermodalPhase::WaitingForTerminal => 2,
                crate::IntermodalPhase::TerminalHandling => 3,
                crate::IntermodalPhase::Delivered => 4,
            });
            hash.write_u64(shipment.terminal_ids().len() as u64);
            for terminal in shipment.terminal_ids() {
                hash.write_u32(terminal.get());
            }
        }
    }

    fn write_investment_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.investment_projects.len() as u64);
        for (id, project) in &self.investment_projects {
            hash.write_u32(id.get());
            hash.write_u32(project.firm().get());
            hash.write_u32(project.region().get());
            hash.write_i64(project.budget().minor_units());
            hash.write_i64(project.spent().minor_units());
            hash.write_u32(project.duration_months());
            hash.write_u32(project.elapsed_months());
            hash.write_u64(project.capacity_batches());
            hash.write_u8(match project.status() {
                crate::InvestmentStatus::Planned => 1,
                crate::InvestmentStatus::Building => 2,
                crate::InvestmentStatus::Completed => 3,
                crate::InvestmentStatus::Cancelled => 4,
            });
        }
    }

    fn write_board_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.board_resolutions.len() as u64);
        for (id, resolution) in &self.board_resolutions {
            hash.write_u32(id.get());
            hash.write_u32(resolution.firm().get());
            hash.write_u32(resolution.proposer().get());
            hash.write_u8(match resolution.action() {
                CorporateAction::SetOverallPolicy => 1,
                CorporateAction::SetOperationsPolicy => 2,
                CorporateAction::SetMarketingPolicy => 3,
                CorporateAction::ProposeMajorInvestment => 4,
                CorporateAction::ApproveMajorInvestment => 5,
                CorporateAction::DeclareDividend => 6,
                CorporateAction::AppointExecutive => 7,
            });
            hash.write_u8(match resolution.status() {
                ResolutionStatus::Open => 1,
                ResolutionStatus::Approved => 2,
                ResolutionStatus::Rejected => 3,
            });
            hash.write_u8(u8::from(resolution.executed()));
            match resolution.mandate() {
                None => hash.write_u8(0),
                Some(crate::BoardMandate::AppointExecutive { actor, role }) => {
                    hash.write_u8(1);
                    hash.write_u32(actor.get());
                    hash.write_u8(match role {
                        CorporateRole::BoardDirector => 1,
                        CorporateRole::ChiefExecutive => 2,
                        CorporateRole::OperationsManager => 3,
                        CorporateRole::MarketingManager => 4,
                    });
                }
                Some(crate::BoardMandate::RemoveExecutive { actor, role }) => {
                    hash.write_u8(2);
                    hash.write_u32(actor.get());
                    hash.write_u8(match role {
                        CorporateRole::BoardDirector => 1,
                        CorporateRole::ChiefExecutive => 2,
                        CorporateRole::OperationsManager => 3,
                        CorporateRole::MarketingManager => 4,
                    });
                }
                Some(crate::BoardMandate::DeclareDividend { amount }) => {
                    hash.write_u8(3);
                    hash.write_i64(amount.minor_units());
                }
                Some(crate::BoardMandate::CommitInvestment { amount }) => {
                    hash.write_u8(4);
                    hash.write_i64(amount.minor_units());
                }
            }
            hash.write_u64(resolution.votes().len() as u64);
            for (actor, vote) in resolution.votes() {
                hash.write_u32(actor.get());
                hash.write_u8(match vote {
                    BoardVote::For => 1,
                    BoardVote::Against => 2,
                    BoardVote::Abstain => 3,
                });
            }
        }
    }

    fn write_business_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.ownership_stakes.len() as u64);
        for ((firm, owner), stake) in &self.ownership_stakes {
            hash.write_u32(firm.get());
            hash.write_u32(owner.get());
            hash.write_u16(stake.economic_rights().get());
            hash.write_u16(stake.voting_rights().get());
        }
        hash.write_u64(self.firm_policies.len() as u64);
        for (firm, policy) in &self.firm_policies {
            hash.write_u32(firm.get());
            hash.write_u16(policy.inventory_buffer_days());
            hash.write_u16(policy.price_markup().get());
            hash.write_u16(policy.marketing_budget().get());
            hash.write_u16(policy.reinvestment().get());
            hash.write_u16(policy.dividend().get());
        }
        hash.write_u64(self.firm_appointments.len() as u64);
        for (firm, actor, role) in self.firm_appointments.keys() {
            hash.write_u32(firm.get());
            hash.write_u32(actor.get());
            hash.write_u8(match role {
                CorporateRole::BoardDirector => 1,
                CorporateRole::ChiefExecutive => 2,
                CorporateRole::OperationsManager => 3,
                CorporateRole::MarketingManager => 4,
            });
        }
    }

    fn write_production_fingerprint(&self, hash: &mut StableHasher) {
        hash.write_u64(self.production_recipes.len() as u64);
        for (id, recipe) in &self.production_recipes {
            hash.write_u32(id.get());
            hash.write_str(recipe.name());
            hash.write_u32(recipe.output_good().get());
            hash.write_u64(recipe.output_per_batch().get());
            hash.write_u64(recipe.labor_milli_worker_months());
            hash.write_u64(recipe.inputs().len() as u64);
            for input in recipe.inputs() {
                hash.write_u32(input.good().get());
                hash.write_u64(input.quantity_per_batch().get());
            }
        }
        hash.write_u64(self.firms.len() as u64);
        for (id, firm) in &self.firms {
            hash.write_u32(id.get());
            hash.write_str(firm.name());
            hash.write_u32(firm.region().get());
            hash.write_u32(firm.recipe().get());
            hash.write_u64(firm.workers());
            hash.write_u64(firm.capacity_batches());
            hash.write_i64(firm.cash().minor_units());
            hash.write_u64(firm.inventories().len() as u64);
            for (good, quantity) in firm.inventories() {
                hash.write_u32(good.get());
                hash.write_u64(quantity.get());
            }
        }
    }

    #[must_use]
    pub fn stable_fingerprint(&self) -> u64 {
        let mut hash = StableHasher::new();
        hash.write_u32(crate::SIMULATION_VERSION);
        hash.write_u64(self.seed.get());
        hash.write_i32(self.date.year());
        hash.write_u16(self.date.day_of_year());
        hash.write_u64(self.goods.len() as u64);
        for (id, good) in &self.goods {
            hash.write_u32(id.get());
            hash.write_str(good.name());
        }
        self.write_production_fingerprint(&mut hash);
        self.write_business_fingerprint(&mut hash);
        self.write_board_fingerprint(&mut hash);
        self.write_investment_fingerprint(&mut hash);
        self.write_logistics_fingerprint(&mut hash);
        self.write_freight_contract_fingerprint(&mut hash);
        self.write_terminal_fingerprint(&mut hash);
        hash.write_u64(self.actor_cash.len() as u64);
        for (actor, cash) in &self.actor_cash {
            hash.write_u32(actor.get());
            hash.write_i64(cash.minor_units());
        }
        hash.write_u64(self.committed_investments.len() as u64);
        for (firm, amount) in &self.committed_investments {
            hash.write_u32(firm.get());
            hash.write_i64(amount.minor_units());
        }
        hash.write_u64(self.consumption_profiles.len() as u64);
        for (id, profile) in &self.consumption_profiles {
            hash.write_u32(id.get());
            hash.write_str(profile.name());
            hash.write_u64(profile.targets().len() as u64);
            for target in profile.targets() {
                hash.write_u32(target.good().get());
                hash.write_u8(target.tier().fingerprint_tag());
                hash.write_u8(target.basis().fingerprint_tag());
                hash.write_u64(target.monthly_quantity().get());
            }
        }
        hash.write_u64(self.regional_prices.len() as u64);
        for ((region, good), price) in &self.regional_prices {
            hash.write_u32(region.get());
            hash.write_u32(good.get());
            hash.write_i64(price.minor_units());
        }
        hash.write_u64(self.countries.len() as u64);
        for (id, country) in &self.countries {
            hash.write_u32(id.get());
            hash.write_str(country.name());
            let indicators = country.indicators();
            hash.write_i64(indicators.treasury().minor_units());
            hash.write_i64(indicators.public_debt().minor_units());
            hash.write_u16(indicators.legitimacy().get());
            hash.write_u16(indicators.elite_cohesion().get());
        }
        hash.write_u64(self.regions.len() as u64);
        for (id, region) in &self.regions {
            hash.write_u32(id.get());
            hash.write_u32(region.country().get());
            hash.write_str(region.name());
            hash.write_u64(region.population().people());
            hash.write_i64(region.annual_output().minor_units());
        }
        hash.write_u64(self.cohorts.len() as u64);
        for (id, cohort) in &self.cohorts {
            hash.write_u32(id.get());
            hash.write_u32(cohort.region().get());
            hash.write_u32(cohort.need_profile().get());
            hash.write_u64(cohort.people().people());
            hash.write_u64(cohort.households());
            hash.write_u8(cohort.age_band().fingerprint_tag());
            hash.write_u8(cohort.household_type().fingerprint_tag());
            hash.write_u8(cohort.education().fingerprint_tag());
            hash.write_u8(cohort.employment().fingerprint_tag());
            hash.write_i64(cohort.annual_income().minor_units());
            hash.write_i64(cohort.liquid_wealth().minor_units());
            hash.write_i64(cohort.debt().minor_units());
        }
        hash.write_u64(self.actors.len() as u64);
        for (id, actor) in &self.actors {
            hash.write_u32(id.get());
            hash.write_str(actor.name());
            hash.write_u32(actor.home_region().get());
            hash.write_i32(actor.born_year());
        }
        hash.write_u64(self.power_nodes.len() as u64);
        for (id, node) in &self.power_nodes {
            hash.write_u32(id.get());
            hash.write_u32(node.country().get());
            hash.write_str(node.name());
            hash.write_u8(node.kind().fingerprint_tag());
            hash.write_u32(node.holder().map_or(0, ActorId::get));
        }
        hash.write_u64(self.influences.len() as u64);
        for ((actor, node), influence) in &self.influences {
            hash.write_u32(actor.get());
            hash.write_u32(node.get());
            hash.write_u16(influence.weight().get());
        }
        hash.finish()
    }
}

fn validated_name(kind: &'static str, name: String) -> Result<String, WorldError> {
    if name.trim().is_empty() {
        Err(WorldError::EmptyName(kind))
    } else {
        Ok(name)
    }
}

#[derive(Clone, Copy, Debug)]
struct StableHasher(u64);

impl StableHasher {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.write(&value.to_le_bytes());
    }

    fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    fn write_i32(&mut self, value: i32) {
        self.write(&value.to_le_bytes());
    }

    fn write_i64(&mut self, value: i64) {
        self.write(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write(value.as_bytes());
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn country(id: u32, name: &str) -> Country {
        Country::new(CountryId::new(id), name).expect("valid country")
    }

    #[test]
    fn invalid_reference_does_not_emit_an_event() {
        let mut world = World::new(
            WorldSeed::new(1),
            SimDate::new(2025, 1).expect("valid date"),
        );
        let before = world.events().len();
        let region = Region::new(
            RegionId::new(1),
            CountryId::new(99),
            "Nowhere",
            Population::new(1),
            Money::from_minor_units(1),
        )
        .expect("valid region values");
        assert_eq!(
            world.register_region(region),
            Err(WorldError::UnknownCountry(CountryId::new(99)))
        );
        assert_eq!(world.events().len(), before);
    }

    #[test]
    fn canonical_storage_ignores_country_insertion_order() {
        let build = |reverse: bool| {
            let mut world = World::new(
                WorldSeed::new(1),
                SimDate::new(2025, 1).expect("valid date"),
            );
            let countries = if reverse {
                [(2, "B"), (1, "A")]
            } else {
                [(1, "A"), (2, "B")]
            };
            for (id, name) in countries {
                world
                    .register_country(country(id, name))
                    .expect("unique country");
            }
            world
        };
        assert_eq!(
            build(false).stable_fingerprint(),
            build(true).stable_fingerprint()
        );
    }

    #[test]
    fn complete_power_graph_is_validated() {
        let mut world = World::new(
            WorldSeed::new(1),
            SimDate::new(2025, 1).expect("valid date"),
        );
        world.register_country(country(1, "A")).expect("country");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Capital",
                    Population::new(100),
                    Money::from_minor_units(1_000),
                )
                .expect("region"),
            )
            .expect("region reference");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Ada", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor reference");
        world
            .register_power_node(
                PowerNode::new(
                    PowerNodeId::new(1),
                    CountryId::new(1),
                    "Presidency",
                    PowerNodeKind::PoliticalOffice,
                    Some(ActorId::new(1)),
                )
                .expect("node"),
            )
            .expect("node references");
        world
            .establish_influence(Influence::new(
                ActorId::new(1),
                PowerNodeId::new(1),
                BasisPoints::new(7_500).expect("weight"),
            ))
            .expect("influence references");
        assert_eq!(world.regions().len(), 1);
        assert_eq!(world.actors().len(), 1);
        assert_eq!(world.power_nodes().len(), 1);
        assert_eq!(world.influences().len(), 1);
    }
}
