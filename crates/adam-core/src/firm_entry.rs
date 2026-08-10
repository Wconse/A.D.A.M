use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ActorId, BasisPoints, CohortId, CorporateRole, DomainEvent, EmploymentAgreement, Firm,
    FirmAppointment, FirmId, FirmPolicy, GoodId, Money, NeedTier, OwnershipStake, QuantityMilli,
    RecipeId, RegionId, World, WorldCommand, WorldError,
};

/// Entry is justified by three months of a region getting nothing of what it
/// asked for, or by proportionally longer stretches of partial shortage. The
/// level is the same one entry always used; what changed is that severity now
/// counts, so a trickle that never closes the gap is no longer worth as much
/// as an empty shelf.
const ENTRY_SIGNAL_BPS: u32 = 3 * crate::accumulator::FULL_MONTH_BPS;
/// A quarter of a full month of evidence fades per month. Nobody founds a farm
/// on a shortage the region no longer has, but nor does one relieved month
/// erase the memory of a year of them: the founder is deciding against a
/// standing impression, and impressions decay rather than reset.
const ENTRY_EVIDENCE_LEAK_BPS: u32 = crate::accumulator::FULL_MONTH_BPS / 4;
/// A worker does not change employer in their first weeks on a job. This is the
/// same tenure the competitive labor market already requires before a switch,
/// and it is owned by the worker, who gives up a place they have only just
/// settled into, and by the poacher, who must wait for the season to turn.
const MINIMUM_TENURE_BEFORE_MOVE_MONTHS: u8 = 3;

/// Refusal used when a founding plan can no longer name a worker to staff it.
const NO_HIRABLE_WORKER: &str =
    "firm founding requires a local worker the founder can hire or outbid";

/// What a region failed to get, keyed by region and good, paired with what it
/// asked for in the same month.
type RegionalShortageEvidence = BTreeMap<(RegionId, GoodId), (u64, u64)>;

/// The smallest crew a works can open with, which is also the unit the labour
/// requirement of a recipe is measured against.
const STARTUP_WORKERS: u64 = 1;
const WORKING_CAPITAL_WAGE_MONTHS: i64 = 3;
const CAPACITY_VALUE_MULTIPLIER: i64 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmFoundingPlan {
    pub founder: ActorId,
    pub firm: FirmId,
    pub region: RegionId,
    pub recipe: RecipeId,
    pub cohort: CohortId,
    /// The employer this firm takes its first worker from, when the region had
    /// nobody idle and the founder had to outbid somebody for staff.
    #[serde(default)]
    pub poached_from: Option<FirmId>,
    pub wage: Money,
    pub capital_cost: Money,
    pub working_capital: Money,
    /// The monthly capacity this founding actually builds.
    ///
    /// A shortage of half a million milli-units is not answered by a single
    /// batch. What gets built is bounded by three real things and nothing
    /// else: the unmet demand that called the firm into being, the cash the
    /// founder can commit, and the hands the region can spare.
    #[serde(default)]
    pub capacity_batches: u64,
    /// The staff the plan is drawn against, one worker-month per thousand
    /// milli-units of recipe labour.
    #[serde(default)]
    pub workers: u64,
    pub pressure_months: u8,
}

