use crate::{
    DemandIntent, FirmMarketOfferPlan, HouseholdCashflow, MarketClearing, MarketOrder,
    PayrollRecord, ProductionPlan, SimDate, World, WorldError, clear_local_market,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonthlyCommercialCycleResult {
    pub production_plans: Vec<ProductionPlan>,
    pub offer_plans: Vec<FirmMarketOfferPlan>,
    pub demand_intents: Vec<DemandIntent>,
    pub clearing: MarketClearing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonthlyEconomicCycleResult {
    pub completed_date: SimDate,
    pub next_date: SimDate,
    pub payroll: Vec<PayrollRecord>,
    pub household_cashflows: Vec<HouseholdCashflow>,
    pub commercial: MonthlyCommercialCycleResult,
}

impl World {
    /// Executes one atomic commercial month from physical production through observation capture.
    /// # Errors
    /// Returns the first planning or settlement error without changing authoritative state.
    pub fn execute_monthly_commercial_cycle(
        &mut self,
    ) -> Result<MonthlyCommercialCycleResult, WorldError> {
        if self.last_commercial_cycle_date == Some(self.date) {
            return Err(WorldError::CommercialCycleAlreadyExecuted(self.date));
        }
        let mut next = self.clone();
        let production_plans = next.execute_monthly_production()?;
        let offer_plans = next.plan_firm_market_offers()?;
        let offers: Vec<_> = offer_plans
            .iter()
            .filter_map(|plan| plan.market_offer())
            .collect();
        let demand_intents = next.plan_monthly_household_demand()?;
        let orders: Vec<_> = demand_intents
            .iter()
            .map(|intent| {
                let cohort = &next.cohorts[&intent.cohort()];
                MarketOrder {
                    buyer: intent.cohort(),
                    tier: intent.tier(),
                    region: cohort.region(),
                    good: intent.good(),
                    quantity: intent.desired(),
                    max_spend: intent.reserved_spend(),
                }
            })
            .collect();
        let clearing = clear_local_market(&orders, &offers)?;
        next.settle_local_market(&clearing)?;
        let firms: Vec<_> = next.firms.keys().copied().collect();
        for firm in firms {
            next.capture_monthly_firm_observation(firm)?;
        }
        next.reset_monthly_firm_accounts();
        next.last_commercial_cycle_date = Some(next.date);
        let production_batches = production_plans.iter().try_fold(0_u64, |sum, plan| {
            sum.checked_add(plan.batches())
                .ok_or(WorldError::ArithmeticOverflow(
                    "commercial cycle production",
                ))
        })?;
        next.events.append(
            next.date,
            crate::DomainEvent::MonthlyCommercialCycleCompleted {
                production_batches,
                seller_offers: u64::try_from(offers.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("commercial cycle offers"))?,
                market_fills: u64::try_from(clearing.fills.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("commercial cycle fills"))?,
                firms_observed: u64::try_from(next.firms.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("commercial cycle firms"))?,
            },
        );
        let result = MonthlyCommercialCycleResult {
            production_plans,
            offer_plans,
            demand_intents,
            clearing,
        };
        *self = next;
        Ok(result)
    }
}

impl World {
    /// Advances the simplified calendar by one month without running economic stages.
    /// # Errors
    /// Returns an error if crossing the calendar boundary overflows the year.
    pub fn advance_month(&mut self) -> Result<(), WorldError> {
        self.date.advance_one_month()?;
        self.events.append(
            self.date,
            crate::DomainEvent::MonthAdvanced { date: self.date },
        );
        Ok(())
    }

    /// Atomically executes payroll, household cashflows, commerce, social observation, and time.
    /// # Errors
    /// Returns the first stage error without changing authoritative state.
    pub fn execute_monthly_economic_cycle(
        &mut self,
    ) -> Result<MonthlyEconomicCycleResult, WorldError> {
        let mut next = self.clone();
        let completed_date = next.date;
        let payroll = next.execute_monthly_payroll()?;
        let household_cashflows = next.execute_monthly_household_cashflows()?;
        let commercial = next.execute_monthly_commercial_cycle()?;
        next.update_monthly_cohort_experience()?;
        next.derive_monthly_social_stress()?;
        next.accumulate_monthly_social_stress()?;
        next.events.append(
            completed_date,
            crate::DomainEvent::MonthlyEconomicCycleCompleted {
                payroll_records: u64::try_from(payroll.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("monthly payroll records"))?,
                household_cashflows: u64::try_from(household_cashflows.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("monthly household cashflows"))?,
                market_fills: u64::try_from(commercial.clearing.fills.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("monthly market fills"))?,
            },
        );
        next.advance_month()?;
        let result = MonthlyEconomicCycleResult {
            completed_date,
            next_date: next.date,
            payroll,
            household_cashflows,
            commercial,
        };
        *self = next;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, ActorId, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget,
        CorporateRole, Country, CountryId, DemandBasis, EducationLevel, EmploymentStatus, Firm,
        FirmAppointment, FirmId, FirmPolicy, Good, GoodId, HouseholdCohort, HouseholdType, Money,
        NeedProfileId, NeedTier, OwnershipStake, Population, ProductionRecipe, QuantityMilli,
        RecipeId, Region, RegionId, SimDate, WorldCommand, WorldSeed,
    };
    use std::collections::BTreeMap;

    #[allow(clippy::too_many_lines)]
    fn commercial_world(with_price: bool) -> World {
        let mut world = World::new(WorldSeed::new(9), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Households",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(1),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Owner", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Farm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    2,
                    Money::default(),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(1),
                    1,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::from_minor_units(120),
                    Money::from_minor_units(100),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(1),
                ActorId::new(1),
                BasisPoints::new(6_000).expect("rights"),
                BasisPoints::new(6_000).expect("rights"),
            ))
            .expect("ownership");
        world
            .register_firm_appointment(FirmAppointment::new(
                FirmId::new(1),
                ActorId::new(1),
                CorporateRole::OperationsManager,
            ))
            .expect("appointment");
        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(1),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                    BasisPoints::new(0).expect("allocation"),
                )
                .expect("policy"),
            )
            .expect("policy");
        world
            .set_firm_production_target(ActorId::new(1), FirmId::new(1), 1)
            .expect("target");
        if with_price {
            world
                .set_regional_price(
                    RegionId::new(1),
                    GoodId::new(1),
                    Money::from_minor_units(10),
                )
                .expect("price");
        }
        world
    }

    #[test]
    fn cycle_is_replayable_and_closes_the_observation_loop() {
        let mut direct = commercial_world(true);
        let mut replayed = direct.clone();
        let result = direct
            .execute_monthly_commercial_cycle()
            .expect("commercial cycle");
        WorldCommand::ExecuteMonthlyCommercialCycle
            .apply(&mut replayed)
            .expect("replayed cycle");

        assert_eq!(result.production_plans[0].batches(), 1);
        assert_eq!(result.offer_plans[0].offered, QuantityMilli::new(1_000));
        assert_eq!(result.clearing.fills.len(), 1);
        assert_eq!(direct.firm_operating_history()[&FirmId::new(1)].len(), 1);
        assert_eq!(
            direct.firm_operating_history()[&FirmId::new(1)][0].sales_revenue(),
            Money::from_minor_units(10)
        );
        assert!(direct.firm_monthly_accounts().is_empty());
        assert!(direct.monthly_firm_market_outcomes().is_empty());
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn economic_cycle_is_replayable_and_advances_exactly_one_month() {
        let mut direct = commercial_world(true);
        let mut replayed = direct.clone();
        let result = direct
            .execute_monthly_economic_cycle()
            .expect("economic cycle");
        WorldCommand::ExecuteMonthlyEconomicCycle
            .apply(&mut replayed)
            .expect("replayed economic cycle");

        assert_eq!(result.completed_date, SimDate::new(2025, 1).expect("date"));
        assert_eq!(result.next_date, SimDate::new(2025, 32).expect("date"));
        assert_eq!(result.household_cashflows.len(), 1);
        assert_eq!(result.commercial.clearing.fills.len(), 1);
        assert_eq!(direct.date(), SimDate::new(2025, 32).expect("date"));
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn economic_year_runs_twelve_months_then_one_annual_closure() {
        let mut direct = commercial_world(true);
        let mut replayed = direct.clone();
        let result = direct.advance_economic_year().expect("economic year");
        WorldCommand::AdvanceEconomicYear
            .apply(&mut replayed)
            .expect("replayed economic year");

        assert_eq!(result.closed_year, 2025);
        assert_eq!(result.months.len(), 12);
        assert_eq!(direct.date(), SimDate::new(2026, 1).expect("date"));
        assert_eq!(direct.firm_operating_history()[&FirmId::new(1)].len(), 12);
        let monthly_cycles = direct
            .events()
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.event(),
                    crate::DomainEvent::MonthlyEconomicCycleCompleted { .. }
                )
            })
            .count();
        assert_eq!(monthly_cycles, 12);
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn fifty_economic_years_keep_bounded_firm_memory() {
        let mut world = commercial_world(true);
        world
            .advance_economic_years(50)
            .expect("fifty economic years");
        assert_eq!(world.date(), SimDate::new(2075, 1).expect("date"));
        assert_eq!(world.firm_operating_history()[&FirmId::new(1)].len(), 12);
        let monthly_cycles = world
            .events()
            .events()
            .iter()
            .filter(|event| {
                matches!(
                    event.event(),
                    crate::DomainEvent::MonthlyEconomicCycleCompleted { .. }
                )
            })
            .count();
        let annual_closures = world
            .events()
            .events()
            .iter()
            .filter(|event| matches!(event.event(), crate::DomainEvent::YearAdvanced { .. }))
            .count();
        assert_eq!(monthly_cycles, 600);
        assert_eq!(annual_closures, 50);
    }

    #[test]
    fn repeated_monthly_stage_is_rejected_without_mutation() {
        let mut world = commercial_world(true);
        world.execute_monthly_payroll().expect("first payroll");
        let after_first = world.clone();
        assert!(matches!(
            world.execute_monthly_payroll(),
            Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "payroll",
                ..
            })
        ));
        assert_eq!(world, after_first);
        assert!(matches!(
            world.execute_monthly_economic_cycle(),
            Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "payroll",
                ..
            })
        ));
        assert_eq!(world, after_first);
    }

    #[test]
    fn cycle_failure_and_duplicate_execution_are_atomic() {
        let mut missing_price = commercial_world(false);
        let before = missing_price.clone();
        assert!(matches!(
            missing_price.execute_monthly_commercial_cycle(),
            Err(WorldError::MissingRegionalPrice { .. })
        ));
        assert_eq!(missing_price, before);

        let mut completed = commercial_world(true);
        completed
            .execute_monthly_commercial_cycle()
            .expect("first cycle");
        let after_first = completed.clone();
        assert!(matches!(
            completed.execute_monthly_commercial_cycle(),
            Err(WorldError::CommercialCycleAlreadyExecuted(_))
        ));
        assert_eq!(completed, after_first);
    }
}
