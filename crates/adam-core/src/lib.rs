#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod cohort;
pub mod event;
pub mod ids;
pub mod rng;
mod simulation;
pub mod time;
pub mod value;
pub mod world;

pub use cohort::{AgeBand, EducationLevel, EmploymentStatus, HouseholdCohort, HouseholdType};
pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use ids::{ActorId, CohortId, CountryId, PowerNodeId, RegionId};
pub use rng::{RandomStream, WorldSeed};
pub use time::{SimDate, TimeError};
pub use value::{BasisPoints, Money, Population, RatePpm, ValueError};
pub use world::{
    Actor, Country, CountryIndicators, Influence, PowerNode, PowerNodeKind, Region, World,
    WorldError,
};

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 4;