impl FirmFoundingPlan {
    #[must_use]
    pub fn total_commitment(self) -> Money {
        Money::from_minor_units(
            self.capital_cost
                .minor_units()
                .saturating_add(self.working_capital.minor_units()),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmEntryDecision {
    pub region: RegionId,
    pub good: GoodId,
    pub unmet_quantity: QuantityMilli,
    pub plan: FirmFoundingPlan,
}

/// The worker a new firm would start with, and what it must pay to get them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EntryRecruitment {
    cohort: CohortId,
    wage: Money,
    poached_from: Option<FirmId>,
    /// How many workers this source could actually supply. A region with one
    /// idle pair of hands and a region with two hundred are not the same
    /// economy, and entry has no business pretending otherwise.
    available_workers: u64,
}

#[derive(Clone, Copy)]
struct FoundationEconomics {
    wage: Money,
    capital_cost: Money,
    working_capital: Money,
    capacity_batches: u64,
    workers: u64,
}

impl World {
    /// Reviews persistent residual survival shortages and founds feasible local producers.
    ///
    /// A shortage must remain after trade, rationing, and reserve release for three consecutive
    /// months. Founding uses a real local actor's cash, one unemployed local worker, an existing
    /// production recipe, and observed regional prices. Every founding crosses the ordinary
    /// replayable command boundary.
    ///
    /// # Errors
    /// Returns an error atomically for duplicate monthly execution or an invalid founding.
    pub fn execute_observed_firm_entry(&mut self) -> Result<Vec<FirmEntryDecision>, WorldError> {
        if self.last_firm_entry_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed firm entry",
                date: self.date,
            });
        }
        let unmet = self.current_regional_entry_evidence()?;
        let mut keys: BTreeSet<_> = self.firm_entry_pressure.keys().copied().collect();
        keys.extend(unmet.keys().copied());
        let mut next = self.clone();
        let mut decisions = Vec::new();
        for (region, good) in keys {
            let (shortfall, demanded) = unmet.get(&(region, good)).copied().unwrap_or_default();
            let quantity = QuantityMilli::new(shortfall);
            let pressure = {
                let pressure = next.firm_entry_pressure.entry((region, good)).or_default();
                *pressure = pressure.accumulate(
                    crate::accumulator::shortage_charge_bps(shortfall, demanded),
                    ENTRY_EVIDENCE_LEAK_BPS,
                );
                *pressure
            };
            let pressure_months = pressure.months();
            // Below the signal the region has simply not asked loudly enough
            // yet, which is an absence of evidence rather than an obstacle.
            let review = if pressure.reached(ENTRY_SIGNAL_BPS) {
                Some(next.plan_firm_entry(region, good, pressure_months, shortfall)?)
            } else {
                None
            };
            let obstacle = match review {
                Some(Err(obstacle)) => Some(obstacle),
                _ => None,
            };
            let plan = match review {
                Some(Ok(plan)) => Some(plan),
                _ => None,
            };
            if let Some(plan) = plan {
                WorldCommand::FoundFirm(plan).apply(&mut next)?;
                next.firm_entry_pressure.insert(
                    (region, good),
                    crate::accumulator::LeakyPressure::discharged(),
                );
                decisions.push(FirmEntryDecision {
                    region,
                    good,
                    unmet_quantity: quantity,
                    plan,
                });
            }
            next.events.append(
                next.date,
                DomainEvent::FirmEntryOpportunityReviewed {
                    region,
                    good,
                    unmet_quantity: quantity,
                    pressure_months,
                    entry_feasible: plan.is_some(),
                    firm_founded: plan.map(|value| value.firm),
                    obstacle,
                },
            );
        }
        next.last_firm_entry_date = Some(next.date);
        next.events.append(
            next.date,
            DomainEvent::ObservedFirmEntryCompleted {
                opportunities_reviewed: u64::try_from(next.firm_entry_pressure.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("firm entry opportunities"))?,
                firms_founded: u64::try_from(decisions.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("firms founded"))?,
            },
        );
        *self = next;
        Ok(decisions)
    }

    /// Applies a fully funded founding plan atomically.
    ///
    /// Refused when the region can no longer supply the worker the plan was
    /// drawn against.
    /// # Errors
    /// Returns an error for stale economics, insufficient founder cash, or invalid references.
    pub fn found_firm(&mut self, plan: FirmFoundingPlan) -> Result<(), WorldError> {
        let founder = self
            .actors()
            .get(&plan.founder)
            .ok_or(WorldError::UnknownActor(plan.founder))?;
        if founder.home_region() != plan.region {
            return Err(WorldError::InvalidProduction(
                "firm founder must live in the founding region",
            ));
        }
        if self.firms.contains_key(&plan.firm) {
            return Err(WorldError::DuplicateFirm(plan.firm));
        }
        self.check_founding_plan(plan)?;
        let commitment = checked_money_add(plan.capital_cost, plan.working_capital)?;
        let available = self
            .actor_cash
            .get(&plan.founder)
            .copied()
            .unwrap_or_default();
        if available.minor_units() < commitment.minor_units() {
            return Err(WorldError::InvalidActorCash);
        }
        let recipe = self
            .production_recipes
            .get(&plan.recipe)
            .ok_or(WorldError::UnknownRecipe(plan.recipe))?;
        let good = self
            .goods
            .get(&recipe.output_good())
            .ok_or(WorldError::UnknownGood(recipe.output_good()))?;
        let region = self
            .regions
            .get(&plan.region)
            .ok_or(WorldError::UnknownRegion(plan.region))?;
        let name = format!(
            "{} {} Works {}",
            region.name(),
            good.name(),
            plan.firm.get()
        );
        let mut next = self.clone();
        next.actor_cash.insert(
            plan.founder,
            Money::from_minor_units(available.minor_units() - commitment.minor_units()),
        );
        next.register_firm(Firm::new(
            plan.firm,
            name,
            plan.region,
            plan.recipe,
            plan.workers,
            plan.capacity_batches,
            plan.working_capital,
            BTreeMap::new(),
        )?)?;
        next.distribute_capital_outlay(plan.firm, plan.region, plan.capital_cost)?;
        next.register_ownership_stake(OwnershipStake::new(
            plan.firm,
            plan.founder,
            BasisPoints::FULL,
            BasisPoints::FULL,
        ))?;
        next.register_firm_appointment(FirmAppointment::new(
            plan.firm,
            plan.founder,
            CorporateRole::ChiefExecutive,
        ))?;
        next.set_firm_policy(plan.founder, plan.firm, startup_policy()?)?;
        next.release_poached_worker(plan)?;
        next.register_employment_agreement(EmploymentAgreement::new(
            plan.firm,
            plan.cohort,
            plan.workers,
            plan.wage,
        )?)?;
        next.set_firm_production_target(plan.founder, plan.firm, plan.capacity_batches)?;
        next.record_firm_founded_event(plan, recipe.output_good());
        *self = next;
        Ok(())
    }

    /// Checks a founding plan against the region as it stands right now.
    ///
    /// A plan is drawn one month and applied the next, so prices, wages and
    /// spare hands may all have moved underneath it. Nothing here repairs the
    /// plan: a stale plan is refused, because a founding that quietly changed
    /// its own terms would be a firm nobody agreed to build.
    fn check_founding_plan(&self, plan: FirmFoundingPlan) -> Result<(), WorldError> {
        let recruitment = self
            .plan_entry_recruitment(plan.region, plan.recipe)?
            .map_err(|_| WorldError::InvalidProduction(NO_HIRABLE_WORKER))?;
        let economics = self
            .foundation_economics(
                plan.region,
                plan.recipe,
                recruitment,
                plan.capacity_batches,
                plan.workers,
            )?
            .ok_or(WorldError::InvalidProduction(
                "firm founding requires observed prices and a local worker",
            ))?;
        // A works cannot open around staff the region no longer has to give.
        if plan.workers > recruitment.available_workers {
            return Err(WorldError::InvalidProduction(NO_HIRABLE_WORKER));
        }
        if plan.cohort != recruitment.cohort
            || plan.poached_from != recruitment.poached_from
            || plan.wage != economics.wage
            || plan.capital_cost != economics.capital_cost
            || plan.working_capital != economics.working_capital
            || plan.capacity_batches == 0
            || plan.workers == 0
            || plan.pressure_months == 0
        {
            return Err(WorldError::InvalidProduction(
                "firm founding plan does not match current regional economics",
            ));
        }
        Ok(())
    }

    /// Takes the first worker off the payroll they are leaving.
    ///
    /// The first worker of a new firm is not conjured. If nobody in the region
    /// was idle, the founder bought them away from an employer that now has one
    /// fewer, and that employer answers - or declines to answer - next month
    /// through the same retention machinery any other poaching would trigger.
    fn release_poached_worker(&mut self, plan: FirmFoundingPlan) -> Result<(), WorldError> {
        let Some(previous_employer) = plan.poached_from else {
            return Ok(());
        };
        let staffed = self
            .employment_agreements
            .get(&(previous_employer, plan.cohort))
            .ok_or(WorldError::InvalidEmployment(
                "a poached worker must have a standing agreement to leave",
            ))?
            .workers();
        // The new works hires the whole crew its plan is drawn against, so the
        // employer it took them from loses exactly that many. Releasing one
        // worker while hiring several would conjure the difference out of air
        // and put more people to work than the cohort contains.
        let remaining = staffed
            .checked_sub(plan.workers)
            .ok_or(WorldError::InvalidEmployment(
                "a poached crew must be staffed at the previous employer",
            ))?;
        self.change_employment_workers(previous_employer, plan.cohort, remaining)
    }

    /// Pays capital spending to the households that build the works.
    ///
    /// Construction firms and equipment makers are not modelled as separate
    /// counterparties yet, so the capital cost of new capacity reaches local
    /// households directly as building income. This is what keeps the circuit
    /// closed on the investment side: founding capital changes hands instead of
    /// vanishing from the world, exactly as ADR 0170 did for taxation.
    pub(crate) fn distribute_capital_outlay(
        &mut self,
        firm: FirmId,
        region: RegionId,
        amount: Money,
    ) -> Result<(), WorldError> {
        let shares = self.plan_regional_capital_shares(region, amount);
        if shares.is_empty() {
            return Err(WorldError::InvalidProduction(
                "capital spending needs local households to build the works",
            ));
        }
        for (cohort, share) in shares {
            self.cohorts
                .get_mut(&cohort)
                .ok_or(WorldError::UnknownCohort(cohort))?
                .credit_wealth(share)?;
            self.events.append(
                self.date,
                DomainEvent::CapitalOutlayDistributed {
                    firm,
                    cohort,
                    amount: share,
                },
            );
        }
        Ok(())
    }

    /// Splits capital spending across a region's cohorts by population with the
    /// largest-remainder method, so the paid total equals the spent total to the
    /// minor unit and no money is created or destroyed in the split.
    pub(crate) fn plan_regional_capital_shares(
        &self,
        region: RegionId,
        amount: Money,
    ) -> Vec<(CohortId, Money)> {
        let total = u128::try_from(amount.minor_units().max(0)).unwrap_or_default();
        if total == 0 {
            return Vec::new();
        }
        let mut rows: Vec<(CohortId, u128, u128)> = Vec::new();
        let mut population = 0_u128;
        for cohort in self.cohorts.values() {
            let people = u128::from(cohort.people().people());
            if cohort.region() == region && people > 0 {
                population += people;
                rows.push((cohort.id(), people, 0));
            }
        }
        if population == 0 {
            return Vec::new();
        }
        let mut assigned = 0_u128;
        for row in &mut rows {
            let numerator = total * row.1;
            row.1 = numerator / population;
            row.2 = numerator % population;
            assigned += row.1;
        }
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|&first, &second| {
            rows[second]
                .2
                .cmp(&rows[first].2)
                .then_with(|| rows[first].0.cmp(&rows[second].0))
        });
        let leftover = usize::try_from(total - assigned).unwrap_or_default();
        for index in order.into_iter().take(leftover) {
            rows[index].1 += 1;
        }
        rows.into_iter()
            .filter(|row| row.1 > 0)
            .map(|(id, share, _)| {
                (
                    id,
                    Money::from_minor_units(i64::try_from(share).unwrap_or(i64::MAX)),
                )
            })
            .collect()
    }

    fn record_firm_founded_event(&mut self, plan: FirmFoundingPlan, good: GoodId) {
        self.events.append(
            self.date,
            DomainEvent::FirmFoundedFromOpportunity {
                founder: plan.founder,
                firm: plan.firm,
                region: plan.region,
                good,
                recipe: plan.recipe,
                cohort: plan.cohort,
                wage: plan.wage,
                capital_cost: plan.capital_cost,
                working_capital: plan.working_capital,
                pressure_months: plan.pressure_months,
            },
        );
    }

    /// Everything a region asked for this month and did not get, from either
    /// kind of buyer.
    ///
    /// Entry answers two kinds of hunger, not one. A household that cannot buy
    /// bread and a bakery that cannot buy grain are both a region asking for a
    /// good it does not make, and the second was invisible for a long time: an
    /// intermediate good is never anybody's survival need, so a permanently
    /// starved input channel could shout its price and its shortfall every
    /// month forever without a single farm being founded.
    fn current_regional_entry_evidence(&self) -> Result<RegionalShortageEvidence, WorldError> {
        let mut unmet = self.current_regional_survival_shortages()?;
        for (&(region, good), &quantity) in &self.monthly_firm_input_shortfalls {
            if quantity.get() == 0 {
                continue;
            }
            let row = unmet.entry((region, good)).or_default();
            row.0 = row
                .0
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow(
                    "regional entry shortage evidence",
                ))?;
            row.1 = row
                .1
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow(
                    "regional entry demand evidence",
                ))?;
        }
        // What the region did manage to buy belongs in the denominator: a mill
        // that got nine tenths of its grain is short, but it is not the same
        // kind of short as one that got none, and a founder can tell the
        // difference.
        for (&(buyer, good), &(quantity, _)) in &self.monthly_firm_procurement_purchases {
            if quantity.get() == 0 {
                continue;
            }
            let Some(firm) = self.firms.get(&buyer) else {
                continue;
            };
            let Some(row) = unmet.get_mut(&(firm.region(), good)) else {
                continue;
            };
            row.1 = row
                .1
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow(
                    "regional entry demand evidence",
                ))?;
        }
        Ok(unmet)
    }

    /// What a region failed to get, together with what it asked for.
    ///
    /// The shortfall alone cannot say how bad a month was: a thousand missing
    /// units are a catastrophe in a region that wanted a thousand and a
    /// rounding error in one that wanted a million. Carrying the denominator
    /// keeps severity comparable across regions and across goods.
    fn current_regional_survival_shortages(&self) -> Result<RegionalShortageEvidence, WorldError> {
        let mut shortages: RegionalShortageEvidence = BTreeMap::new();
        for ((cohort, good, tier), quantity) in &self.unmet_demand {
            if *tier != NeedTier::Survival || quantity.get() == 0 {
                continue;
            }
            let region = self
                .cohorts
                .get(cohort)
                .ok_or(WorldError::UnknownCohort(*cohort))?
                .region();
            let row = shortages.entry((region, *good)).or_default();
            row.0 = row
                .0
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("regional survival shortage"))?;
            row.1 = row
                .1
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("regional survival demand"))?;
        }
        for ((cohort, good, tier), quantity) in &self.monthly_consumption {
            if *tier != NeedTier::Survival || quantity.get() == 0 {
                continue;
            }
            let region = self
                .cohorts
                .get(cohort)
                .ok_or(WorldError::UnknownCohort(*cohort))?
                .region();
            let Some(row) = shortages.get_mut(&(region, *good)) else {
                continue;
            };
            row.1 = row
                .1
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("regional survival demand"))?;
        }
        Ok(shortages)
    }

    /// Plans an entry, or names the one scarce thing that stopped it.
    ///
    /// The outer result is a failure of the world; the inner one is a verdict
    /// about this opportunity. A region that wants a good and gets no producer
    /// of it must be able to say which of the four ingredients was missing,
    /// otherwise an economy that cannot answer a shortage reads exactly like an
    /// economy that never heard it.
    fn plan_firm_entry(
        &self,
        region: RegionId,
        good: GoodId,
        pressure_months: u8,
        shortfall_milli: u64,
    ) -> Result<Result<FirmFoundingPlan, crate::FirmEntryObstacle>, WorldError> {
        let Some(recipe) = self
            .production_recipes
            .values()
            .filter(|recipe| recipe.output_good() == good)
            .map(crate::ProductionRecipe::id)
            .next()
        else {
            return Ok(Err(crate::FirmEntryObstacle::NoKnownRecipe));
        };
        let recruitment = match self.plan_entry_recruitment(region, recipe)? {
            Ok(recruitment) => recruitment,
            Err(obstacle) => return Ok(Err(obstacle)),
        };
        // The founder is settled before the size, because how large a works can
        // open is partly the question of whose wallet stands behind it.
        let mut founder = None;
        let mut founder_cash = Money::default();
        for (actor, definition) in self.actors() {
            if definition.home_region() != region {
                continue;
            }
            let cash = self.actor_cash.get(actor).copied().unwrap_or_default();
            if founder.is_none() || cash.minor_units() > founder_cash.minor_units() {
                founder = Some(*actor);
                founder_cash = cash;
            }
        }
        let Some(founder) = founder else {
            return Ok(Err(crate::FirmEntryObstacle::NoSolventFounder));
        };
        let (wanted_batches, workers_per_batch) = {
            let definition = self
                .production_recipes
                .get(&recipe)
                .ok_or(WorldError::UnknownRecipe(recipe))?;
            (
                shortfall_milli
                    .div_ceil(definition.output_per_batch().get().max(1))
                    .max(1),
                definition
                    .labor_milli_worker_months()
                    .max(1)
                    .div_ceil(1_000)
                    .max(1),
            )
        };
        // Hands the region can spare, expressed as the batches they could run.
        let hirable_batches = (recruitment.available_workers / workers_per_batch).max(1);
        let Some(unit) =
            self.foundation_economics(region, recipe, recruitment, 1, workers_per_batch)?
        else {
            return Ok(Err(crate::FirmEntryObstacle::NoLocalPrices));
        };
        let unit_commitment = checked_money_add(unit.capital_cost, unit.working_capital)?
            .minor_units()
            .max(1);
        let affordable_batches = u64::try_from(founder_cash.minor_units().max(0) / unit_commitment)
            .map_err(|_| WorldError::ArithmeticOverflow("affordable startup batches"))?;
        // Not even one batch is within reach, which is the old obstacle exactly.
        if affordable_batches == 0 {
            return Ok(Err(crate::FirmEntryObstacle::NoSolventFounder));
        }
        let capacity_batches = wanted_batches.min(hirable_batches).min(affordable_batches);
        let workers = capacity_batches
            .checked_mul(workers_per_batch)
            .ok_or(WorldError::ArithmeticOverflow("startup staff requirement"))?;
        let Some(economics) =
            self.foundation_economics(region, recipe, recruitment, capacity_batches, workers)?
        else {
            return Ok(Err(crate::FirmEntryObstacle::NoLocalPrices));
        };
        let commitment = checked_money_add(economics.capital_cost, economics.working_capital)?;
        if founder_cash.minor_units() < commitment.minor_units() {
            return Ok(Err(crate::FirmEntryObstacle::NoSolventFounder));
        }
        let firm = FirmId::new(self.firms.keys().next_back().map_or(Ok(1), |id| {
            id.get()
                .checked_add(1)
                .ok_or(WorldError::ArithmeticOverflow("autonomous firm identifier"))
        })?);
        Ok(Ok(FirmFoundingPlan {
            founder,
            firm,
            region,
            recipe,
            cohort: recruitment.cohort,
            poached_from: recruitment.poached_from,
            wage: economics.wage,
            capital_cost: economics.capital_cost,
            working_capital: economics.working_capital,
            capacity_batches: economics.capacity_batches,
            workers: economics.workers,
            pressure_months,
        }))
    }

    /// Finds the worker a new firm would start with, and what it must pay them.
    ///
    /// Entry used to wait for somebody to be idle, and in a fully employed
    /// region that meant a shortage could shout its price forever with nobody
    /// able to answer it: the entry pressure ADR 0176 accumulates had no channel
    /// into production at all. In life a new employer does not wait for
    /// unemployment. It takes staff from an existing employer by offering more
    /// than the work is currently paid.
    ///
    /// Both sides of a poached hire are owned. The founder pays a wage strictly
    /// above the incumbent's and must carry three months of it as working
    /// capital before a single batch is sold, and it may not bid past what the
    /// work is worth at local prices, so nobody buys labour they cannot earn
    /// back. The incumbent pays by losing a worker it had, and answers - or
    /// declines to answer - through the ordinary retention machinery next month.
    /// The worker is paid for the move, which is why they make it.
    fn plan_entry_recruitment(
        &self,
        region: RegionId,
        recipe: RecipeId,
    ) -> Result<Result<EntryRecruitment, crate::FirmEntryObstacle>, WorldError> {
        // Entry predates the competitive labor market. Recipes without an explicit
        // market profile retain the historical Basic qualification floor here;
        // only post-entry vacancy matching is opt-in.
        let minimum = self
            .recipe_minimum_education
            .get(&recipe)
            .copied()
            .unwrap_or(crate::EducationLevel::Basic);
        let mut idle = None;
        let mut idle_available = 0_u64;
        for (cohort, definition) in &self.cohorts {
            if definition.region() != region
                || definition.employment() != crate::EmploymentStatus::Unemployed
                || definition.education() < minimum
            {
                continue;
            }
            let allocated: u64 = self
                .employment_agreements
                .values()
                .filter(|agreement| agreement.cohort() == *cohort && agreement.active())
                .map(EmploymentAgreement::workers)
                .sum();
            let available = definition.people().people().saturating_sub(allocated);
            if available > idle_available {
                idle = Some(*cohort);
                idle_available = available;
            }
        }
        // Idle labour is hired first, and at its own reservation wage, because
        // nobody outbids an employer for a worker who is standing in the street.
        if let Some(cohort) = idle.filter(|_| idle_available >= STARTUP_WORKERS) {
            return Ok(Ok(EntryRecruitment {
                cohort,
                wage: self.idle_reservation_wage(cohort)?,
                poached_from: None,
                available_workers: idle_available,
            }));
        }
        let Some(ceiling) = self.recipe_labor_value_ceiling(region, recipe)? else {
            return Ok(Err(crate::FirmEntryObstacle::NoLocalPrices));
        };
        let mut best: Option<EntryRecruitment> = None;
        // Remembers that somebody could have done the work and was turned down
        // only by arithmetic, which is a different economy from an empty one.
        let mut priced_out = false;
        for ((employer, cohort), agreement) in &self.employment_agreements {
            if !agreement.active()
                || agreement.workers() < STARTUP_WORKERS
                || agreement.months_at_current_firm() < MINIMUM_TENURE_BEFORE_MOVE_MONTHS
                || self.is_firm_insolvent(*employer)
            {
                continue;
            }
            let Some(definition) = self.cohorts.get(cohort) else {
                continue;
            };
            if definition.region() != region
                || definition.education() < minimum
                || !self.cohort_has_recipe_skill(*cohort, recipe)
                || self.workforce_training.contains_key(cohort)
            {
                continue;
            }
            let Some(offer) = poaching_offer(agreement.wage())? else {
                continue;
            };
            if offer.minor_units() > ceiling.minor_units() {
                priced_out = true;
                continue;
            }
            // The founder takes the worker whose present employer values them
            // least, because that is the cheapest hire it can defend paying.
            let improves = match best {
                Some(current) => offer.minor_units() < current.wage.minor_units(),
                None => true,
            };
            if improves {
                best = Some(EntryRecruitment {
                    cohort: *cohort,
                    wage: offer,
                    poached_from: Some(*employer),
                    available_workers: agreement.workers(),
                });
            }
        }
        Ok(best.ok_or(if priced_out {
            crate::FirmEntryObstacle::RaiseExceedsWorkValue
        } else {
            crate::FirmEntryObstacle::NoWorkerToBeHad
        }))
    }

    /// What an idle cohort has been living on per person per month, which is the
    /// wage a founder must at least match to interest them.
    fn idle_reservation_wage(&self, cohort: CohortId) -> Result<Money, WorldError> {
        let definition = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?;
        let people = definition.people().people().max(1);
        let wage_units = definition.annual_income().minor_units()
            / i64::try_from(people)
                .map_err(|_| WorldError::ArithmeticOverflow("entry cohort population"))?
            / 12;
        // A wage is a positive quantity by the type's own domain; zero would not
        // be an agreement at all.
        Ok(Money::from_minor_units(wage_units.max(1)))
    }

    fn foundation_economics(
        &self,
        region: RegionId,
        recipe: RecipeId,
        recruitment: EntryRecruitment,
        capacity_batches: u64,
        workers: u64,
    ) -> Result<Option<FoundationEconomics>, WorldError> {
        let recipe = self
            .production_recipes
            .get(&recipe)
            .ok_or(WorldError::UnknownRecipe(recipe))?;
        let cohort = self
            .cohorts
            .get(&recruitment.cohort)
            .ok_or(WorldError::UnknownCohort(recruitment.cohort))?;
        if cohort.region() != region || cohort.people().people() == 0 {
            return Ok(None);
        }
        let wage = recruitment.wage;
        let Some(output_price) = self
            .regional_prices
            .get(&(region, recipe.output_good()))
            .copied()
        else {
            return Ok(None);
        };
        let capacity_batches = capacity_batches.max(1);
        let workers = workers.max(1);
        let batches = i64::try_from(capacity_batches)
            .map_err(|_| WorldError::ArithmeticOverflow("startup capacity batches"))?;
        let staff = i64::try_from(workers)
            .map_err(|_| WorldError::ArithmeticOverflow("startup staff count"))?;
        let output_value = quantity_value(output_price, recipe.output_per_batch())?;
        let capital_cost = Money::from_minor_units(
            output_value
                .minor_units()
                .checked_mul(CAPACITY_VALUE_MULTIPLIER)
                .and_then(|value| value.checked_mul(batches))
                .ok_or(WorldError::ArithmeticOverflow("startup capacity cost"))?
                .max(1),
        );
        let mut input_cost = Money::default();
        for input in recipe.inputs() {
            let Some(price) = self.regional_prices.get(&(region, input.good())).copied() else {
                return Ok(None);
            };
            input_cost = checked_money_add(
                input_cost,
                quantity_value(price, input.quantity_per_batch())?,
            )?;
        }
        // Every batch the works will actually run needs its own month of
        // materials. Costing one batch and building many is how a firm opens
        // already unable to buy what it must convert.
        let input_cost = Money::from_minor_units(
            input_cost
                .minor_units()
                .checked_mul(batches)
                .ok_or(WorldError::ArithmeticOverflow("startup input cost"))?,
        );
        let wage_buffer = Money::from_minor_units(
            wage.minor_units()
                .checked_mul(WORKING_CAPITAL_WAGE_MONTHS)
                .and_then(|value| value.checked_mul(staff))
                .ok_or(WorldError::ArithmeticOverflow("startup wage buffer"))?,
        );
        let working_capital = checked_money_add(input_cost, wage_buffer)?;
        Ok(Some(FoundationEconomics {
            wage,
            capital_cost,
            working_capital,
            capacity_batches,
            workers,
        }))
    }

    #[must_use]
    pub fn firm_entry_pressure(
        &self,
    ) -> &BTreeMap<(RegionId, GoodId), crate::accumulator::LeakyPressure> {
        &self.firm_entry_pressure
    }
}

