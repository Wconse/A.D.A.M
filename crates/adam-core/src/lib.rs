#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod business;
pub mod cohort;
pub mod demand;
pub mod event;
pub mod ids;
pub mod production;
pub mod rng;
mod simulation;
pub mod time;
pub mod value;
pub mod world;

pub use business::{FirmPolicy, OwnershipStake};
pub use cohort::{AgeBand, EducationLevel, EmploymentStatus, HouseholdCohort, HouseholdType};
pub use demand::{
    ConsumptionProfile, ConsumptionTarget, DemandBasis, DemandIntent, Good, NeedTier,
};
pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use ids::{
    ActorId, CohortId, CountryId, FirmId, GoodId, NeedProfileId, PowerNodeId, RecipeId, RegionId,
};
pub use production::{Firm, ProductionInput, ProductionPlan, ProductionRecipe};
pub use rng::{RandomStream, WorldSeed};
pub use time::{SimDate, TimeError};
pub use value::{BasisPoints, Money, Population, QuantityMilli, RatePpm, ValueError};
pub use world::{
    Actor, Country, CountryIndicators, Influence, PowerNode, PowerNodeKind, Region, World,
    WorldError,
};

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 6;
