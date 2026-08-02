use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ActorId, BasisPoints, CohortId, CorporateRole, DomainEvent, EmploymentAgreement, Firm,
    FirmAppointment, FirmId, FirmPolicy, GoodId, Money, NeedTier, OwnershipStake, QuantityMilli,
    RecipeId, RegionId, World, WorldCommand, WorldError,
};

const ENTRY_SIGNAL_MONTHS: u8 = 3;
const STARTUP_CAPACITY_BATCHES: u64 = 1;
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
    pub wage: Money,
    pub capital_cost: Money,
    pub working_capital: Money,
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

#[derive(Clone, Copy)]
struct FoundationEconomics {
    wage: Money,
    capital_cost: Money,
    working_capital: Money,
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
        // Entry answers two kinds of hunger, not one. A household that cannot
        // buy bread and a bakery that cannot buy grain are both a region
        // asking for a good it does not make, and the second was invisible
        // until now: an intermediate good is never anybody's survival need, so
        // a permanently starved input channel could shout its price and its
        // shortfall every month forever without a single farm being founded.
        let mut unmet = self.current_regional_survival_shortages()?;
        for (&(region, good), &quantity) in &self.monthly_firm_input_shortfalls {
            if quantity.get() == 0 {
                continue;
            }
            let row = unmet.entry((region, good)).or_default();
            *row = QuantityMilli::new(row.get().checked_add(quantity.get()).ok_or(
                WorldError::ArithmeticOverflow("regional entry shortage evidence"),
            )?);
        }
        let unmet = unmet;
        let mut keys: BTreeSet<_> = self.firm_entry_pressure.keys().copied().collect();
        keys.extend(unmet.keys().copied());
        let mut next = self.clone();
        let mut decisions = Vec::new();
        for (region, good) in keys {
            let quantity = unmet.get(&(region, good)).copied().unwrap_or_default();
            let pressure_months = {
                let pressure = next.firm_entry_pressure.entry((region, good)).or_default();
                *pressure = update_entry_pressure(*pressure, quantity.get() > 0);
                *pressure
            };
            let plan = if pressure_months >= ENTRY_SIGNAL_MONTHS {
                next.plan_firm_entry(region, good, pressure_months)?
            } else {
                None
            };
            if let Some(plan) = plan {
                WorldCommand::FoundFirm(plan).apply(&mut next)?;
                next.firm_entry_pressure.insert((region, good), 0);
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
        let economics = self
            .foundation_economics(plan.region, plan.recipe, plan.cohort)?
            .ok_or(WorldError::InvalidProduction(
                "firm founding requires observed prices and an unemployed local worker",
            ))?;
        if plan.wage != economics.wage
            || plan.capital_cost != economics.capital_cost
            || plan.working_capital != economics.working_capital
            || plan.pressure_months == 0
        {
            return Err(WorldError::InvalidProduction(
                "firm founding plan does not match current regional economics",
            ));
        }
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
            STARTUP_WORKERS,
            STARTUP_CAPACITY_BATCHES,
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
        next.register_employment_agreement(EmploymentAgreement::new(
            plan.firm,
            plan.cohort,
            STARTUP_WORKERS,
            plan.wage,
        )?)?;
        next.set_firm_production_target(plan.founder, plan.firm, STARTUP_CAPACITY_BATCHES)?;
        next.record_firm_founded_event(plan, recipe.output_good());
        *self = next;
        Ok(())
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

    fn current_regional_survival_shortages(
        &self,
    ) -> Result<BTreeMap<(RegionId, GoodId), QuantityMilli>, WorldError> {
        let mut shortages: BTreeMap<(RegionId, GoodId), QuantityMilli> = BTreeMap::new();
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
            *row = QuantityMilli::new(
                row.get()
                    .checked_add(quantity.get())
                    .ok_or(WorldError::ArithmeticOverflow("regional survival shortage"))?,
            );
        }
        Ok(shortages)
    }

    fn plan_firm_entry(
        &self,
        region: RegionId,
        good: GoodId,
        pressure_months: u8,
    ) -> Result<Option<FirmFoundingPlan>, WorldError> {
        let Some(recipe) = self
            .production_recipes
            .values()
            .filter(|recipe| recipe.output_good() == good)
            .map(crate::ProductionRecipe::id)
            .next()
        else {
            return Ok(None);
        };
        let Some(cohort) = self.select_entry_worker_cohort(region, recipe) else {
            return Ok(None);
        };
        let Some(economics) = self.foundation_economics(region, recipe, cohort)? else {
            return Ok(None);
        };
        let commitment = checked_money_add(economics.capital_cost, economics.working_capital)?;
        let mut founder = None;
        let mut founder_cash = Money::default();
        for (actor, definition) in self.actors() {
            if definition.home_region() != region {
                continue;
            }
            let cash = self.actor_cash.get(actor).copied().unwrap_or_default();
            if cash.minor_units() < commitment.minor_units() {
                continue;
            }
            if founder.is_none() || cash.minor_units() > founder_cash.minor_units() {
                founder = Some(*actor);
                founder_cash = cash;
            }
        }
        let Some(founder) = founder else {
            return Ok(None);
        };
        let firm = FirmId::new(self.firms.keys().next_back().map_or(Ok(1), |id| {
            id.get()
                .checked_add(1)
                .ok_or(WorldError::ArithmeticOverflow("autonomous firm identifier"))
        })?);
        Ok(Some(FirmFoundingPlan {
            founder,
            firm,
            region,
            recipe,
            cohort,
            wage: economics.wage,
            capital_cost: economics.capital_cost,
            working_capital: economics.working_capital,
            pressure_months,
        }))
    }

    fn select_entry_worker_cohort(&self, region: RegionId, recipe: RecipeId) -> Option<CohortId> {
        // Entry predates the competitive labor market. Recipes without an explicit
        // market profile retain the historical Basic qualification floor here;
        // only post-entry vacancy matching is opt-in.
        let minimum = self
            .recipe_minimum_education
            .get(&recipe)
            .copied()
            .unwrap_or(crate::EducationLevel::Basic);
        let mut selected = None;
        let mut selected_available = 0_u64;
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
            if available > selected_available {
                selected = Some(*cohort);
                selected_available = available;
            }
        }
        selected.filter(|_| selected_available >= STARTUP_WORKERS)
    }