fn startup_policy() -> Result<FirmPolicy, WorldError> {
    FirmPolicy::new(
        30,
        BasisPoints::new(1_000)
            .map_err(|_| WorldError::InvalidBusinessPolicy("invalid startup markup"))?,
        BasisPoints::ZERO,
        BasisPoints::HALF,
        BasisPoints::ZERO,
    )
}

pub(crate) fn quantity_value(price: Money, quantity: QuantityMilli) -> Result<Money, WorldError> {
    let value = i128::from(price.minor_units())
        .checked_mul(i128::from(quantity.get()))
        .ok_or(WorldError::ArithmeticOverflow("startup quantity value"))?
        / i128::from(QuantityMilli::SCALE);
    Ok(Money::from_minor_units(i64::try_from(value).map_err(
        |_| WorldError::ArithmeticOverflow("startup quantity value"),
    )?))
}

/// What it takes to move a worker who already has a job.
///
/// The margin is the one the labor market already requires of any competing
/// offer, shared rather than copied so that an entrant and an established rival
/// must beat the same wage by the same amount. Returns `None` when the standing
/// wage is not a positive quantity, since there is then nothing to outbid.
fn poaching_offer(current: Money) -> Result<Option<Money>, WorldError> {
    let standing = current.minor_units();
    if standing <= 0 {
        return Ok(None);
    }
    let scaled = i128::from(standing)
        .checked_mul(crate::labor::MATERIAL_OFFER_IMPROVEMENT_PERCENT)
        .ok_or(WorldError::ArithmeticOverflow("entry poaching offer"))?;
    // Rounded up: an offer that rounds down to the standing wage is not an offer.
    let offer = i64::try_from((scaled + 99) / 100)
        .map_err(|_| WorldError::ArithmeticOverflow("entry poaching offer"))?;
    if offer <= standing {
        return Ok(None);
    }
    Ok(Some(Money::from_minor_units(offer)))
}

