use crate::{ActorId, DomainEvent, FirmId, Money, World, WorldError};

const DISTRESS_DOWNSIZING_BPS: u64 = 2_500;

const RECAPITALIZATION_TRIGGER_MONTHS: u8 = 3;
const INSOLVENCY_TRIGGER_MONTHS: u8 = 6;

/// Owner-funded liquidity support caused by persistent unpaid wage claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmRecapitalization {
    pub firm: FirmId,
    pub owner: ActorId,
    pub amount: Money,
    pub arrears: Money,
}

/// A bounded employment reduction chosen by authorized firm management when
/// owners cannot finance persistent payroll distress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmDistressDownsizing {
    pub firm: FirmId,
    pub actor: ActorId,
    pub previous_workers: u64,
    pub current_workers: u64,
    pub arrears: Money,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmDistressAction {
    Recapitalized(FirmRecapitalization),
    Downsized(FirmDistressDownsizing),
    DeclaredInsolvent {
        firm: FirmId,
        administrator: ActorId,
    },
}

impl World {
    /// Observes payroll arrears and chooses a resource-conserving response after
    /// three consecutive distressed payrolls: owner recapitalization when liquid,
    /// otherwise bounded authorized downsizing.
    ///
    /// # Errors
    /// Returns an error on inconsistent references or arithmetic overflow without
    /// partially moving money or employment.
    pub(crate) fn execute_observed_firm_distress_response(
        &mut self,
    ) -> Result<Vec<FirmDistressAction>, WorldError> {
        let firm_ids: Vec<_> = self.firms.keys().copied().collect();
        let mut next = self.clone();
        let mut completed = Vec::new();
        for firm in firm_ids {
            if let Some(action) = next.respond_to_firm_distress(firm)? {
                completed.push(action);
            }
        }
        *self = next;
        Ok(completed)
    }

    fn respond_to_firm_distress(
        &mut self,
        firm: FirmId,
    ) -> Result<Option<FirmDistressAction>, WorldError> {
        if self.is_firm_insolvent(firm) {
            return Ok(None);
        }
        let arrears = self.firm_wage_arrears(firm)?;
        if arrears.minor_units() <= 0 {
            self.firm_distress_months.remove(&firm);
            return Ok(None);
        }
        let months = self
            .firm_distress_months
            .get(&firm)
            .copied()
            .unwrap_or(0)
            .saturating_add(1)
            .min(INSOLVENCY_TRIGGER_MONTHS);
        self.firm_distress_months.insert(firm, months);
        if months < RECAPITALIZATION_TRIGGER_MONTHS {
            return Ok(None);
        }
        if let Some(owner) = self.largest_economic_owner(firm) {
            let available = self.actor_cash.get(&owner).copied().unwrap_or_default();
            let amount =
                Money::from_minor_units(available.minor_units().min(arrears.minor_units()));
            if amount.minor_units() > 0 {
                return self
                    .recapitalize_distressed_firm(firm, owner, available, amount, arrears)
                    .map(Some);
            }
        }
        if months >= INSOLVENCY_TRIGGER_MONTHS && self.active_firm_workers(firm) == 0 {
            return self.declare_distressed_firm_insolvent(firm, arrears);
        }
        self.downsize_distressed_firm(firm, arrears)
    }

