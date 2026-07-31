use std::collections::BTreeMap;

use crate::{
    ActorId, BasisPoints, CountryId, DomainEvent, GoodId, Money, ProgramId, QuantityMilli,
    RegionId, World, WorldError,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum PublicServicePriority {
    Healthcare,
    Infrastructure,
    Administration,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum GovernmentProgramStatus {
    Announced,
    Active,
    Completed,
    Cancelled,
    Failed,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ProgramFundingSource {
    Treasury,
    PublicDebt,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ProgramPoliticalStance {
    Support,
    Opposition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProgramPoliticalInfluence {
    actor: ActorId,
    office: crate::PowerNodeId,
    home_region: RegionId,
    stance: ProgramPoliticalStance,
    weight: BasisPoints,
    regional_share: BasisPoints,
    fair_share: BasisPoints,
    execution_modifier: i32,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ProgramRegionalOutcomeKind {
    Beneficiary,
    Underfulfilled,
    Excluded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgramRegionalMemory {
    promised: Money,
    committed: Money,
    delivered: Money,
    fulfillment: BasisPoints,
    outcome: ProgramRegionalOutcomeKind,
    years_excluded: u16,
    political_memory: i32,
}

impl ProgramRegionalMemory {
    #[must_use]
    pub const fn promised(self) -> Money {
        self.promised
    }
    #[must_use]
    pub const fn committed(self) -> Money {
        self.committed
    }
    #[must_use]
    pub const fn delivered(self) -> Money {
        self.delivered
    }
    #[must_use]
    pub const fn fulfillment(self) -> BasisPoints {
        self.fulfillment
    }
    #[must_use]
    pub const fn outcome(self) -> ProgramRegionalOutcomeKind {
        self.outcome
    }
    #[must_use]
    pub const fn years_excluded(self) -> u16 {
        self.years_excluded
    }
    #[must_use]
    pub const fn political_memory(self) -> i32 {
        self.political_memory
    }
}

/// A durable political promise. Declaration is intentionally separate from appropriation and
/// delivery: an authorized government may promise more than its current treasury can fund.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GovernmentProgram {
    id: ProgramId,
    country: CountryId,
    initiator: ActorId,
    name: String,
    regional_shares: BTreeMap<RegionId, BasisPoints>,
    material_requirements: BTreeMap<GoodId, QuantityMilli>,
    temporary_workers_required: u64,
    promised_annual_funding: Money,
    duration_years: u16,
    priority: PublicServicePriority,
    promised_improvement: BasisPoints,
    start_year: i32,
    status: GovernmentProgramStatus,
    appropriated_funding: Money,
    delivered_funding: Money,
    carryover_funding: Money,
    years_delayed: u16,
    last_appropriation_year: Option<i32>,
    last_execution_year: Option<i32>,
    regional_memory: BTreeMap<RegionId, ProgramRegionalMemory>,
}

impl GovernmentProgram {
    /// Builds an announced program without reserving or spending treasury cash.
    /// # Errors
    /// Rejects empty names, negative promises, zero duration, or shares not totaling 100%.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ProgramId,
        country: CountryId,
        initiator: ActorId,
        name: impl Into<String>,
        regional_shares: BTreeMap<RegionId, BasisPoints>,
        promised_annual_funding: Money,
        duration_years: u16,
        priority: PublicServicePriority,
        promised_improvement: BasisPoints,
        start_year: i32,
    ) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorldError::EmptyName("government program"));
        }
        if promised_annual_funding.minor_units() < 0 {
            return Err(WorldError::InvalidGovernmentProgram(
                "promised annual funding cannot be negative",
            ));
        }
        if duration_years == 0 {
            return Err(WorldError::InvalidGovernmentProgram(
                "program duration must be at least one year",
            ));
        }
        let total: u32 = regional_shares
            .values()
            .map(|share| u32::from(share.get()))
            .sum();
        if regional_shares.is_empty() || total != 10_000 {
            return Err(WorldError::InvalidGovernmentProgram(
                "regional shares must sum to exactly 10000 basis points",
            ));
        }
        Ok(Self {
            id,
            country,
            initiator,
            name,
            regional_shares,
            material_requirements: BTreeMap::new(),
            temporary_workers_required: 0,
            promised_annual_funding,
            duration_years,
            priority,
            promised_improvement,
            start_year,
            status: GovernmentProgramStatus::Announced,
            appropriated_funding: Money::from_minor_units(0),
            delivered_funding: Money::from_minor_units(0),
            carryover_funding: Money::from_minor_units(0),
            years_delayed: 0,
            last_appropriation_year: None,
            last_execution_year: None,
            regional_memory: BTreeMap::new(),
        })
    }

    #[must_use]
    pub const fn id(&self) -> ProgramId {
        self.id
    }
    #[must_use]
    pub const fn country(&self) -> CountryId {
        self.country
    }
    #[must_use]
    pub const fn initiator(&self) -> ActorId {
        self.initiator
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn regional_shares(&self) -> &BTreeMap<RegionId, BasisPoints> {
        &self.regional_shares
    }
    #[must_use]
    pub fn material_requirements(&self) -> &BTreeMap<GoodId, QuantityMilli> {
        &self.material_requirements
    }
    #[must_use]
    pub const fn temporary_workers_required(&self) -> u64 {
        self.temporary_workers_required
    }

    /// Requires temporary unemployed workers for one full annual execution.
    #[must_use]
    pub const fn with_temporary_workers(mut self, workers: u64) -> Self {
        self.temporary_workers_required = workers;
        self
    }

    /// Adds a physical annual requirement consumed from regional public reserves.
    /// # Errors
    /// Rejects a zero requirement.
    pub fn with_material_requirement(
        mut self,
        good: GoodId,
        quantity: QuantityMilli,
    ) -> Result<Self, WorldError> {
        if quantity.get() == 0 {
            return Err(WorldError::InvalidGovernmentProgram(
                "program material requirement must be positive",
            ));
        }
        self.material_requirements.insert(good, quantity);
        Ok(self)
    }
    #[must_use]
    pub const fn promised_annual_funding(&self) -> Money {
        self.promised_annual_funding
    }
    #[must_use]
    pub const fn duration_years(&self) -> u16 {
        self.duration_years
    }
    #[must_use]
    pub const fn priority(&self) -> PublicServicePriority {
        self.priority
    }
    #[must_use]
    pub const fn promised_improvement(&self) -> BasisPoints {
        self.promised_improvement
    }
    #[must_use]
    pub const fn start_year(&self) -> i32 {
        self.start_year
    }
    #[must_use]
    pub const fn status(&self) -> GovernmentProgramStatus {
        self.status
    }
    #[must_use]
    pub const fn appropriated_funding(&self) -> Money {
        self.appropriated_funding
    }
    #[must_use]
    pub const fn delivered_funding(&self) -> Money {
        self.delivered_funding
    }
    #[must_use]
    pub const fn carryover_funding(&self) -> Money {
        self.carryover_funding
    }
    #[must_use]
    pub const fn years_delayed(&self) -> u16 {
        self.years_delayed
    }
    #[must_use]
    pub const fn last_appropriation_year(&self) -> Option<i32> {
        self.last_appropriation_year
    }
    #[must_use]
    pub const fn last_execution_year(&self) -> Option<i32> {
        self.last_execution_year
    }
    #[must_use]
    pub fn regional_memory(&self) -> &BTreeMap<RegionId, ProgramRegionalMemory> {
        &self.regional_memory
    }

    fn record_appropriation(&mut self, year: i32, amount: Money) -> Result<(), WorldError> {
        let cumulative = self
            .appropriated_funding
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow(
                "government program appropriation",
            ))?;
        let carryover = self
            .carryover_funding
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow(
                "government program carryover",
            ))?;
        self.appropriated_funding = Money::from_minor_units(cumulative);
        self.carryover_funding = Money::from_minor_units(carryover);
        self.last_appropriation_year = Some(year);
        self.status = GovernmentProgramStatus::Active;
        Ok(())
    }

    fn record_execution(&mut self, year: i32, delivered: Money) -> Result<(), WorldError> {
        let delivered_total = self
            .delivered_funding
            .minor_units()
            .checked_add(delivered.minor_units())
            .ok_or(WorldError::ArithmeticOverflow(
                "government program delivery",
            ))?;
        let carryover = self
            .carryover_funding
            .minor_units()
            .checked_sub(delivered.minor_units())
            .ok_or(WorldError::ArithmeticOverflow(
                "government program execution carryover",
            ))?;
        if carryover < 0 {
            return Err(WorldError::InvalidGovernmentProgram(
                "program cannot deliver more than committed carryover",
            ));
        }
        self.delivered_funding = Money::from_minor_units(delivered_total);
        self.carryover_funding = Money::from_minor_units(carryover);
        self.last_execution_year = Some(year);
        if carryover > 0 {
            self.years_delayed = self.years_delayed.saturating_add(1);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn record_regional_memory(
        &mut self,
        region: RegionId,
        promised: Money,
        committed: Money,
        delivered: Money,
        fulfillment: BasisPoints,
        outcome: ProgramRegionalOutcomeKind,
        political_shift: i32,
    ) -> Result<ProgramRegionalMemory, WorldError> {
        let previous = self.regional_memory.get(&region).copied();
        let add = |before: Money, value: Money, context| {
            before
                .minor_units()
                .checked_add(value.minor_units())
                .map(Money::from_minor_units)
                .ok_or(WorldError::ArithmeticOverflow(context))
        };
        let memory = ProgramRegionalMemory {
            promised: add(
                previous.map_or(Money::default(), ProgramRegionalMemory::promised),
                promised,
                "program regional promise",
            )?,
            committed: add(
                previous.map_or(Money::default(), ProgramRegionalMemory::committed),
                committed,
                "program regional commitment",
            )?,
            delivered: add(
                previous.map_or(Money::default(), ProgramRegionalMemory::delivered),
                delivered,
                "program regional delivery memory",
            )?,
            fulfillment,
            outcome,
            years_excluded: if outcome == ProgramRegionalOutcomeKind::Excluded {
                previous
                    .map_or(0, ProgramRegionalMemory::years_excluded)
                    .saturating_add(1)
            } else {
                0
            },
            political_memory: previous
                .map_or(0, ProgramRegionalMemory::political_memory)
                .saturating_add(political_shift)
                .clamp(-5_000, 5_000),
        };
        self.regional_memory.insert(region, memory);
        Ok(memory)
    }

    fn cancel(&mut self) {
        self.status = GovernmentProgramStatus::Cancelled;
    }
}

impl World {
    /// Records an authorized political promise as persistent world state.
    ///
    /// This does not reserve cash. Over-promising is legal and will become consequential during
    /// appropriation and execution slices.
    /// # Errors
    /// Rejects duplicate IDs, unknown/foreign regions, past starts, or missing authority.
    pub fn declare_government_program(
        &mut self,
        program: GovernmentProgram,
    ) -> Result<(), WorldError> {
        if self.government_programs.contains_key(&program.id()) {
            return Err(WorldError::DuplicateGovernmentProgram(program.id()));
        }
        if !self.countries.contains_key(&program.country()) {
            return Err(WorldError::UnknownCountry(program.country()));
        }
        if !self.can_authorize_emergency_relief(program.initiator(), program.country()) {
            return Err(WorldError::UnauthorizedGovernmentAction {
                actor: program.initiator(),
                country: program.country(),
            });
        }
        if program.start_year() < self.date.year() {
            return Err(WorldError::InvalidGovernmentProgram(
                "program cannot start before the current simulation year",
            ));
        }
        for good in program.material_requirements().keys() {
            if !self.goods.contains_key(good) {
                return Err(WorldError::UnknownGood(*good));
            }
        }
        for region in program.regional_shares().keys() {
            let registered = self
                .regions
                .get(region)
                .ok_or(WorldError::UnknownRegion(*region))?;
            if registered.country() != program.country() {
                return Err(WorldError::InvalidGovernmentProgram(
                    "program cannot promise funding to a foreign region",
                ));
            }
        }
        self.events.append(
            self.date,
            DomainEvent::GovernmentProgramDeclared {
                program: program.id(),
                country: program.country(),
                initiator: program.initiator(),
                name: program.name().to_owned(),
                regional_shares: program.regional_shares().clone(),
                promised_annual_funding: program.promised_annual_funding(),
                duration_years: program.duration_years(),
                priority: program.priority(),
                promised_improvement: program.promised_improvement(),
                start_year: program.start_year(),
            },
        );
        self.government_programs.insert(program.id(), program);
        Ok(())
    }

    /// Commits one program appropriation for the current year.
    ///
    /// Zero is a valid explicit political decision. Treasury appropriations spend existing cash;
    /// debt appropriations create bounded public debt and commit the proceeds directly.
    /// # Errors
    /// Rejects unknown programs, duplicate annual decisions, terminal programs, absent authority,
    /// insufficient treasury, debt-limit excess, negative amounts, or inactive schedule years.
    #[allow(clippy::too_many_lines)]
    pub fn appropriate_government_program(
        &mut self,
        actor: ActorId,
        program: ProgramId,
        amount: Money,
        source: ProgramFundingSource,
    ) -> Result<(), WorldError> {
        if amount.minor_units() < 0 {
            return Err(WorldError::InvalidGovernmentProgram(
                "program appropriation cannot be negative",
            ));
        }
        let state = self
            .government_programs
            .get(&program)
            .ok_or(WorldError::UnknownGovernmentProgram(program))?;
        let country = state.country();
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        if matches!(
            state.status(),
            GovernmentProgramStatus::Completed
                | GovernmentProgramStatus::Cancelled
                | GovernmentProgramStatus::Failed
        ) {
            return Err(WorldError::InvalidGovernmentProgram(
                "terminal program cannot receive an appropriation",
            ));
        }
        let end_year = state
            .start_year()
            .checked_add(i32::from(state.duration_years()))
            .ok_or(WorldError::ArithmeticOverflow(
                "government program end year",
            ))?;
        if self.date.year() < state.start_year() || self.date.year() >= end_year {
            return Err(WorldError::InvalidGovernmentProgram(
                "program is outside its scheduled appropriation window",
            ));
        }
        if state.last_appropriation_year() == Some(self.date.year()) {
            return Err(WorldError::InvalidGovernmentProgram(
                "program already has an appropriation decision for this year",
            ));
        }
        let promised = state.promised_annual_funding();
        let opening = self.countries[&country].indicators();
        match source {
            ProgramFundingSource::Treasury => {
                if amount.minor_units() > opening.treasury().minor_units() {
                    return Err(WorldError::InsufficientTreasury(country));
                }
            }
            ProgramFundingSource::PublicDebt => {
                if amount.minor_units() > self.emergency_debt_headroom(country)? {
                    return Err(WorldError::InvalidGovernmentProgram(
                        "program public-debt limit exceeded",
                    ));
                }
            }
        }
        let new_treasury = match source {
            ProgramFundingSource::Treasury => opening
                .treasury()
                .minor_units()
                .checked_sub(amount.minor_units())
                .ok_or(WorldError::ArithmeticOverflow(
                    "program treasury appropriation",
                ))?,
            ProgramFundingSource::PublicDebt => opening.treasury().minor_units(),
        };
        let new_debt = match source {
            ProgramFundingSource::Treasury => opening.public_debt().minor_units(),
            ProgramFundingSource::PublicDebt => opening
                .public_debt()
                .minor_units()
                .checked_add(amount.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("program public debt"))?,
        };
        let mut next_program = state.clone();
        next_program.record_appropriation(self.date.year(), amount)?;
        let shortfall = promised
            .minor_units()
            .saturating_sub(amount.minor_units())
            .max(0);
        let country_state = self
            .countries
            .get_mut(&country)
            .ok_or(WorldError::UnknownCountry(country))?;
        country_state
            .indicators_mut()
            .set_treasury(Money::from_minor_units(new_treasury));
        country_state
            .indicators_mut()
            .set_public_debt(Money::from_minor_units(new_debt));
        let carryover = next_program.carryover_funding();
        self.government_programs.insert(program, next_program);
        self.events.append(
            self.date,
            DomainEvent::GovernmentProgramAppropriated {
                program,
                country,
                actor,
                source,
                promised,
                appropriated: amount,
                shortfall: Money::from_minor_units(shortfall),
                carryover,
            },
        );
        Ok(())
    }

    fn government_program_political_modifier(
        &self,
        program: &GovernmentProgram,
    ) -> (i32, Vec<ProgramPoliticalInfluence>) {
        let domestic_count = self
            .regions
            .values()
            .filter(|region| region.country() == program.country())
            .count()
            .max(1);
        let fair_value = u16::try_from(10_000 / domestic_count).unwrap_or(0);
        let fair_share = BasisPoints::new(fair_value).expect("fair program share is bounded");
        let political_offices: Vec<_> = self
            .power_nodes()
            .values()
            .filter(|node| {
                node.country() == program.country()
                    && node.kind() == crate::PowerNodeKind::PoliticalOffice
            })
            .map(crate::PowerNode::id)
            .collect();
        let mut evidence = Vec::new();
        let mut total = 0_i32;
        for influence in self.influences().values() {
            if !political_offices.contains(&influence.node()) {
                continue;
            }
            let Some(actor) = self.actors().get(&influence.actor()) else {
                continue;
            };
            let home_region = actor.home_region();
            if self
                .regions
                .get(&home_region)
                .is_none_or(|region| region.country() != program.country())
            {
                continue;
            }
            let regional_share = program
                .regional_shares()
                .get(&home_region)
                .copied()
                .unwrap_or(BasisPoints::ZERO);
            let share_delta = i32::from(regional_share.get()) - i32::from(fair_share.get());
            let modifier =
                share_delta.saturating_mul(i32::from(influence.weight().get())) / 10_000 / 4;
            if modifier == 0 {
                continue;
            }
            total = total.saturating_add(modifier);
            evidence.push(ProgramPoliticalInfluence {
                actor: influence.actor(),
                office: influence.node(),
                home_region,
                stance: if modifier > 0 {
                    ProgramPoliticalStance::Support
                } else {
                    ProgramPoliticalStance::Opposition
                },
                weight: influence.weight(),
                regional_share,
                fair_share,
                execution_modifier: modifier,
            });
        }
        (total.clamp(-2_000, 2_000), evidence)
    }

    /// Executes the current program allocation through regional administrative capacity.
    ///
    /// Appropriated money does not become services instantly. Each region can absorb between 25%
    /// and 100% of its committed share according to existing administration; the remainder stays
    /// as explicit carryover and increments delay memory.
    /// # Errors
    /// Rejects unknown/inactive programs, absent authority, missing current-year appropriation,
    /// or duplicate annual execution.
    #[allow(clippy::missing_panics_doc, clippy::too_many_lines)]
    pub fn execute_government_program(
        &mut self,
        actor: ActorId,
        program: ProgramId,
    ) -> Result<(), WorldError> {
        let state = self
            .government_programs
            .get(&program)
            .ok_or(WorldError::UnknownGovernmentProgram(program))?;
        let country = state.country();
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        if state.status() != GovernmentProgramStatus::Active {
            return Err(WorldError::InvalidGovernmentProgram(
                "only an active program can execute",
            ));
        }
        if state.last_appropriation_year() != Some(self.date.year()) {
            return Err(WorldError::InvalidGovernmentProgram(
                "program requires a current-year appropriation decision before execution",
            ));
        }
        if state.last_execution_year() == Some(self.date.year()) {
            return Err(WorldError::InvalidGovernmentProgram(
                "program execution already completed for this year",
            ));
        }
        let opening_carryover = state.carryover_funding();
        let (political_modifier, political_evidence) =
            self.government_program_political_modifier(state);
        let regional_commitments =
            allocate_program_budget(opening_carryover.minor_units(), state.regional_shares());
        let promised = state.promised_annual_funding().minor_units().max(1);
        let priority = state.priority();
        let promised_improvement = state.promised_improvement();
        let shares = state.regional_shares().clone();
        let mut deliveries = BTreeMap::new();
        let mut delivered_total = 0_i64;
        for (region, committed) in regional_commitments {
            let services = self.regional_public_services[&region];
            let administrative_absorption = 2_500_i32
                .saturating_add(i32::from(services.administration().get()) * 7_500 / 10_000);
            let political_administrative_absorption = administrative_absorption
                .saturating_add(political_modifier)
                .clamp(0, 10_000);
            let infrastructure_absorption = 5_000_i32
                .saturating_add(i32::from(services.infrastructure().get()) * 5_000 / 10_000);
            let institutional_absorption =
                political_administrative_absorption.min(infrastructure_absorption);
            let regional_workers_required = u64::try_from(
                u128::from(state.temporary_workers_required())
                    .saturating_mul(u128::from(shares[&region].get()))
                    / 10_000,
            )
            .unwrap_or(u64::MAX);
            let used_workers = self
                .government_program_labor_usage
                .get(&(self.date.year(), region))
                .copied()
                .unwrap_or(0);
            let unemployed = self
                .cohorts
                .values()
                .filter(|cohort| {
                    cohort.region() == region
                        && cohort.employment() == crate::EmploymentStatus::Unemployed
                })
                .map(|cohort| cohort.people().people())
                .sum::<u64>();
            let available_workers = unemployed.saturating_sub(used_workers);
            let labor_absorption = if regional_workers_required == 0 {
                10_000_i32
            } else {
                i32::try_from(
                    u128::from(available_workers).saturating_mul(10_000)
                        / u128::from(regional_workers_required),
                )
                .unwrap_or(10_000)
                .min(10_000)
            };
            let mut material_absorption = 10_000_i32;
            for (good, total_required) in state.material_requirements() {
                let regional_required = u64::try_from(
                    u128::from(total_required.get())
                        .saturating_mul(u128::from(shares[&region].get()))
                        / 10_000,
                )
                .unwrap_or(u64::MAX);
                if regional_required == 0 {
                    continue;
                }
                let available = self
                    .government_reserves
                    .get(&(region, *good))
                    .copied()
                    .unwrap_or_default()
                    .get();
                let availability = i32::try_from(
                    u128::from(available).saturating_mul(10_000) / u128::from(regional_required),
                )
                .unwrap_or(10_000)
                .min(10_000);
                material_absorption = material_absorption.min(availability);
            }
            let absorption = institutional_absorption
                .min(material_absorption)
                .min(labor_absorption);
            let delivered = i64::try_from(
                i128::from(committed).saturating_mul(i128::from(absorption)) / 10_000,
            )
            .map_err(|_| WorldError::ArithmeticOverflow("regional program delivery"))?;
            delivered_total =
                delivered_total
                    .checked_add(delivered)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "government program delivered total",
                    ))?;
            deliveries.insert(
                region,
                (
                    committed,
                    delivered,
                    absorption,
                    material_absorption,
                    labor_absorption,
                    regional_workers_required,
                ),
            );
        }
        let mut next_reserves = self.government_reserves.clone();
        let mut material_consumption = Vec::new();
        for (region, (_, _, absorption, _, _, _)) in &deliveries {
            for (good, total_required) in state.material_requirements() {
                let regional_required = u64::try_from(
                    u128::from(total_required.get())
                        .saturating_mul(u128::from(shares[region].get()))
                        / 10_000,
                )
                .unwrap_or(u64::MAX);
                let consumed = u64::try_from(
                    u128::from(regional_required)
                        .saturating_mul(u128::try_from(*absorption).unwrap_or(0))
                        / 10_000,
                )
                .unwrap_or(u64::MAX);
                if consumed == 0 {
                    continue;
                }
                let stock = next_reserves
                    .get(&(*region, *good))
                    .copied()
                    .unwrap_or_default()
                    .get();
                if consumed > stock {
                    return Err(WorldError::InvalidGovernmentProgram(
                        "planned program material use exceeds public reserves",
                    ));
                }
                next_reserves.insert((*region, *good), QuantityMilli::new(stock - consumed));
                material_consumption.push((*region, *good, QuantityMilli::new(consumed)));
            }
        }
        let mut next_labor_usage = self.government_program_labor_usage.clone();
        let mut temporary_labor = Vec::new();
        for (region, (_, delivered, absorption, _, _, required)) in &deliveries {
            if *required == 0 {
                continue;
            }
            let workers = u64::try_from(
                u128::from(*required).saturating_mul(u128::try_from(*absorption).unwrap_or(0))
                    / 10_000,
            )
            .unwrap_or(u64::MAX);
            let entry = next_labor_usage
                .entry((self.date.year(), *region))
                .or_default();
            *entry = entry
                .checked_add(workers)
                .ok_or(WorldError::ArithmeticOverflow("temporary program labor"))?;
            temporary_labor.push((*region, workers, Money::from_minor_units(*delivered)));
        }
        let mut next_cohorts = self.cohorts.clone();
        for (region, workers, wages) in &temporary_labor {
            let recipients: Vec<_> = next_cohorts
                .values()
                .filter(|cohort| {
                    cohort.region() == *region
                        && cohort.employment() == crate::EmploymentStatus::Unemployed
                })
                .map(crate::HouseholdCohort::id)
                .collect();
            if let Some(cohort) = recipients.first() {
                next_cohorts
                    .get_mut(cohort)
                    .expect("temporary labor cohort exists")
                    .credit_wealth(*wages)?;
            } else if *workers > 0 || wages.minor_units() > 0 {
                return Err(WorldError::InvalidGovernmentProgram(
                    "temporary program labor has no eligible wage recipient",
                ));
            }
        }
        let mut next = state.clone();
        next.record_execution(self.date.year(), Money::from_minor_units(delivered_total))?;
        let domestic_regions: Vec<_> = self
            .regions
            .values()
            .filter(|region| region.country() == country)
            .map(crate::Region::id)
            .collect();
        let mut regional_outcomes = Vec::new();
        for region in domestic_regions {
            let share = shares.get(&region).copied().unwrap_or(BasisPoints::ZERO);
            let promised_minor = i64::try_from(
                i128::from(state.promised_annual_funding().minor_units())
                    .saturating_mul(i128::from(share.get()))
                    / 10_000,
            )
            .map_err(|_| WorldError::ArithmeticOverflow("regional program promise"))?;
            let (committed_minor, delivered_minor) =
                deliveries.get(&region).map_or((0, 0), |row| (row.0, row.1));
            let fulfillment_value = if promised_minor <= 0 {
                0
            } else {
                i128::from(delivered_minor).saturating_mul(10_000) / i128::from(promised_minor)
            }
            .clamp(0, 10_000);
            let fulfillment = BasisPoints::new(u16::try_from(fulfillment_value).unwrap_or(10_000))
                .expect("program fulfillment is bounded");
            let outcome = if share == BasisPoints::ZERO {
                ProgramRegionalOutcomeKind::Excluded
            } else if fulfillment.get() >= 8_000 {
                ProgramRegionalOutcomeKind::Beneficiary
            } else {
                ProgramRegionalOutcomeKind::Underfulfilled
            };
            let political_shift = if outcome == ProgramRegionalOutcomeKind::Excluded {
                -100
            } else {
                let fulfilled_reward = i32::from(fulfillment.get()) * 150 / 10_000;
                let broken_penalty =
                    i32::from(10_000_u16.saturating_sub(fulfillment.get())) * 300 / 10_000;
                fulfilled_reward - broken_penalty
            };
            let memory = next.record_regional_memory(
                region,
                Money::from_minor_units(promised_minor),
                Money::from_minor_units(committed_minor),
                Money::from_minor_units(delivered_minor),
                fulfillment,
                outcome,
                political_shift,
            )?;
            regional_outcomes.push((region, share, memory, political_shift));
        }
        let remaining_carryover = next.carryover_funding();
        let years_delayed = next.years_delayed();
        self.government_programs.insert(program, next);
        self.government_reserves = next_reserves;
        self.government_program_labor_usage = next_labor_usage;
        self.cohorts = next_cohorts;
        for (region, share, memory, political_shift) in regional_outcomes {
            self.events.append(
                self.date,
                DomainEvent::GovernmentProgramRegionalOutcomeRecorded {
                    program,
                    country,
                    region,
                    share,
                    promised: memory.promised(),
                    committed: memory.committed(),
                    delivered: memory.delivered(),
                    fulfillment: memory.fulfillment(),
                    outcome: memory.outcome(),
                    years_excluded: memory.years_excluded(),
                    political_shift,
                    political_memory: memory.political_memory(),
                },
            );
        }
        for (region, workers, wages) in temporary_labor {
            self.events.append(
                self.date,
                DomainEvent::GovernmentProgramTemporaryLaborEmployed {
                    program,
                    country,
                    region,
                    workers,
                    wages,
                },
            );
        }
        for (region, good, quantity) in material_consumption {
            self.events.append(
                self.date,
                DomainEvent::GovernmentProgramMaterialConsumed {
                    program,
                    country,
                    region,
                    good,
                    quantity,
                },
            );
        }
        for influence in political_evidence {
            self.events.append(
                self.date,
                DomainEvent::GovernmentProgramPoliticalInfluenceApplied {
                    program,
                    country,
                    actor: influence.actor,
                    office: influence.office,
                    home_region: influence.home_region,
                    stance: influence.stance,
                    weight: influence.weight,
                    regional_share: influence.regional_share,
                    fair_share: influence.fair_share,
                    execution_modifier: influence.execution_modifier,
                },
            );
        }
        for (
            region,
            (committed, delivered, absorption, material_absorption, labor_absorption, _),
        ) in deliveries
        {
            let improvement_value = i128::from(promised_improvement.get())
                .saturating_mul(i128::from(delivered))
                / i128::from(promised);
            let improvement = BasisPoints::new(
                u16::try_from(improvement_value.clamp(0, 10_000)).unwrap_or(10_000),
            )
            .expect("program improvement is bounded");
            let services =
                self.regional_public_services[&region].improved_by_program(priority, improvement);
            self.regional_public_services.insert(region, services);
            self.events.append(
                self.date,
                DomainEvent::GovernmentProgramRegionalDelivery {
                    program,
                    country,
                    region,
                    share: shares[&region],
                    committed: Money::from_minor_units(committed),
                    delivered: Money::from_minor_units(delivered),
                    administrative_absorption: BasisPoints::new(
                        u16::try_from(absorption).unwrap_or(10_000),
                    )
                    .expect("administrative absorption is bounded"),
                    political_modifier,
                    material_absorption: BasisPoints::new(
                        u16::try_from(material_absorption).unwrap_or(10_000),
                    )
                    .expect("material absorption is bounded"),
                    labor_absorption: BasisPoints::new(
                        u16::try_from(labor_absorption).unwrap_or(10_000),
                    )
                    .expect("labor absorption is bounded"),
                    improvement,
                    priority,
                },
            );
        }
        self.events.append(
            self.date,
            DomainEvent::GovernmentProgramExecuted {
                program,
                country,
                actor,
                opening_carryover,
                delivered: Money::from_minor_units(delivered_total),
                remaining_carryover,
                years_delayed,
            },
        );
        Ok(())
    }

    /// Terminates future appropriations without refunding already committed carryover.
    /// # Errors
    /// Rejects unknown programs, absent authority, or an already terminal program.
    pub fn cancel_government_program(
        &mut self,
        actor: ActorId,
        program: ProgramId,
    ) -> Result<(), WorldError> {
        let state = self
            .government_programs
            .get(&program)
            .ok_or(WorldError::UnknownGovernmentProgram(program))?;
        let country = state.country();
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        if matches!(
            state.status(),
            GovernmentProgramStatus::Completed
                | GovernmentProgramStatus::Cancelled
                | GovernmentProgramStatus::Failed
        ) {
            return Err(WorldError::InvalidGovernmentProgram(
                "program is already terminal",
            ));
        }
        let mut next = state.clone();
        next.cancel();
        let retained_carryover = next.carryover_funding();
        self.government_programs.insert(program, next);
        self.events.append(
            self.date,
            DomainEvent::GovernmentProgramCancelled {
                program,
                country,
                actor,
                retained_carryover,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn government_programs(&self) -> &BTreeMap<ProgramId, GovernmentProgram> {
        &self.government_programs
    }
}

fn allocate_program_budget(
    budget: i64,
    shares: &BTreeMap<RegionId, BasisPoints>,
) -> BTreeMap<RegionId, i64> {
    let budget = budget.max(0);
    let mut rows = Vec::with_capacity(shares.len());
    let mut assigned = 0_i64;
    for (region, share) in shares {
        let scaled = i128::from(budget).saturating_mul(i128::from(share.get()));
        let floor = i64::try_from(scaled / 10_000).unwrap_or(i64::MAX);
        let remainder = scaled % 10_000;
        assigned = assigned.saturating_add(floor);
        rows.push((*region, floor, remainder));
    }
    rows.sort_by_key(|(region, _, remainder)| (std::cmp::Reverse(*remainder), *region));
    for row in rows
        .iter_mut()
        .take(usize::try_from(budget.saturating_sub(assigned)).unwrap_or(0))
    {
        row.1 = row.1.saturating_add(1);
    }
    rows.sort_by_key(|(region, _, _)| *region);
    rows.into_iter()
        .map(|(region, amount, _)| (region, amount))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, Country, Population, PowerNode, PowerNodeId, PowerNodeKind, Region, SimDate,
        WorldCommand, WorldSeed, replay_commands,
    };

    fn world() -> World {
        let mut world = World::new(WorldSeed::new(100), SimDate::new(2026, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Union").expect("country"))
            .expect("country");
        for id in 1..=2 {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        format!("Region {id}"),
                        Population::new(100),
                        Money::from_minor_units(10_000),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Premier", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_power_node(
                PowerNode::new(
                    PowerNodeId::new(1),
                    CountryId::new(1),
                    "Government",
                    PowerNodeKind::PoliticalOffice,
                    Some(ActorId::new(1)),
                )
                .expect("office"),
            )
            .expect("office");
        world
    }

    fn program(id: u32, initiator: u32) -> GovernmentProgram {
        GovernmentProgram::new(
            ProgramId::new(id),
            CountryId::new(1),
            ActorId::new(initiator),
            "Southern Valley Recovery",
            BTreeMap::from([
                (RegionId::new(1), BasisPoints::new(2_500).expect("share")),
                (RegionId::new(2), BasisPoints::new(7_500).expect("share")),
            ]),
            Money::from_minor_units(1_000_000),
            4,
            PublicServicePriority::Infrastructure,
            BasisPoints::new(2_000).expect("promise"),
            2026,
        )
        .expect("program")
    }

    #[test]
    fn authorized_program_may_promise_more_than_current_treasury() {
        let mut world = world();
        world
            .declare_government_program(program(1, 1))
            .expect("declaration");
        let stored = &world.government_programs()[&ProgramId::new(1)];
        assert_eq!(stored.status(), GovernmentProgramStatus::Announced);
        assert_eq!(stored.appropriated_funding(), Money::from_minor_units(0));
        assert_eq!(
            stored.promised_annual_funding(),
            Money::from_minor_units(1_000_000)
        );
    }

    #[test]
    fn declaration_is_replayable_and_unauthorized_attempt_is_atomic() {
        let mut direct = world();
        let mut replayed = direct.clone();
        let command = WorldCommand::DeclareGovernmentProgram(program(1, 1));
        command.apply(&mut direct).expect("direct");
        replay_commands(&mut replayed, &[command]).expect("replay");
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());

        let before = direct.clone();
        assert!(matches!(
            direct.declare_government_program(program(2, 99)),
            Err(WorldError::UnauthorizedGovernmentAction { .. })
        ));
        assert_eq!(direct, before);
    }

    #[test]
    fn treasury_and_debt_appropriations_use_real_distinct_resources() {
        let mut treasury_world = world();
        treasury_world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(500));
        treasury_world
            .declare_government_program(program(1, 1))
            .expect("program");
        treasury_world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(300),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        assert_eq!(
            treasury_world.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(200)
        );
        assert_eq!(
            treasury_world.government_programs()[&ProgramId::new(1)].carryover_funding(),
            Money::from_minor_units(300)
        );

        let mut debt_world = world();
        debt_world
            .declare_government_program(program(1, 1))
            .expect("program");
        debt_world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(300),
                ProgramFundingSource::PublicDebt,
            )
            .expect("debt appropriation");
        assert_eq!(
            debt_world.countries()[&CountryId::new(1)]
                .indicators()
                .treasury(),
            Money::from_minor_units(0)
        );
        assert_eq!(
            debt_world.countries()[&CountryId::new(1)]
                .indicators()
                .public_debt(),
            Money::from_minor_units(300)
        );
    }

    #[test]
    fn zero_funding_and_cancellation_are_explicit_replayable_choices() {
        let mut direct = world();
        direct
            .declare_government_program(program(1, 1))
            .expect("program");
        let mut replayed = direct.clone();
        let commands = [
            WorldCommand::AppropriateGovernmentProgram {
                actor: ActorId::new(1),
                program: ProgramId::new(1),
                amount: Money::from_minor_units(0),
                source: ProgramFundingSource::Treasury,
            },
            WorldCommand::CancelGovernmentProgram {
                actor: ActorId::new(1),
                program: ProgramId::new(1),
            },
        ];
        for command in &commands {
            command.apply(&mut direct).expect("direct");
        }
        replay_commands(&mut replayed, &commands).expect("replay");
        assert_eq!(direct, replayed);
        assert_eq!(
            direct.government_programs()[&ProgramId::new(1)].status(),
            GovernmentProgramStatus::Cancelled
        );
        assert_eq!(
            direct.government_programs()[&ProgramId::new(1)].appropriated_funding(),
            Money::from_minor_units(0)
        );
    }

    #[test]
    fn execution_is_administratively_bounded_and_preserves_carryover() {
        let mut world = world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        world
            .declare_government_program(program(1, 1))
            .expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(1_000),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        let program = &world.government_programs()[&ProgramId::new(1)];
        assert_eq!(program.delivered_funding(), Money::from_minor_units(624));
        assert_eq!(program.carryover_funding(), Money::from_minor_units(376));
        assert_eq!(program.years_delayed(), 1);
    }

    #[test]
    fn execution_is_once_per_year_and_replayable() {
        let mut direct = world();
        direct
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        direct
            .declare_government_program(program(1, 1))
            .expect("program");
        let mut replayed = direct.clone();
        let commands = [
            WorldCommand::AppropriateGovernmentProgram {
                actor: ActorId::new(1),
                program: ProgramId::new(1),
                amount: Money::from_minor_units(1_000),
                source: ProgramFundingSource::Treasury,
            },
            WorldCommand::ExecuteGovernmentProgram {
                actor: ActorId::new(1),
                program: ProgramId::new(1),
            },
        ];
        for command in &commands {
            command.apply(&mut direct).expect("direct");
        }
        replay_commands(&mut replayed, &commands).expect("replay");
        assert_eq!(direct, replayed);
        let before = direct.clone();
        assert!(matches!(
            direct.execute_government_program(ActorId::new(1), ProgramId::new(1)),
            Err(WorldError::InvalidGovernmentProgram(_))
        ));
        assert_eq!(direct, before);
    }

    #[test]
    fn favored_and_excluded_power_bases_change_execution_without_veto() {
        fn influenced_world(favor_patron: bool) -> World {
            let mut world = world();
            world
                .countries
                .get_mut(&CountryId::new(1))
                .expect("country")
                .indicators_mut()
                .set_treasury(Money::from_minor_units(1_000));
            world
                .register_actor(
                    Actor::new(ActorId::new(2), "Regional patron", RegionId::new(2), 1975)
                        .expect("actor"),
                )
                .expect("actor");
            world
                .establish_influence(crate::Influence::new(
                    ActorId::new(2),
                    PowerNodeId::new(1),
                    BasisPoints::new(8_000).expect("weight"),
                ))
                .expect("influence");
            let shares = if favor_patron {
                BTreeMap::from([
                    (RegionId::new(1), BasisPoints::new(2_500).expect("share")),
                    (RegionId::new(2), BasisPoints::new(7_500).expect("share")),
                ])
            } else {
                BTreeMap::from([
                    (RegionId::new(1), BasisPoints::FULL),
                    (RegionId::new(2), BasisPoints::ZERO),
                ])
            };
            let program = GovernmentProgram::new(
                ProgramId::new(1),
                CountryId::new(1),
                ActorId::new(1),
                "Political program",
                shares,
                Money::from_minor_units(1_000),
                2,
                PublicServicePriority::Infrastructure,
                BasisPoints::new(1_000).expect("promise"),
                2026,
            )
            .expect("program");
            world
                .declare_government_program(program)
                .expect("declaration");
            world
                .appropriate_government_program(
                    ActorId::new(1),
                    ProgramId::new(1),
                    Money::from_minor_units(1_000),
                    ProgramFundingSource::Treasury,
                )
                .expect("appropriation");
            world
                .execute_government_program(ActorId::new(1), ProgramId::new(1))
                .expect("execution");
            world
        }
        let supported = influenced_world(true);
        let opposed = influenced_world(false);
        assert!(
            supported.government_programs()[&ProgramId::new(1)]
                .delivered_funding()
                .minor_units()
                > 624
        );
        assert!(
            opposed.government_programs()[&ProgramId::new(1)]
                .delivered_funding()
                .minor_units()
                < 624
        );
        assert!(supported.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::GovernmentProgramPoliticalInfluenceApplied {
                stance: ProgramPoliticalStance::Support,
                ..
            }
        )));
        assert!(opposed.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::GovernmentProgramPoliticalInfluenceApplied {
                stance: ProgramPoliticalStance::Opposition,
                ..
            }
        )));
    }

    #[test]
    fn scarce_public_materials_bind_delivery_and_are_physically_consumed() {
        let mut world = world();
        world
            .register_good(
                crate::Good::new(GoodId::new(1), "Construction materials").expect("good"),
            )
            .expect("good");
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        world
            .government_reserves
            .insert((RegionId::new(1), GoodId::new(1)), QuantityMilli::new(125));
        world
            .government_reserves
            .insert((RegionId::new(2), GoodId::new(1)), QuantityMilli::new(375));
        let program = program(1, 1)
            .with_material_requirement(GoodId::new(1), QuantityMilli::new(1_000))
            .expect("requirement");
        world.declare_government_program(program).expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(1_000),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        let state = &world.government_programs()[&ProgramId::new(1)];
        assert_eq!(state.delivered_funding(), Money::from_minor_units(500));
        assert_eq!(state.carryover_funding(), Money::from_minor_units(500));
        assert_eq!(
            world.government_reserves()[&(RegionId::new(1), GoodId::new(1))],
            QuantityMilli::new(0)
        );
        assert_eq!(
            world.government_reserves()[&(RegionId::new(2), GoodId::new(1))],
            QuantityMilli::new(0)
        );
        assert!(world.events().events().iter().any(|event| matches!(event.event(), DomainEvent::GovernmentProgramMaterialConsumed { quantity, .. } if quantity.get() > 0)));
    }

    #[test]
    fn temporary_workers_bind_execution_receive_wages_and_cannot_be_double_used() {
        let mut world = world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        let cohort = crate::HouseholdCohort::new(
            crate::CohortId::new(1),
            RegionId::new(1),
            crate::NeedProfileId::new(1),
            crate::Population::new(10),
            10,
            crate::AgeBand::Adult,
            crate::HouseholdType::WorkingAge,
            crate::EducationLevel::Secondary,
            crate::EmploymentStatus::Unemployed,
            Money::from_minor_units(0),
            Money::from_minor_units(0),
            Money::from_minor_units(0),
        )
        .expect("cohort");
        world.cohorts.insert(cohort.id(), cohort);
        let program = program(1, 1).with_temporary_workers(40);
        world.declare_government_program(program).expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(1_000),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        assert!(
            world.government_programs()[&ProgramId::new(1)]
                .delivered_funding()
                .minor_units()
                < 624
        );
        assert!(
            world.cohorts[&crate::CohortId::new(1)]
                .liquid_wealth()
                .minor_units()
                > 0
        );
        assert!(world.government_program_labor_usage[&(2026, RegionId::new(1))] > 0);
    }

    #[test]
    fn beneficiaries_and_excluded_regions_gain_distinct_persistent_memory() {
        let mut world = world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        let program = GovernmentProgram::new(
            ProgramId::new(1),
            CountryId::new(1),
            ActorId::new(1),
            "Capital first",
            BTreeMap::from([
                (RegionId::new(1), BasisPoints::FULL),
                (RegionId::new(2), BasisPoints::ZERO),
            ]),
            Money::from_minor_units(1_000),
            2,
            PublicServicePriority::Infrastructure,
            BasisPoints::new(1_000).expect("promise"),
            2026,
        )
        .expect("program");
        world.declare_government_program(program).expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(1_000),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        let memory = world.government_programs()[&ProgramId::new(1)].regional_memory();
        assert_eq!(
            memory[&RegionId::new(2)].outcome(),
            ProgramRegionalOutcomeKind::Excluded
        );
        assert_eq!(memory[&RegionId::new(2)].years_excluded(), 1);
        assert!(memory[&RegionId::new(2)].political_memory() < 0);
        assert_eq!(
            memory[&RegionId::new(1)].outcome(),
            ProgramRegionalOutcomeKind::Underfulfilled
        );
    }

    #[test]
    fn broken_and_excluded_promises_reduce_satisfaction_legitimacy_and_cohesion_once() {
        let mut world = world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        let program = GovernmentProgram::new(
            ProgramId::new(1),
            CountryId::new(1),
            ActorId::new(1),
            "Unequal promise",
            BTreeMap::from([
                (RegionId::new(1), BasisPoints::FULL),
                (RegionId::new(2), BasisPoints::ZERO),
            ]),
            Money::from_minor_units(1_000),
            2,
            PublicServicePriority::Infrastructure,
            BasisPoints::new(1_000).expect("promise"),
            2026,
        )
        .expect("program");
        world.declare_government_program(program).expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(0),
                ProgramFundingSource::Treasury,
            )
            .expect("zero appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        let before_legitimacy = world.countries()[&CountryId::new(1)]
            .indicators()
            .legitimacy();
        let before_cohesion = world.countries()[&CountryId::new(1)]
            .indicators()
            .elite_cohesion();
        world
            .update_annual_regional_interests(crate::SimDate::new(2027, 1).expect("date"))
            .expect("consequences");
        assert!(world.regional_interests()[&RegionId::new(1)].satisfaction() < BasisPoints::HALF);
        assert!(world.regional_interests()[&RegionId::new(2)].satisfaction() < BasisPoints::HALF);
        assert!(
            world.countries()[&CountryId::new(1)]
                .indicators()
                .legitimacy()
                < before_legitimacy
        );
        assert!(
            world.countries()[&CountryId::new(1)]
                .indicators()
                .elite_cohesion()
                < before_cohesion
        );
    }

    #[test]
    fn chronicle_names_program_delivery_losers_and_political_consequences() {
        let mut world = world();
        world
            .countries
            .get_mut(&CountryId::new(1))
            .expect("country")
            .indicators_mut()
            .set_treasury(Money::from_minor_units(1_000));
        let program = GovernmentProgram::new(
            ProgramId::new(1),
            CountryId::new(1),
            ActorId::new(1),
            "Visible State",
            BTreeMap::from([
                (RegionId::new(1), BasisPoints::FULL),
                (RegionId::new(2), BasisPoints::ZERO),
            ]),
            Money::from_minor_units(1_000),
            2,
            PublicServicePriority::Infrastructure,
            BasisPoints::new(1_000).expect("promise"),
            2026,
        )
        .expect("program");
        world.declare_government_program(program).expect("program");
        world
            .appropriate_government_program(
                ActorId::new(1),
                ProgramId::new(1),
                Money::from_minor_units(0),
                ProgramFundingSource::Treasury,
            )
            .expect("appropriation");
        world
            .execute_government_program(ActorId::new(1), ProgramId::new(1))
            .expect("execution");
        world
            .update_annual_regional_interests(crate::SimDate::new(2027, 1).expect("date"))
            .expect("consequences");
        let text = world
            .chronicle()
            .into_iter()
            .map(|entry| entry.text)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(text.contains("Visible State"));
        assert!(text.contains("regional loser"));
        assert!(text.contains("polarization"));
    }
}
