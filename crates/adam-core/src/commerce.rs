use crate::{
    DemandIntent, FirmMarketOfferPlan, MarketClearing, MarketOrder, ProductionPlan, World,
    WorldError, clear_local_market,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonthlyCommercialCycleResult {
    pub production_plans: Vec<ProductionPlan>,
    pub offer_plans: Vec<FirmMarketOfferPlan>,
    pub demand_intents: Vec<DemandIntent>,
    pub clearing: MarketClearing,
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
