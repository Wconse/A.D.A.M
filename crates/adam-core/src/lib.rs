#![doc = "Deterministic, engine-independent simulation core for A.D.A.M."]

pub mod accounting;
pub mod board;
pub mod business;
pub mod capacity_investment;
pub mod chronicle;
pub mod cohort;
pub mod command;
pub mod commerce;
pub mod coping;
pub mod credit_market;
pub mod demand;
pub mod distress;
pub mod event;
pub mod firm_entry;
pub mod freight;
pub mod government_program;
pub mod health;
pub mod housing;
pub mod ids;
pub mod insolvency;
pub mod investment;
pub mod labor;
pub mod logistics;
pub mod management;
pub mod market;
pub mod migration;
pub mod observation;
pub mod political_economy;
pub mod pricing;
pub mod procurement;
pub mod production;
pub mod public_reserve;
pub mod public_services;
pub mod rationing;
pub mod regional_interests;
pub mod relief;
pub mod rng;
pub mod route_investment;
pub mod service_allocation;
mod simulation;
pub mod social;
pub mod terminal;
pub mod time;
pub mod value;
pub mod world;
pub mod world_logistics;

pub use accounting::{
    EmploymentAdjustmentProposal, FirmExpectationSource, FirmExpectations, FirmMonthlyAccounts,
};
pub use board::{BoardMandate, BoardResolution, BoardVote, ResolutionStatus};
pub use business::{CorporateAction, CorporateRole, FirmAppointment, FirmPolicy, OwnershipStake};
pub use capacity_investment::CapacityInvestmentDecision;
pub use chronicle::ChronicleEntry;
pub use cohort::{
    AgeBand, EducationLevel, EmploymentStatus, HouseholdCashflow, HouseholdCohort, HouseholdType,
};
pub use command::{WorldCommand, replay_commands};
pub use commerce::{MonthlyCommercialCycleResult, MonthlyEconomicCycleResult};
pub use coping::HouseholdSurvivalBorrowing;
pub use credit_market::{AutonomousFirmCreditDecision, BorrowerCreditHistory, LenderCreditHistory};
pub use demand::{
    ConsumptionProfile, ConsumptionTarget, DemandBasis, DemandIntent, Good, NeedTier,
};
pub use distress::{FirmDistressAction, FirmDistressDownsizing, FirmRecapitalization};
pub use event::{DomainEvent, EventEnvelope, EventLog};
pub use firm_entry::{FirmEntryDecision, FirmFoundingPlan};
pub use freight::{
    ContractStatus, FreightCapacityLedger, FreightContract, FreightEconomics,
    MonthlyFreightCapacityLedger, RouteOperatingCost, evaluate_freight_economics,
};
pub use government_program::{
    GovernmentProgram, GovernmentProgramStatus, ProgramFundingSource, ProgramPoliticalStance,
    ProgramRegionalMemory, ProgramRegionalOutcomeKind, PublicServicePriority,
};
pub use health::CohortHealth;
pub use housing::{HousingConstruction, RegionalHousingMarket};
pub use ids::{
    ActorId, CohortId, ContractId, CountryId, FirmId, GoodId, NeedProfileId, PowerNodeId,
    ProgramId, ProjectId, RecipeId, RegionId, ResolutionId, RouteId, ShipmentId, SkillId,
    TerminalId,
};
pub use insolvency::{
    FirmCreditOffer, FirmCreditSchedule, FirmCreditorClaim, FirmCreditorPriority,
    FirmDebtServicePayment, FirmInsolvency, FirmLiquidation, FirmLiquidationCapacitySale,
    FirmLiquidationInventorySale, FirmReorganization, FirmReorganizationPlan,
};
pub use investment::{InvestmentProject, InvestmentStatus};
pub use labor::{
    EmploymentAgreement, EmploymentMatch, EmploymentRetention, EmploymentSwitch, PayrollRecord,
    RegionalLaborMarketObservation, RegionalOccupationLaborMarketObservation,
    RegionalSkillLaborMarketObservation, TrainingSponsor, UnprofitableEmploymentRelease,
    WorkforceTraining,
};
pub use logistics::{
    IntermodalPhase, IntermodalShipmentLifecycle, LegShipmentLifecycle, LogisticsRoute,
    MultiLegShipmentPlan, RouteCapacityLedger, ShipmentLifecycle, ShipmentOrder, ShipmentPlan,
    ShipmentStatus, ShipmentTransition, TransportMode, plan_direct_shipment,
    plan_multileg_shipment,
};
pub use management::FirmManagementDecision;
pub use market::{
    FirmMarketOfferPlan, HouseholdImportDependence, MarketClearing, MarketFill, MarketOffer,
    MarketOfferOutcome, MarketOrder, clear_local_market, clear_market_with_delivery,
};
pub use migration::HouseholdMigration;
pub use observation::{FIRM_OBSERVATION_HISTORY_LIMIT, FirmOperatingObservation};
pub use political_economy::RegionalSocialPressure;
pub use pricing::{ObservedMarketPressure, RegionalPriceAdjustment};
pub use procurement::{FirmProcurementFill, FirmProcurementOrder, FirmProcurementResult};
pub use production::{
    Firm, ProductionAdjustmentProposal, ProductionInput, ProductionPlan, ProductionRecipe,
};
pub use public_reserve::{
    GovernmentReserveDistribution, GovernmentReserveMaintenance, GovernmentReservePolicyReview,
    GovernmentReserveProcurement, ReservePriorityRevisionReason,
};
pub use public_services::RegionalPublicServices;
pub use rationing::{SurvivalRationingAllocation, SurvivalRationingOutcome};
pub use regional_interests::{
    RegionalFiscalPosition, RegionalInterest, RegionalPolicyOutcome, RegionalPolicyPriority,
};
pub use relief::{
    EmergencyReliefPayment, EmergencyReliefStrategy, GovernmentEmergencyPolicy,
    PhysicalShortageStrategy,
};
pub use rng::{RandomStream, WorldSeed};
pub use route_investment::RouteCapacityExpansion;
pub use service_allocation::{
    CountryServiceAllocation, ServiceAllocationInfluenceKind, ServiceAllocationSource,
};
pub use simulation::EconomicYearResult;
pub use social::{CohortExperience, SocialStress, SocialStressMemory};
pub use terminal::{LogisticsTerminal, TerminalCapacityLedger, TerminalQueue, TerminalQueueEntry};
pub use time::{SimDate, TimeError};
pub use value::{BasisPoints, Money, Population, QuantityMilli, RatePpm, ValueError};
pub use world::{
    Actor, Country, CountryIndicators, Influence, PowerNode, PowerNodeKind, Region, World,
    WorldError,
};
pub use world_logistics::InventoryShipment;

/// Version of the simulation rules that participate in determinism guarantees.
pub const SIMULATION_VERSION: u32 = 103;
