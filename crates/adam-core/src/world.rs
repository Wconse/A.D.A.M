use std::collections::BTreeMap;
use std::fmt;

use crate::{
    ActorId, BasisPoints, CohortId, ConsumptionProfile, CountryId, DomainEvent, EventLog, Good,
    GoodId, HouseholdCohort, Money, NeedProfileId, Population, PowerNodeId, RegionId, SimDate,
    TimeError, WorldSeed,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    PopulationAccounting {
        region: RegionId,
        region_population: Population,
        cohort_population: Population,
    },
    ArithmeticOverflow(&'static str),
    Time(TimeError),
}

impl fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCountry(id) => write!(formatter, "country {id} already exists"),
            Self::DuplicateGood(id) => write!(formatter, "good {id} already exists"),
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
            Self::InvalidPrice => formatter.write_str("regional price must be positive"),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    pub(crate) seed: WorldSeed,
    pub(crate) date: SimDate,
    pub(crate) goods: BTreeMap<GoodId, Good>,
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