    fn active_firm_workers(&self, firm: FirmId) -> u64 {
        self.employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm && agreement.active())
            .map(crate::EmploymentAgreement::workers)
            .sum()
    }

    fn declare_distressed_firm_insolvent(
        &mut self,
        firm: FirmId,
        arrears: Money,
    ) -> Result<Option<FirmDistressAction>, WorldError> {
        let Some(administrator) = self.operations_actor(firm) else {
            return Ok(None);
        };
        self.declare_firm_insolvent(firm, administrator, arrears)?;
        Ok(Some(FirmDistressAction::DeclaredInsolvent {
            firm,
            administrator,
        }))
    }

    fn firm_wage_arrears(&self, firm: FirmId) -> Result<Money, WorldError> {
        let value = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm)
            .try_fold(0_i64, |total, agreement| {
                total
                    .checked_add(agreement.arrears().minor_units())
                    .ok_or(WorldError::ArithmeticOverflow("firm wage arrears"))
            })?;
        Ok(Money::from_minor_units(value))
    }

    fn largest_economic_owner(&self, firm: FirmId) -> Option<ActorId> {
        self.ownership_stakes
            .values()
            .filter(|stake| stake.firm() == firm)
            .max_by_key(|stake| {
                (
                    stake.economic_rights().get(),
                    std::cmp::Reverse(stake.owner()),
                )
            })
            .map(|stake| stake.owner())
    }

    fn recapitalize_distressed_firm(
        &mut self,
        firm: FirmId,
        owner: ActorId,
        available: Money,
        amount: Money,
        arrears: Money,
    ) -> Result<FirmDistressAction, WorldError> {
        let firm_cash = self
            .firms
            .get(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?
            .cash()
            .minor_units();
        let new_firm_cash = firm_cash
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("firm recapitalization"))?;
        self.actor_cash.insert(
            owner,
            Money::from_minor_units(available.minor_units() - amount.minor_units()),
        );
        self.firms
            .get_mut(&firm)
            .ok_or(WorldError::UnknownFirm(firm))?
            .set_cash(Money::from_minor_units(new_firm_cash));
        self.firm_distress_months.remove(&firm);
        let result = FirmRecapitalization {
            firm,
            owner,
            amount,
            arrears,
        };
        self.events.append(
            self.date,
            DomainEvent::FirmRecapitalized {
                firm,
                owner,
                amount,
                wage_arrears: arrears,
            },
        );
        Ok(FirmDistressAction::Recapitalized(result))
    }

    fn downsize_distressed_firm(
        &mut self,
        firm: FirmId,
        arrears: Money,
    ) -> Result<Option<FirmDistressAction>, WorldError> {
        let Some(actor) = self.operations_actor(firm) else {
            return Ok(None);
        };
        let agreements: Vec<_> = self
            .employment_agreements
            .values()
            .filter(|agreement| agreement.firm() == firm && agreement.active())
            .map(|agreement| (agreement.cohort(), agreement.workers()))
            .collect();
        let previous_workers = agreements.iter().try_fold(0_u64, |total, (_, workers)| {
            total
                .checked_add(*workers)
                .ok_or(WorldError::ArithmeticOverflow("distress workers"))
        })?;
        if previous_workers == 0 {
            return Ok(None);
        }
        let mut current_workers = 0_u64;
        for (cohort, workers) in agreements {
            let reduction = workers
                .saturating_mul(DISTRESS_DOWNSIZING_BPS)
                .div_ceil(10_000)
                .max(1);
            let retained = workers.saturating_sub(reduction);
            self.change_employment_workers(firm, cohort, retained)?;
            current_workers = current_workers
                .checked_add(retained)
                .ok_or(WorldError::ArithmeticOverflow("distress workers"))?;
        }
        let result = FirmDistressDownsizing {
            firm,
            actor,
            previous_workers,
            current_workers,
            arrears,
        };
        self.events.append(
            self.date,
            DomainEvent::FirmDownsizedForDistress {
                firm,
                actor,
                previous_workers,
                current_workers,
                wage_arrears: arrears,
            },
        );
        Ok(Some(FirmDistressAction::Downsized(result)))
    }

    #[must_use]
    pub fn firm_distress_months(&self) -> &std::collections::BTreeMap<FirmId, u8> {
        &self.firm_distress_months
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, AgeBand, BasisPoints, CohortId, ConsumptionProfile, ConsumptionTarget, Country,
        CountryId, DemandBasis, EducationLevel, EmploymentAgreement, EmploymentStatus, Firm,
        FirmPolicy, FirmReorganizationPlan, Good, GoodId, HouseholdCohort, HouseholdType,
        NeedProfileId, NeedTier, OwnershipStake, Population, ProductionRecipe, QuantityMilli,
        RecipeId, Region, RegionId, SimDate, WorldCommand, WorldSeed,
    };
    use std::collections::BTreeMap;

    #[allow(clippy::too_many_lines)]
    fn distressed_world() -> World {
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
                    "Workers",
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
                    Money::from_minor_units(1_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(10),
            )
            .expect("regional price");
        world
            .register_actor(
                Actor::new(ActorId::new(1), "Owner", RegionId::new(1), 1980).expect("actor"),
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
                    1,
                    1,
                    Money::default(),
                    BTreeMap::from([(GoodId::new(1), QuantityMilli::new(2_500))]),
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
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(1),
                    CohortId::new(1),
                    1,
                    Money::from_minor_units(100),
                )
                .expect("employment"),
            )
            .expect("employment");
        world
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(1),
                ActorId::new(1),
                BasisPoints::new(10_000).expect("rights"),
                BasisPoints::new(10_000).expect("rights"),
            ))
            .expect("ownership");
        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(1),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("marketing"),
                    BasisPoints::new(0).expect("reinvestment"),
                    BasisPoints::new(0).expect("dividend"),
                )
                .expect("policy"),
            )
            .expect("firm policy");
        world
            .actor_cash
            .insert(ActorId::new(1), Money::from_minor_units(500));
        world
    }

    #[test]
    fn persistent_wage_arrears_trigger_bounded_owner_recapitalization() {
        let mut world = distressed_world();
        for expected in [1, 2] {
            world.execute_monthly_payroll().expect("payroll");
            assert!(
                world
                    .execute_observed_firm_distress_response()
                    .expect("distress")
                    .is_empty()
            );
            assert_eq!(world.firm_distress_months()[&FirmId::new(1)], expected);
            world.advance_month().expect("month");
        }
        world.execute_monthly_payroll().expect("third payroll");
        let mut replayed = world.clone();
        let recapitalizations = world
            .execute_observed_firm_distress_response()
            .expect("recapitalization");
        replayed
            .execute_observed_firm_distress_response()
            .expect("replay");

        assert_eq!(recapitalizations.len(), 1);
        assert!(matches!(
            recapitalizations[0],
            FirmDistressAction::Recapitalized(FirmRecapitalization { amount, .. })
                if amount == Money::from_minor_units(300)
        ));
        assert_eq!(
            world.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(300)
        );
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(200)
        );
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].arrears(),
            Money::from_minor_units(300)
        );
        assert!(!world.firm_distress_months().contains_key(&FirmId::new(1)));
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(
            matches!(world.events().events().last().map(crate::EventEnvelope::event),
                Some(DomainEvent::FirmRecapitalized { firm, owner, amount, .. })
                    if *firm == FirmId::new(1) && *owner == ActorId::new(1) && *amount == Money::from_minor_units(300)
            )
        );
    }

    #[test]
    fn cashless_owner_triggers_bounded_downsizing_and_worker_claim_survives_layoff() {
        let mut world = distressed_world();
        world.actor_cash.insert(ActorId::new(1), Money::default());
        for _ in 0..2 {
            world.execute_monthly_payroll().expect("payroll");
            assert!(
                world
                    .execute_observed_firm_distress_response()
                    .expect("distress")
                    .is_empty()
            );
            world.advance_month().expect("month");
        }
        world.execute_monthly_payroll().expect("third payroll");
        let actions = world
            .execute_observed_firm_distress_response()
            .expect("downsizing");
        assert!(matches!(
            actions.as_slice(),
            [FirmDistressAction::Downsized(FirmDistressDownsizing {
                previous_workers: 1,
                current_workers: 0,
                arrears,
                ..
            })] if *arrears == Money::from_minor_units(300)
        ));
        let agreement = &world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))];
        assert!(!agreement.active());
        assert_eq!(agreement.arrears(), Money::from_minor_units(300));

        world
            .actor_cash
            .insert(ActorId::new(1), Money::from_minor_units(300));
        world.advance_month().expect("month");
        world
            .execute_monthly_payroll()
            .expect("inactive claim observed");
        let rescue = world
            .execute_observed_firm_distress_response()
            .expect("late rescue");
        assert!(matches!(
            rescue.as_slice(),
            [FirmDistressAction::Recapitalized(_)]
        ));
        world.advance_month().expect("month");
        let payroll = world.execute_monthly_payroll().expect("claim settlement");
        assert_eq!(payroll.len(), 1);
        assert_eq!(payroll[0].paid, Money::from_minor_units(300));
        assert_eq!(payroll[0].arrears, Money::default());
        assert_eq!(
            world.household_cohorts()[&CohortId::new(1)].liquid_wealth(),
            Money::from_minor_units(300)
        );
        world.events.append(
            world.date(),
            DomainEvent::EconomicYearCompleted {
                closed_year: world.date().year(),
                monthly_cycles: 12,
            },
        );
        let chronicle = world.chronicle();
        assert!(chronicle.iter().any(|entry| {
            entry
                .text
                .contains("Cashless firms released 1 workers across 1 distress downsizings")
        }));
    }

    #[test]
    fn prolonged_zero_workforce_distress_declares_and_freezes_insolvency() {
        let mut world = distressed_world();
        world.actor_cash.insert(ActorId::new(1), Money::default());
        assert_eq!(world.plan_firm_market_offers().expect("offers").len(), 1);
        let mut replay_source = None;
        for month in 1..=6 {
            world.execute_monthly_payroll().expect("payroll");
            if month == 6 {
                replay_source = Some(world.clone());
            }
            let actions = world
                .execute_observed_firm_distress_response()
                .expect("distress response");
            if month == 6 {
                assert!(matches!(
                    actions.as_slice(),
                    [FirmDistressAction::DeclaredInsolvent {
                        firm,
                        administrator,
                    }] if *firm == FirmId::new(1) && *administrator == ActorId::new(1)
                ));
            }
            if month < 6 {
                world.advance_month().expect("month");
            }
        }
        let mut replayed = replay_source.expect("replay source");
        replayed
            .execute_observed_firm_distress_response()
            .expect("replayed insolvency");
        assert_eq!(world, replayed);
        assert!(world.is_firm_insolvent(FirmId::new(1)));
        let insolvency = &world.firm_insolvencies()[&FirmId::new(1)];
        assert_eq!(insolvency.administrator(), ActorId::new(1));
        assert_eq!(insolvency.wage_arrears(), Money::from_minor_units(300));
        assert_eq!(
            insolvency.inventories_at_declaration()[&GoodId::new(1)],
            QuantityMilli::new(2_500)
        );
        assert_eq!(
            world.firms()[&FirmId::new(1)].inventories(),
            insolvency.inventories_at_declaration()
        );
        assert!(
            world
                .plan_firm_market_offers()
                .expect("frozen offers")
                .is_empty()
        );
        let production = world.plan_monthly_production().expect("frozen production");
        assert_eq!(production[0].batches(), 0);
        assert_eq!(production[0].limiting_factor(), "insolvency");
        assert!(matches!(
            world.events().events().last().map(crate::EventEnvelope::event),
            Some(DomainEvent::FirmInsolvencyDeclared { firm, inventory, .. })
                if *firm == FirmId::new(1) && inventory == &vec![(GoodId::new(1), QuantityMilli::new(2_500))]
        ));
        world.events.append(
            world.date(),
            DomainEvent::EconomicYearCompleted {
                closed_year: world.date().year(),
                monthly_cycles: 12,
            },
        );
        assert!(world.chronicle().iter().any(|entry| {
            entry.text.contains(
                "1 firm entered insolvency administration with 300 minor currency units of unpaid wages and 2500 milli-units of inventory preserved",
            )
        }));
    }
    #[test]
    #[allow(clippy::too_many_lines)]
    fn funded_reorganization_pays_claims_and_reopens_through_command_boundary() {
        let mut world = distressed_world();
        world.actor_cash.insert(ActorId::new(1), Money::default());
        for month in 1..=6 {
            world.execute_monthly_payroll().expect("payroll");
            world
                .execute_observed_firm_distress_response()
                .expect("distress response");
            if month < 6 {
                world.advance_month().expect("month");
            }
        }
        assert!(world.is_firm_insolvent(FirmId::new(1)));
        assert!(matches!(
            world.set_firm_production_target(ActorId::new(1), FirmId::new(1), 1),
            Err(WorldError::InvalidFirmReorganization(_))
        ));
        assert!(matches!(
            world.change_employment_workers(FirmId::new(1), CohortId::new(1), 1),
            Err(WorldError::InvalidFirmReorganization(_))
        ));

        world
            .actor_cash
            .insert(ActorId::new(1), Money::from_minor_units(500));
        let plan = FirmReorganizationPlan {
            firm: FirmId::new(1),
            administrator: ActorId::new(1),
            sponsor: ActorId::new(1),
            contribution: Money::from_minor_units(400),
            staffing: vec![(CohortId::new(1), 1)],
            production_target: 1,
        };
        let mut underfunded = world.clone();
        let mut bad_plan = plan.clone();
        bad_plan.contribution = Money::from_minor_units(399);
        let before = underfunded.clone();
        assert!(matches!(
            underfunded.reorganize_firm(&bad_plan),
            Err(WorldError::InvalidFirmReorganization(_))
        ));
        assert_eq!(underfunded, before);

        let mut replayed = world.clone();
        let result = world.reorganize_firm(&plan).expect("reorganization");
        WorldCommand::ReorganizeFirm(plan)
            .apply(&mut replayed)
            .expect("replayed reorganization");

        assert_eq!(result.claims_paid, Money::from_minor_units(300));
        assert_eq!(result.cash_reserve, Money::from_minor_units(100));
        assert_eq!(result.workers, 1);
        assert!(!world.is_firm_insolvent(FirmId::new(1)));
        assert_eq!(
            world.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(100)
        );
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(100)
        );
        assert_eq!(
            world.household_cohorts()[&CohortId::new(1)].liquid_wealth(),
            Money::from_minor_units(300)
        );
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].workers(),
            1
        );
        assert_eq!(world.firm_production_targets()[&FirmId::new(1)], 1);
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(matches!(
            world.events().events().last().map(crate::EventEnvelope::event),
            Some(DomainEvent::FirmReorganized {
                firm,
                claims_paid,
                workers,
                production_target,
                cash_reserve,
                ..
            }) if *firm == FirmId::new(1)
                && *claims_paid == Money::from_minor_units(300)
                && *workers == 1
                && *production_target == 1
                && *cash_reserve == Money::from_minor_units(100)
        ));
        assert_eq!(
            world
                .plan_firm_market_offers()
                .expect("reopened offers")
                .len(),
            1
        );
        assert_eq!(
            world
                .plan_monthly_production()
                .expect("reopened production")[0]
                .batches(),
            1
        );
        world.events.append(
            world.date(),
            DomainEvent::EconomicYearCompleted {
                closed_year: world.date().year(),
                monthly_cycles: 12,
            },
        );
        assert!(world.chronicle().iter().any(|entry| {
            entry.text.contains(
                "1 firm left insolvency after owners contributed 400 minor currency units, paid 300 in worker claims, and funded 1 returning workers",
            )
        }));
    }
    #[test]
    #[allow(clippy::too_many_lines)]
    fn observed_reorganization_respects_owner_reinvestment_policy_and_replays() {
        let mut world = distressed_world();
        world.actor_cash.insert(ActorId::new(1), Money::default());
        for month in 1..=6 {
            world.execute_monthly_payroll().expect("payroll");
            world
                .execute_observed_firm_distress_response()
                .expect("distress response");
            if month < 6 {
                world.advance_month().expect("month");
            }
        }
        world
            .actor_cash
            .insert(ActorId::new(1), Money::from_minor_units(1_000));
        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(1),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("marketing"),
                    BasisPoints::new(3_000).expect("reinvestment"),
                    BasisPoints::new(0).expect("dividend"),
                )
                .expect("policy"),
            )
            .expect("cautious policy");
        assert!(
            world
                .plan_observed_firm_reorganization(FirmId::new(1))
                .expect("proposal")
                .is_none()
        );

        world
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(1),
                FirmPolicy::new(
                    0,
                    BasisPoints::new(0).expect("markup"),
                    BasisPoints::new(0).expect("marketing"),
                    BasisPoints::new(5_000).expect("reinvestment"),
                    BasisPoints::new(0).expect("dividend"),
                )
                .expect("policy"),
            )
            .expect("recovery policy");
        let proposal = world
            .plan_observed_firm_reorganization(FirmId::new(1))
            .expect("proposal")
            .expect("affordable proposal");
        assert_eq!(proposal.contribution, Money::from_minor_units(400));
        assert_eq!(proposal.staffing, vec![(CohortId::new(1), 1)]);
        assert_eq!(proposal.production_target, 1);

        let mut replayed = world.clone();
        let completed = world
            .execute_observed_firm_reorganizations()
            .expect("observed reorganization");
        WorldCommand::ExecuteObservedFirmReorganizations
            .apply(&mut replayed)
            .expect("replayed observation");
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].contribution, Money::from_minor_units(400));
        assert_eq!(completed[0].claims_paid, Money::from_minor_units(300));
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(!world.is_firm_insolvent(FirmId::new(1)));
    }
}
