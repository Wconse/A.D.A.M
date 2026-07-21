#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod event;
pub mod ids;
pub mod rng;
pub mod time;
pub mod world;

pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use ids::{ActorId, CountryId};
pub use rng::{RandomStream, WorldSeed};
pub use time::{SimDate, TimeError};
pub use world::{Country, World, WorldError};

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 1;
