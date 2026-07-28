use crate::{
    ActorId, CorporateAction, CorporateRole, DomainEvent, FirmId, World, WorldCommand, WorldError,
};

const MANAGEMENT_FORECAST_HORIZON_MONTHS: u16 = 3;

/// One authorized production-target decision made from observed operating history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmManagementDecision {
    pub actor: ActorId,
    pub firm: FirmId,
    pub previous_batches: u64,
    pub target_batches: u64,
}

impl World {
    /// Reviews observed firm history and applies authorized production advice for the next month.
    ///
    /// Operations managers are preferred, followed by chief executives and strict-majority owners.
    /// Every target change crosses the same replayable command boundary used by a player.
    ///
    /// # Errors
    /// Returns an error on duplicate monthly execution or failed expectation/target transition without
    /// partially applying any firm's decisions.
    pub fn execute_observed_firm_management(
        &mut self,
    ) -> Result<Vec<FirmManagementDecision>, WorldError> {
        if self.last_firm_management_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed firm management",
                date: self.date,
            });
        }
        let managed: Vec<_> = self
            .firms
            .keys()
            .filter(|firm| !self.is_firm_insolvent(**firm))
            .filter_map(|firm| {
                let observed = self
                    .firm_operating_history
                    .get(firm)
                    .is_some_and(|history| !history.is_empty());
                observed
                    .then(|| self.operations_actor(*firm).map(|actor| (*firm, actor)))
                    .flatten()
            })
            .collect();
        let mut next = self.clone();
        for (firm, _) in &managed {
            next.derive_firm_expectations_from_observations(
                *firm,
                MANAGEMENT_FORECAST_HORIZON_MONTHS,
            )?;
        }
        let proposals = next.plan_observed_production_adjustments()?;
        let mut decisions = Vec::new();
        for (firm, actor) in managed {
            let Some(proposal) = proposals.iter().find(|proposal| proposal.firm == firm) else {
                continue;
            };
            let previous = next
                .firm_production_targets
                .get(&firm)
                .copied()
                .unwrap_or(next.firms[&firm].capacity_batches());
            let target = next.bound_target_for_procurement_shortfall(
                firm,
                previous,
                proposal.advisory_batches,
            );
            if previous == target {
                continue;
            }
            WorldCommand::SetFirmProductionTarget {
                actor,
                firm,
                batches: target,
            }
            .apply(&mut next)?;
            decisions.push(FirmManagementDecision {
                actor,
                firm,
                previous_batches: previous,
                target_batches: target,
            });
        }
        next.last_firm_management_date = Some(next.date);
        next.events.append(
            next.date,
            DomainEvent::ObservedFirmManagementCompleted {
                firms_reviewed: u64::try_from(proposals.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("managed firms reviewed"))?,
                targets_changed: u64::try_from(decisions.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("management decisions"))?,
            },
        );
        *self = next;
        Ok(decisions)
    }

    /// A current input shortfall forbids expansion and limits contraction
    /// to one batch, with a one-batch floor for an already active target. The
    /// floor keeps next month's procurement order alive instead of trapping the
    /// firm at zero after a temporary logistics or supplier disruption.
    fn bound_target_for_procurement_shortfall(
        &self,
        firm: FirmId,
        previous: u64,
        advisory: u64,
    ) -> u64 {
        let shortfall = self.events.events().iter().rev().any(|envelope| {
            envelope.date() == self.date
                && matches!(
                    envelope.event(),
                    DomainEvent::FirmProcurementShortfall { buyer, .. }
                        | DomainEvent::FirmProcurementRouteCapacityShortfall { buyer, .. }
                        if *buyer == firm
                )
        });
        if !shortfall || previous == 0 {
            return advisory;
        }
        let floor = previous.saturating_sub(1).max(1);
        advisory.clamp(floor, previous)
    }

    pub(crate) fn operations_actor(&self, firm: FirmId) -> Option<ActorId> {
        for role in [
            CorporateRole::OperationsManager,
            CorporateRole::ChiefExecutive,
        ] {
            if let Some((_, actor, _)) =
                self.firm_appointments
                    .keys()
                    .find(|(candidate_firm, _, candidate_role)| {
                        *candidate_firm == firm && *candidate_role == role
                    })
            {
                return Some(*actor);
            }
        }
        self.actors().keys().copied().find(|actor| {
            self.can_perform_corporate_action(*actor, firm, CorporateAction::SetProductionTarget)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Actor, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget, Country,
        CountryId, DemandBasis, EducationLevel, EmploymentStatus, Firm, FirmAppointment,
        FirmPolicy, Good, GoodId, HouseholdCohort, HouseholdType, Money, NeedProfileId, NeedTier,
        OwnershipStake, Population, ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId,
        SimDate, WorldSeed,
    };
    use std::collections::BTreeMap;

    use super::*;

    #[allow(clippy::too_many_lines)]
    fn managed_world() -> World {
        let mut world = World::new(WorldSeed::new(5), SimDate::new(2025, 1).expect("date"));
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
                    Money::from_minor_units(1_000_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Manager", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food",
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
                    5,
                    5,
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
                    Money::default(),
                    Money::default(),
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
            .set_firm_production_target(ActorId::new(1), FirmId::new(1), 5)
            .expect("target");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(10),
            )
            .expect("price");
        world
    }

    #[test]
    fn manager_reduces_target_after_observed_unsold_output() {
        let mut world = managed_world();
        world
            .execute_monthly_commercial_cycle()
            .expect("commercial cycle");
        let mut replayed = world.clone();
        let decisions = world
            .execute_observed_firm_management()
            .expect("management");
        WorldCommand::ExecuteObservedFirmManagement
            .apply(&mut replayed)
            .expect("replay");

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].previous_batches, 5);
        assert_eq!(decisions[0].target_batches, 0);
        assert_eq!(world.firm_production_targets()[&FirmId::new(1)], 0);
        assert_eq!(world, replayed);
    }
}