    fn foundation_economics(
        &self,
        region: RegionId,
        recipe: RecipeId,
        cohort: CohortId,
    ) -> Result<Option<FoundationEconomics>, WorldError> {
        let recipe = self
            .production_recipes
            .get(&recipe)
            .ok_or(WorldError::UnknownRecipe(recipe))?;
        let cohort = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?;
        if cohort.region() != region || cohort.employment() != crate::EmploymentStatus::Unemployed {
            return Ok(None);
        }
        let people = cohort.people().people();
        if people == 0 {
            return Ok(None);
        }
        let wage_units = cohort.annual_income().minor_units()
            / i64::try_from(people)
                .map_err(|_| WorldError::ArithmeticOverflow("entry cohort population"))?
            / 12;
        let wage = Money::from_minor_units(wage_units.max(1));
        let Some(output_price) = self
            .regional_prices
            .get(&(region, recipe.output_good()))
            .copied()
        else {
            return Ok(None);
        };
        let output_value = quantity_value(output_price, recipe.output_per_batch())?;
        let capital_cost = Money::from_minor_units(
            output_value
                .minor_units()
                .checked_mul(CAPACITY_VALUE_MULTIPLIER)
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
        let wage_buffer = Money::from_minor_units(
            wage.minor_units()
                .checked_mul(WORKING_CAPITAL_WAGE_MONTHS)
                .ok_or(WorldError::ArithmeticOverflow("startup wage buffer"))?,
        );
        let working_capital = checked_money_add(input_cost, wage_buffer)?;
        Ok(Some(FoundationEconomics {
            wage,
            capital_cost,
            working_capital,
        }))
    }

    #[must_use]
    pub fn firm_entry_pressure(&self) -> &BTreeMap<(RegionId, GoodId), u8> {
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

fn checked_money_add(first: Money, second: Money) -> Result<Money, WorldError> {
    Ok(Money::from_minor_units(
        first
            .minor_units()
            .checked_add(second.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("firm founding commitment"))?,
    ))
}

const fn update_entry_pressure(current: u8, active: bool) -> u8 {
    if active { current.saturating_add(1) } else { 0 }
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
            world.firm_entry_pressure()[&(RegionId::new(1), GoodId::new(1))],
            3
        );
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(22)
        );
    }
}
