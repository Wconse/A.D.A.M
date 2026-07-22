#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod board;
pub mod business;
pub mod cohort;
pub mod command;
pub mod demand;
pub mod event;
pub mod freight;
pub mod ids;
pub mod investment;
pub mod logistics;
pub mod production;
pub mod rng;
mod simulation;
pub mod terminal;
pub mod time;
pub mod value;
pub mod world;
pub mod world_logistics;

pub use board::{BoardMandate, BoardResolution, BoardVote, ResolutionStatus};
pub use business::{CorporateAction, CorporateRole, FirmAppointment, FirmPolicy, OwnershipStake};
pub use cohort::{AgeBand, EducationLevel, EmploymentStatus, HouseholdCohort, HouseholdType};
pub use command::{WorldCommand, replay_commands};
pub use demand::{
    ConsumptionProfile, ConsumptionTarget, DemandBasis, DemandIntent, Good, NeedTier,
};
pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use freight::{
    ContractStatus, FreightCapacityLedger, FreightContract, FreightEconomics,
    MonthlyFreightCapacityLedger, RouteOperatingCost, evaluate_freight_economics,
};
pub use ids::{
    ActorId, CohortId, ContractId, CountryId, FirmId, GoodId, NeedProfileId, PowerNodeId,
    ProjectId, RecipeId, RegionId, ResolutionId, RouteId, ShipmentId, TerminalId,
};
pub use investment::{InvestmentProject, InvestmentStatus};
pub use logistics::{
    IntermodalPhase, IntermodalShipmentLifecycle, LegShipmentLifecycle, LogisticsRoute,
    MultiLegShipmentPlan, RouteCapacityLedger, ShipmentLifecycle, ShipmentOrder, ShipmentPlan,
    ShipmentStatus, ShipmentTransition, TransportMode, plan_direct_shipment,
    plan_multileg_shipment,
};
pub use production::{Firm, ProductionInput, ProductionPlan, ProductionRecipe};
pub use rng::{RandomStream, WorldSeed};
pub use terminal::{LogisticsTerminal, TerminalCapacityLedger, TerminalQueue, TerminalQueueEntry};
pub use time::{SimDate, TimeError};
pub use value::{BasisPoints, Money, Population, QuantityMilli, RatePpm, ValueError};
pub use world::{
    Actor, Country, CountryIndicators, Influence, PowerNode, PowerNodeKind, Region, World,
    WorldError,
};
pub use world_logistics::InventoryShipment;

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 6;