fn checked_money_add(first: Money, second: Money) -> Result<Money, WorldError> {
    Ok(Money::from_minor_units(
        first
            .minor_units()
            .checked_add(second.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("firm founding commitment"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, AgeBand, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
        EducationLevel, EmploymentStatus, Good, HouseholdCohort, HouseholdType, NeedProfileId,
        Population, ProductionRecipe, Region, SimDate, WorldSeed,
    };

    fn entry_world(founder_cash: i64) -> World {
        let mut world = World::new(WorldSeed::new(76), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Aster").expect("country"))
            .expect("country registration");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "North March",
                    Population::new(10),
                    Money::from_minor_units(1_200),
                )
                .expect("region"),
            )
            .expect("region registration");
        world
            .register_good(Good::new(GoodId::new(1), "grain").expect("good"))
            .expect("good registration");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "survival",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile registration");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(10),
                    10,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Basic,
                    EmploymentStatus::Unemployed,
                    Money::from_minor_units(120),
                    Money::from_minor_units(0),
                    Money::from_minor_units(0),
                )
                .expect("cohort"),
            )
            .expect("cohort registration");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Iris Vale", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor registration");
        world
            .register_actor_cash(ActorId::new(1), Money::from_minor_units(founder_cash))
            .expect("founder cash");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(10),
            )
            .expect("price");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "grain workshop",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe registration");
        world.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
            QuantityMilli::new(1_000),
        );
        world
    }

    /// A region whose only qualified people already have jobs. Nobody is idle,
    /// so the sole way a new producer can open is by buying a worker away from
    /// the employer that has one.
    fn fully_employed_entry_world(incumbent_wage: i64) -> World {
        let mut world = entry_world(100);
        world
            .register_firm(
                crate::Firm::new(
                    FirmId::new(1),
                    "North March Grain Works",
                    RegionId::new(1),
                    RecipeId::new(1),
                    10,
                    1,
                    Money::from_minor_units(1_000),
                    BTreeMap::new(),
                )
                .expect("incumbent firm"),
            )
            .expect("incumbent registration");
        world.firm_production_targets.insert(FirmId::new(1), 1);
        world
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(1),
                    CohortId::new(1),
                    10,
                    Money::from_minor_units(incumbent_wage),
                )
                .expect("standing agreement"),
            )
            .expect("agreement registration");
        world
    }

    /// Runs the months a worker must spend with an employer before anybody can
    /// court them, leaving the world one review short of the entry signal.
    fn settle_tenure_before_entry(world: &mut World) {
        for _ in 0..2 {
            world
                .execute_observed_labor_matching()
                .expect("labor matching");
            assert!(
                world
                    .execute_observed_firm_entry()
                    .expect("entry review")
                    .is_empty()
            );
            world.advance_month().expect("month");
        }
        world
            .execute_observed_labor_matching()
            .expect("labor matching");
    }

    #[test]
    fn a_founder_outbids_an_incumbent_when_the_region_has_nobody_idle() {
        let mut world = fully_employed_entry_world(5);
        settle_tenure_before_entry(&mut world);
        let mut replayed = world.clone();

        let decisions = world.execute_observed_firm_entry().expect("entry review");
        WorldCommand::ExecuteObservedFirmEntry
            .apply(&mut replayed)
            .expect("replayed entry review");

        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert_eq!(decisions.len(), 1);
        let plan = decisions[0].plan;
        assert_eq!(plan.poached_from, Some(FirmId::new(1)));
        // The standing wage is five; a worker moves for the same material margin
        // any other competing offer must clear, rounded up to the minor unit.
        assert_eq!(plan.wage, Money::from_minor_units(6));
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            9,
            "the incumbent is one worker short of what it had"
        );
        let hired = &world.employment_agreements()[&(plan.firm, CohortId::new(1))];
        assert_eq!(hired.workers(), 1);
        assert_eq!(hired.wage(), Money::from_minor_units(6));
    }

    #[test]
    fn shortage_evidence_outlives_the_monthly_accounts_reset() {
        // The monthly accounts reset runs at the end of the commercial cycle
        // and firm entry reviews the region afterwards. While the reset also
        // wiped the shortfall map, every shortage was recorded and then thrown
        // away before the only party entitled to answer it could read it, so
        // the whole channel was dead in a real month and alive only in tests
        // that drove the stages by hand.
        let mut world = entry_world(1_000);
        world
            .monthly_firm_input_shortfalls
            .insert((RegionId::new(1), GoodId::new(1)), QuantityMilli::new(500));
        world.reset_monthly_firm_accounts();
        assert_eq!(
            world
                .monthly_firm_input_shortfalls
                .get(&(RegionId::new(1), GoodId::new(1)))
                .copied(),
            Some(QuantityMilli::new(500)),
            "entry reads the shortage after the reset, so the evidence must survive it"
        );
    }

    #[test]
    fn a_founder_will_not_outbid_past_what_the_work_is_worth() {
        // A batch fetches ten and consumes nothing, so a month of this work is
        // worth ten. Courting a worker paid ten would cost eleven, which the
        // output cannot repay - so the shortage goes unanswered, and it is the
        // region that pays for that, not an invisible rule.
        let mut world = fully_employed_entry_world(10);
        settle_tenure_before_entry(&mut world);

        let decisions = world.execute_observed_firm_entry().expect("entry review");

        assert!(decisions.is_empty());
        assert_eq!(world.firms().len(), 1);
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            10
        );
    }

    #[test]
    fn persistent_shortage_founds_capitalized_staffed_firm_through_shared_command() {
        let mut direct = entry_world(100);
        for _ in 0..2 {
            assert!(
                direct
                    .execute_observed_firm_entry()
                    .expect("entry review")
                    .is_empty()
            );
            direct.advance_month().expect("month");
        }
        let mut replayed = direct.clone();
        WorldCommand::ExecuteObservedFirmEntry
            .apply(&mut direct)
            .expect("third review");
        WorldCommand::ExecuteObservedFirmEntry
            .apply(&mut replayed)
            .expect("replayed third review");
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        let firm = &direct.firms()[&FirmId::new(1)];
        assert_eq!(firm.capacity_batches(), 1);
        assert_eq!(firm.workers(), 1);
        assert_eq!(firm.cash(), Money::from_minor_units(3));
        assert_eq!(
            direct.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(77)
        );
        assert_eq!(
            direct.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            1
        );
        assert_eq!(direct.firm_production_targets()[&FirmId::new(1)], 1);
        assert!(direct.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::FirmFoundedFromOpportunity {
                founder,
                firm,
                capital_cost,
                working_capital,
                pressure_months: 3,
                ..
            } if *founder == ActorId::new(1)
                && *firm == FirmId::new(1)
                && *capital_cost == Money::from_minor_units(20)
                && *working_capital == Money::from_minor_units(3)
        )));
    }

    #[test]
    fn founding_capital_is_paid_to_the_households_that_build_the_works() {
        let mut world = entry_world(100);
        for _ in 0..2 {
            world.execute_observed_firm_entry().expect("entry review");
            world.advance_month().expect("month");
        }
        world
            .execute_observed_firm_entry()
            .expect("third entry review");

        let paid: i64 = world
            .events()
            .events()
            .iter()
            .filter_map(|envelope| match envelope.event() {
                DomainEvent::CapitalOutlayDistributed { firm, amount, .. }
                    if *firm == FirmId::new(1) =>
                {
                    Some(amount.minor_units())
                }
                _ => None,
            })
            .sum();

        assert_eq!(
            paid, 20,
            "every minor unit of founding capital must reach the builders instead of vanishing"
        );
    }

    #[test]
    fn persistent_shortage_does_not_create_unfunded_firm() {
        let mut world = entry_world(22);
        for month in 0..3 {
            assert!(
                world
                    .execute_observed_firm_entry()
                    .expect("entry review")
                    .is_empty()
            );
            if month < 2 {
                world.advance_month().expect("month");
            }
        }
        assert!(world.firms().is_empty());
        assert_eq!(
            world.firm_entry_pressure()[&(RegionId::new(1), GoodId::new(1))].months(),
            3
        );
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(22)
        );
    }

    /// A hunger of five batches is not answered by one.
    ///
    /// Entry used to open every works at a single batch whatever the region was
    /// short of, so a shortage of half a million milli-units was met by four
    /// thousand and the queue never cleared. Size now follows the evidence.
    #[test]
    fn a_works_opens_at_the_size_of_the_hunger_it_answers() {
        let mut world = entry_world(200);
        world.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
            QuantityMilli::new(5_000),
        );
        for _ in 0..2 {
            world.execute_observed_firm_entry().expect("entry review");
            world.advance_month().expect("month");
        }
        world.execute_observed_firm_entry().expect("third review");
        let firm = &world.firms()[&FirmId::new(1)];
        assert_eq!(
            firm.capacity_batches(),
            5,
            "a works must be built for the demand that called it, not for a token batch"
        );
        assert_eq!(firm.workers(), 5);
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            5
        );
        assert_eq!(world.firm_production_targets()[&FirmId::new(1)], 5);
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(85),
            "five batches of capacity and five wage buffers must be paid for in full"
        );
    }

    /// The other side of the same rule: ambition is bounded by the wallet.
    #[test]
    fn a_thin_wallet_opens_a_smaller_works_and_not_a_free_one() {
        let mut world = entry_world(60);
        world.unmet_demand.insert(
            (CohortId::new(1), GoodId::new(1), NeedTier::Survival),
            QuantityMilli::new(5_000),
        );
        for _ in 0..2 {
            world.execute_observed_firm_entry().expect("entry review");
            world.advance_month().expect("month");
        }
        world.execute_observed_firm_entry().expect("third review");
        let firm = &world.firms()[&FirmId::new(1)];
        assert_eq!(
            firm.capacity_batches(),
            2,
            "a founder cannot build capacity it has no money to pay for"
        );
        assert_eq!(firm.workers(), 2);
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(14)
        );
    }
}
