use crate::{
    ActorId, CorporateAction, DomainEvent, FirmId, InvestmentProject, InvestmentStatus, Money,
    ProductionAdjustmentProposal, ProjectId, RegionId, World, WorldCommand, WorldError,
};

/// Technological time to erect new productive capacity. Construction is not a
/// modelled industry yet, so the build time is a fixed physical lag rather than
/// an outcome of a builder's own order book.
const CONSTRUCTION_MONTHS: u32 = 3;

/// Installed capacity costs twice the value of what a batch produces, the same
/// valuation a founder already faces when building a new works in `firm_entry`.
const CAPACITY_VALUE_MULTIPLIER: i64 = 2;

/// One authorized capacity-expansion decision made from observed scarcity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityInvestmentDecision {
    pub actor: ActorId,
    pub firm: FirmId,
    pub project: ProjectId,
    pub budget: Money,
    pub capacity_batches: u64,
}

#[derive(Clone, Copy)]
struct CapacityExpansionPlan {
    actor: ActorId,
    firm: FirmId,
    region: RegionId,
    project: ProjectId,
    budget: Money,
    capacity_batches: u64,
}

impl World {
    /// Advances open construction and lets sold-out firms buy more capacity.
    ///
    /// This is the supply side of the price signal. A firm that keeps selling
    /// out while demand goes unfilled, and that expects a positive operating
    /// margin, commits its own cash to installed capacity. Nothing bounds the
    /// expansion except what the firm can pay for and how much demand it
    /// actually observed going away empty.
    ///
    /// Every decision crosses the same replayable command boundary a player
    /// would use, and the construction budget is paid to local households as
    /// building income, so investment moves money instead of destroying it.
    ///
    /// # Errors
    /// Returns an error on duplicate monthly execution, or on a failed
    /// commitment, launch, or payout, without partially applying any decision.
    pub fn execute_observed_capacity_investment(
        &mut self,
    ) -> Result<Vec<CapacityInvestmentDecision>, WorldError> {
        if self.last_capacity_investment_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed capacity investment",
                date: self.date,
            });
        }
        let mut next = self.clone();

        let building: Vec<ProjectId> = next
            .investment_projects
            .iter()
            .filter(|(_, project)| {
                matches!(
                    project.status(),
                    InvestmentStatus::Planned | InvestmentStatus::Building
                )
            })
            .map(|(id, _)| *id)
            .collect();
        for project in building {
            WorldCommand::AdvanceInvestmentProject(project).apply(&mut next)?;
        }

        let proposals = next.plan_observed_production_adjustments()?;
        let mut decisions = Vec::new();
        for proposal in &proposals {
            let Some(plan) = next.plan_capacity_expansion(proposal)? else {
                continue;
            };
            WorldCommand::CommitFirmInvestment {
                actor: plan.actor,
                firm: plan.firm,
                amount: plan.budget,
            }
            .apply(&mut next)?;
            WorldCommand::LaunchInvestmentProject(InvestmentProject::new(
                plan.project,
                plan.firm,
                plan.region,
                plan.budget,
                CONSTRUCTION_MONTHS,
                plan.capacity_batches,
            )?)
            .apply(&mut next)?;
            // The builders are paid when the works are ordered: committed cash
            // leaves the firm and arrives as local building income. Without
            // this the money would sit outside every balance in the world.
            next.distribute_capital_outlay(plan.firm, plan.region, plan.budget)?;
            decisions.push(CapacityInvestmentDecision {
                actor: plan.actor,
                firm: plan.firm,
                project: plan.project,
                budget: plan.budget,
                capacity_batches: plan.capacity_batches,
            });
        }

        next.last_capacity_investment_date = Some(next.date);
        next.events.append(
            next.date,
            DomainEvent::ObservedCapacityInvestmentCompleted {
                firms_reviewed: u64::try_from(proposals.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("capacity firms reviewed"))?,
                projects_launched: u64::try_from(decisions.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("capacity projects launched"))?,
            },
        );
        *self = next;
        Ok(decisions)
    }

    /// Sizes one firm's expansion against observed unfilled demand and its own cash.
    fn plan_capacity_expansion(
        &self,
        proposal: &ProductionAdjustmentProposal,
    ) -> Result<Option<CapacityExpansionPlan>, WorldError> {
        let firm_id = proposal.firm;
        if self.is_firm_insolvent(firm_id) {
            return Ok(None);
        }
        // Scarcity must be the firm's own observed experience, not a general
        // impression of the market: it ran out more often than it was left
        // holding stock, and it expects to earn money on the extra units.
        if proposal.stockout_observations <= proposal.unsold_observations {
            return Ok(None);
        }
        if proposal
            .expected_operating_cash_margin
            .is_none_or(|margin| margin.minor_units() <= 0)
        {
            return Ok(None);
        }
        let firm = self
            .firms
            .get(&firm_id)
            .ok_or(WorldError::UnknownFirm(firm_id))?;
        // Only a firm whose demand already absorbs everything it can physically
        // make has a reason to buy more capacity.
        if proposal.market_demand_ceiling_batches < firm.capacity_batches() {
            return Ok(None);
        }
        if self.investment_projects.values().any(|project| {
            project.firm() == firm_id
                && matches!(
                    project.status(),
                    InvestmentStatus::Planned | InvestmentStatus::Building
                )
        }) {
            return Ok(None);
        }
        let recipe = self
            .production_recipes
            .get(&firm.recipe())
            .ok_or(WorldError::UnknownRecipe(firm.recipe()))?;
        let output_per_batch = recipe.output_per_batch().get();
        if output_per_batch == 0 {
            return Ok(None);
        }
        let wanted = proposal
            .average_unmet_market_demand
            .get()
            .div_ceil(output_per_batch);
        if wanted == 0 {
            return Ok(None);
        }
        let Some(price) = self
            .regional_prices
            .get(&(firm.region(), recipe.output_good()))
            .copied()
        else {
            return Ok(None);
        };
        let batch_cost = crate::firm_entry::quantity_value(price, recipe.output_per_batch())?
            .minor_units()
            .checked_mul(CAPACITY_VALUE_MULTIPLIER)
            .ok_or(WorldError::ArithmeticOverflow("capacity batch cost"))?
            .max(1);
        // The only thing limiting the expansion is the firm's own money. It
        // cannot order works it cannot pay the builders for.
        let affordable = u64::try_from(firm.cash().minor_units() / batch_cost).unwrap_or_default();
        let capacity_batches = wanted.min(affordable);
        if capacity_batches == 0 {
            return Ok(None);
        }
        let budget = i64::try_from(capacity_batches)
            .ok()
            .and_then(|batches| batch_cost.checked_mul(batches))
            .ok_or(WorldError::ArithmeticOverflow("capacity project budget"))?;
        let Some(actor) = self.investment_actor(firm_id) else {
            return Ok(None);
        };
        let project = ProjectId::new(self.investment_projects.keys().next_back().map_or(
            Ok(1),
            |id| {
                id.get()
                    .checked_add(1)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "capacity project identifier",
                    ))
            },
        )?);
        Ok(Some(CapacityExpansionPlan {
            actor,
            firm: firm_id,
            region: firm.region(),
            project,
            budget: Money::from_minor_units(budget),
            capacity_batches,
        }))
    }

    fn investment_actor(&self, firm: FirmId) -> Option<ActorId> {
        self.actors().keys().copied().find(|actor| {
            self.can_perform_corporate_action(*actor, firm, CorporateAction::ProposeMajorInvestment)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
        CorporateRole, Country, CountryId, DemandBasis, DomainEvent, EducationLevel,
        EmploymentAgreement, EmploymentStatus, Firm, FirmAppointment, FirmId, FirmPolicy, Good,
        GoodId, HouseholdCohort, HouseholdType, Money, NeedProfileId, NeedTier, OwnershipStake,
        Population, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, World,
        WorldSeed,
    };

    const BAKERY: u32 = 1;
    const BREAD: u32 = 1;

    /// A single profitable bakery facing far more hunger than it can bake.
    #[allow(clippy::too_many_lines)]
    fn hungry_market_world() -> World {
        let mut world = World::new(WorldSeed::new(7), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register country");
        world
            .register_good(Good::new(GoodId::new(BREAD), "Bread").expect("good"))
            .expect("register bread");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Households",
                    vec![ConsumptionTarget::new(
                        GoodId::new(BREAD),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("register profile");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(10),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("register region");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Owner", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("register actor");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(BAKERY),
                    "Bread recipe",
                    GoodId::new(BREAD),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("bread recipe"),
            )
            .expect("register bread recipe");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(BAKERY),
                    "Bakery",
                    RegionId::new(1),
                    RecipeId::new(BAKERY),
                    1,
                    1,
                    Money::from_minor_units(10_000),
                    BTreeMap::new(),
                )
                .expect("bakery"),
            )
            .expect("register bakery");
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
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::default(),
                    Money::from_minor_units(100_000),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("register cohort");
        world
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(BAKERY),
                ActorId::new(1),
                BasisPoints::new(6_000).expect("rights"),
                BasisPoints::new(6_000).expect("rights"),
            ))
            .expect("ownership");
        world
            .register_firm_appointment(FirmAppointment::new(
                FirmId::new(BAKERY),
                ActorId::new(1),
                CorporateRole::ChiefExecutive,
            ))
            .expect("appointment");
        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(BAKERY),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                )
                .expect("policy"),
            )
            .expect("set policy");
        world
            .set_firm_production_target(ActorId::new(1), FirmId::new(BAKERY), 1)
            .expect("target");
        world
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(BAKERY),
                    CohortId::new(1),
                    1,
                    Money::from_minor_units(10),
                )
                .expect("agreement"),
            )
            .expect("register agreement");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(BREAD),
                Money::from_minor_units(100),
            )
            .expect("bread price");
        world
    }

    #[test]
    fn a_sold_out_profitable_firm_buys_capacity_and_pays_the_builders() {
        let mut world = hungry_market_world();
        let opening_capacity = world.firms()[&FirmId::new(BAKERY)].capacity_batches();

        world.advance_economic_year().expect("economic year");

        let proposals = world
            .plan_observed_production_adjustments()
            .expect("proposals");
        let launched: u64 = world
            .events()
            .events()
            .iter()
            .filter_map(|envelope| match envelope.event() {
                DomainEvent::ObservedCapacityInvestmentCompleted {
                    projects_launched, ..
                } => Some(*projects_launched),
                _ => None,
            })
            .sum();
        assert!(
            launched > 0,
            "a sold-out firm earning money on every batch must order more capacity: {proposals:?}"
        );

        let capacity = world.firms()[&FirmId::new(BAKERY)].capacity_batches();
        assert!(
            capacity > opening_capacity,
            "finished construction must raise installed capacity: {opening_capacity} -> {capacity}"
        );

        let building_income: i64 = world
            .events()
            .events()
            .iter()
            .filter_map(|envelope| match envelope.event() {
                DomainEvent::CapitalOutlayDistributed { amount, .. } => Some(amount.minor_units()),
                _ => None,
            })
            .sum();
        assert!(
            building_income > 0,
            "the households that build the works must be paid for it"
        );
    }
}
