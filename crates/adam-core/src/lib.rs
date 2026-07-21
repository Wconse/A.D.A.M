#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod event;
pub mod ids;
pub mod rng;
pub mod time;
pub mod value;
pub mod world;

pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use ids::{ActorId, CountryId, PowerNodeId, RegionId};
pub use rng::{RandomStream, WorldSeed};
pub use time::{SimDate, TimeError};
pub use value::{BasisPoints, Money, Population, ValueError};
pub use world::{Actor, Country, Influence, PowerNode, PowerNodeKind, Region, World, WorldError};

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 2;
