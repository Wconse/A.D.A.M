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
        let matured_default = self.firm_creditor_claims.values().any(|claim| {
            claim.firm() == firm
                && claim.schedule().is_some_and(|schedule| {
                    schedule.installments_remaining() == 0
                        && (claim.principal().minor_units() > 0
                            || claim.accrued_interest().minor_units() > 0)
                })
        });
        if arrears.minor_units() <= 0 && !matured_default {
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
        let observations = self.firm_operating_history.get(&firm).map_or(0, Vec::len);
        let awaiting_credit_evidence = (2..4).contains(&observations);
        if months < RECAPITALIZATION_TRIGGER_MONTHS
            || (months <= RECAPITALIZATION_TRIGGER_MONTHS.saturating_add(1)
                && awaiting_credit_evidence)
        {
            return Ok(None);
        }
        if arrears.minor_units() > 0 {
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
        FirmCreditorPriority, FirmPolicy, FirmReorganizationPlan, Good, GoodId, HouseholdCohort,
        HouseholdType, NeedProfileId, NeedTier, OwnershipStake, Population, ProductionInput,
        ProductionRecipe, QuantityMilli, RecipeId, Region, RegionId, SimDate, WorldCommand,
        WorldSeed,
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

    #[test]
    fn bounded_administration_liquidates_with_worker_priority_and_replays() {
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
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("firm")
            .set_cash(Money::from_minor_units(200));
        for _ in 0..11 {
            world.advance_month().expect("administration month");
            assert!(
                world
                    .execute_observed_firm_liquidations()
                    .expect("not due")
                    .is_empty()
            );
        }
        world.advance_month().expect("twelfth administration month");
        let mut replayed = world.clone();
        let liquidations = world
            .execute_observed_firm_liquidations()
            .expect("liquidation");
        WorldCommand::ExecuteObservedFirmLiquidations
            .apply(&mut replayed)
            .expect("replayed liquidation");

        assert_eq!(liquidations.len(), 1);
        assert_eq!(liquidations[0].claims_paid, Money::from_minor_units(200));
        assert_eq!(
            liquidations[0].claims_written_off,
            Money::from_minor_units(100)
        );
        assert_eq!(
            liquidations[0].inventory_written_off[&GoodId::new(1)],
            QuantityMilli::new(2_500)
        );
        assert_eq!(liquidations[0].owner_distribution, Money::default());
        assert!(world.is_firm_liquidated(FirmId::new(1)));
        assert!(world.firms()[&FirmId::new(1)].inventories().is_empty());
        assert_eq!(world.firms()[&FirmId::new(1)].cash(), Money::default());
        assert_eq!(
            world.household_cohorts()[&CohortId::new(1)].liquid_wealth(),
            Money::from_minor_units(200)
        );
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].arrears(),
            Money::default()
        );
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(
            world
                .plan_observed_firm_reorganization(FirmId::new(1))
                .expect("terminal plan")
                .is_none()
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn liquidation_pays_workers_then_ranked_creditors_before_owners() {
        let mut world = distressed_world();
        for (id, name, cash) in [(2, "Secured lender", 100), (3, "Trade creditor", 200)] {
            world
                .register_actor(
                    Actor::new(ActorId::new(id), name, RegionId::new(1), 1975).expect("creditor"),
                )
                .expect("register creditor");
            world
                .actor_cash
                .insert(ActorId::new(id), Money::from_minor_units(cash));
        }
        let commands = [
            WorldCommand::IssueFirmCredit {
                creditor: ActorId::new(2),
                firm: FirmId::new(1),
                priority: FirmCreditorPriority::Secured,
                principal: Money::from_minor_units(100),
            },
            WorldCommand::IssueFirmCredit {
                creditor: ActorId::new(3),
                firm: FirmId::new(1),
                priority: FirmCreditorPriority::Unsecured,
                principal: Money::from_minor_units(200),
            },
        ];
        let mut credit_replay = world.clone();
        world
            .issue_firm_credit(
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(100),
            )
            .expect("secured credit");
        world
            .issue_firm_credit(
                ActorId::new(3),
                FirmId::new(1),
                FirmCreditorPriority::Unsecured,
                Money::from_minor_units(200),
            )
            .expect("unsecured credit");
        for command in &commands {
            command.apply(&mut credit_replay).expect("credit replay");
        }
        assert_eq!(world, credit_replay);
        assert_eq!(
            world.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(300)
        );

        // The borrowed working capital is exhausted before payroll distress begins.
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("firm")
            .set_cash(Money::default());
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
            .firms
            .get_mut(&FirmId::new(1))
            .expect("estate")
            .set_cash(Money::from_minor_units(350));
        for _ in 0..12 {
            world.advance_month().expect("administration month");
        }
        let mut replayed = world.clone();
        let liquidation = world
            .execute_observed_firm_liquidations()
            .expect("liquidation")
            .pop()
            .expect("liquidation result");
        WorldCommand::ExecuteObservedFirmLiquidations
            .apply(&mut replayed)
            .expect("liquidation replay");

        assert_eq!(liquidation.claims_paid, Money::from_minor_units(300));
        assert_eq!(liquidation.claims_written_off, Money::default());
        assert_eq!(
            liquidation.creditor_claims_paid,
            Money::from_minor_units(50)
        );
        assert_eq!(
            liquidation.creditor_claims_written_off,
            Money::from_minor_units(250)
        );
        assert_eq!(liquidation.owner_distribution, Money::default());
        assert_eq!(
            world.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(50)
        );
        assert_eq!(world.actor_cash()[&ActorId::new(3)], Money::default());
        let secured_history = world.lender_credit_history()[&ActorId::new(2)];
        assert_eq!(
            secured_history.principal_repaid(),
            Money::from_minor_units(50)
        );
        assert_eq!(
            secured_history.realized_losses(),
            Money::from_minor_units(50)
        );
        assert_eq!(secured_history.defaulted_loans(), 1);
        let unsecured_history = world.lender_credit_history()[&ActorId::new(3)];
        assert_eq!(unsecured_history.principal_repaid(), Money::default());
        assert_eq!(
            unsecured_history.realized_losses(),
            Money::from_minor_units(200)
        );
        assert_eq!(unsecured_history.defaulted_loans(), 1);
        assert!(world.firm_creditor_claims().is_empty());
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(world.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::FirmCreditorClaimSettled {
                creditor,
                priority: FirmCreditorPriority::Secured,
                paid,
                written_off,
                ..
            } if *creditor == ActorId::new(2)
                && *paid == Money::from_minor_units(50)
                && *written_off == Money::from_minor_units(50)
        )));
    }

    #[test]
    fn matured_debt_default_without_wage_arrears_enters_insolvency() {
        let mut world = distressed_world();
        world
            .issue_scheduled_firm_credit(
                ActorId::new(1),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(12),
                1,
            )
            .expect("one-month credit");
        world
            .change_employment_workers(FirmId::new(1), CohortId::new(1), 0)
            .expect("close workforce");
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("firm")
            .debit_cash(Money::from_minor_units(12))
            .expect("consume loan cash");
        world.advance_month().expect("loan maturity");
        let service = world
            .execute_monthly_firm_debt_service()
            .expect("missed maturity");
        assert_eq!(service.len(), 1);
        assert!(service[0].overdue);
        assert_eq!(service[0].paid, Money::default());
        assert_eq!(
            world.employment_agreements()[&(FirmId::new(1), CohortId::new(1))].arrears(),
            Money::default()
        );

        let mut final_actions = Vec::new();
        for month in 1..=6 {
            final_actions = world
                .execute_observed_firm_distress_response()
                .expect("debt-default distress");
            if month < 6 {
                world.advance_month().expect("default month");
            }
        }
        assert!(matches!(
            final_actions.as_slice(),
            [FirmDistressAction::DeclaredInsolvent { firm, .. }]
                if *firm == FirmId::new(1)
        ));
        assert!(world.is_firm_insolvent(FirmId::new(1)));
        assert_eq!(
            world.firm_insolvencies()[&FirmId::new(1)].wage_arrears(),
            Money::default()
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn autonomous_credit_market_funds_viable_gap_from_competing_domestic_lenders() {
        let mut direct = distressed_world();
        direct
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(100),
            )
            .expect("collateral price");
        for (id, name) in [(2, "First lender"), (3, "Second lender")] {
            direct
                .register_actor(
                    Actor::new(ActorId::new(id), name, RegionId::new(1), 1975).expect("lender"),
                )
                .expect("register lender");
            direct
                .actor_cash
                .insert(ActorId::new(id), Money::from_minor_units(1_000));
        }
        for month in 0..3 {
            direct
                .record_firm_sale(FirmId::new(1), Money::from_minor_units(500))
                .expect("observed sales");
            direct
                .record_firm_production(FirmId::new(1), 1)
                .expect("observed production");
            direct
                .capture_monthly_firm_observation(FirmId::new(1))
                .expect("operating observation");
            direct.reset_monthly_firm_accounts();
            if month < 2 {
                direct.advance_month().expect("observation month");
            }
        }
        let mut replayed = direct.clone();
        let decisions = direct
            .execute_observed_firm_credit_market()
            .expect("autonomous credit market");
        WorldCommand::ExecuteObservedFirmCreditMarket
            .apply(&mut replayed)
            .expect("replayed credit market");

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].actor, ActorId::new(1));
        assert_eq!(decisions[0].creditor, ActorId::new(2));
        assert_eq!(decisions[0].funding_gap, Money::from_minor_units(100));
        assert_eq!(decisions[0].principal, Money::from_minor_units(110));
        assert_eq!(decisions[0].annual_interest.get(), 600);
        assert_eq!(
            direct.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(110)
        );
        assert_eq!(
            direct.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(890)
        );
        assert_eq!(
            direct.actor_cash()[&ActorId::new(3)],
            Money::from_minor_units(1_000)
        );
        assert!(direct.firm_credit_offers().is_empty());
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        direct.events.append(
            direct.date(),
            DomainEvent::EconomicYearCompleted {
                closed_year: direct.date().year(),
                monthly_cycles: 12,
            },
        );
        assert!(direct.chronicle().iter().any(|entry| {
            entry.text.contains(
                "accepted 1 observed credit offers providing 110 minor currency units of working capital",
            )
        }));

        let before = direct.clone();
        assert!(matches!(
            direct.execute_observed_firm_credit_market(),
            Err(WorldError::MonthlyStageAlreadyExecuted { .. })
        ));
        assert_eq!(direct, before);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn scarce_lender_headroom_is_ranked_across_contemporaneous_applications() {
        let mut direct = distressed_world();
        direct
            .register_good(Good::new(GoodId::new(2), "Prepared food").expect("good"))
            .expect("prepared good");
        direct
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(2),
                Money::from_minor_units(100),
            )
            .expect("prepared price");
        direct
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(2),
                    "Prepared food",
                    GoodId::new(2),
                    QuantityMilli::new(1_000),
                    1,
                    vec![ProductionInput::new(
                        GoodId::new(1),
                        QuantityMilli::new(10_000),
                    )],
                )
                .expect("recipe"),
            )
            .expect("prepared recipe");
        direct
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Kitchen",
                    RegionId::new(1),
                    RecipeId::new(2),
                    1,
                    1,
                    Money::default(),
                    BTreeMap::from([(GoodId::new(2), QuantityMilli::new(2_000))]),
                )
                .expect("firm"),
            )
            .expect("kitchen");
        direct
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(2),
                ActorId::new(1),
                BasisPoints::new(10_000).expect("rights"),
                BasisPoints::new(10_000).expect("rights"),
            ))
            .expect("kitchen ownership");
        direct
            .set_firm_policy(
                ActorId::new(1),
                FirmId::new(2),
                FirmPolicy::new(
                    0,
                    BasisPoints::ZERO,
                    BasisPoints::ZERO,
                    BasisPoints::ZERO,
                    BasisPoints::ZERO,
                )
                .expect("policy"),
            )
            .expect("kitchen policy");
        direct
            .register_actor(
                Actor::new(ActorId::new(2), "Finite lender", RegionId::new(1), 1975)
                    .expect("lender"),
            )
            .expect("register lender");
        // Forty-percent portfolio headroom is 120: enough for one 110-unit request, not both.
        direct
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(300));
        direct.firm_distress_months.insert(FirmId::new(1), 2);
        for month in 0..3 {
            for firm in [FirmId::new(1), FirmId::new(2)] {
                direct
                    .record_firm_sale(firm, Money::from_minor_units(500))
                    .expect("observed sales");
                direct
                    .record_firm_production(firm, 1)
                    .expect("observed production");
                direct
                    .capture_monthly_firm_observation(firm)
                    .expect("operating observation");
            }
            direct.reset_monthly_firm_accounts();
            if month < 2 {
                direct.advance_month().expect("observation month");
            }
        }

        let mut replayed = direct.clone();
        let decisions = direct
            .execute_observed_firm_credit_market()
            .expect("batched credit market");
        WorldCommand::ExecuteObservedFirmCreditMarket
            .apply(&mut replayed)
            .expect("replayed batch");

        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].firm, FirmId::new(2));
        assert_eq!(decisions[0].creditor, ActorId::new(2));
        assert_eq!(decisions[0].funding_gap, Money::from_minor_units(100));
        assert_eq!(decisions[0].principal, Money::from_minor_units(110));
        assert_eq!(direct.firms()[&FirmId::new(1)].cash(), Money::default());
        assert_eq!(
            direct.firms()[&FirmId::new(2)].cash(),
            Money::from_minor_units(110)
        );
        assert_eq!(
            direct.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(190)
        );
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn lender_track_record_changes_rate_and_portfolio_capacity() {
        let mut world = distressed_world();
        world
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        world
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(1_000));
        assert_eq!(
            world
                .autonomous_lender_capacity(ActorId::new(2))
                .expect("baseline capacity"),
            Money::from_minor_units(400)
        );
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("baseline rate"),
            BasisPoints::new(600).expect("base rate")
        );
        let baseline_fingerprint = world.stable_fingerprint();
        world
            .record_lender_credit_outcome(
                ActorId::new(2),
                Money::from_minor_units(100),
                Money::from_minor_units(10),
                Money::default(),
                true,
                false,
            )
            .expect("successful history");
        assert_ne!(world.stable_fingerprint(), baseline_fingerprint);
        assert_eq!(
            world
                .autonomous_lender_capacity(ActorId::new(2))
                .expect("earned capacity"),
            Money::from_minor_units(410)
        );
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("earned discount"),
            BasisPoints::new(550).expect("discounted rate")
        );
        world
            .record_lender_credit_outcome(
                ActorId::new(2),
                Money::default(),
                Money::default(),
                Money::from_minor_units(100),
                true,
                true,
            )
            .expect("default history");
        assert_eq!(
            world
                .autonomous_lender_capacity(ActorId::new(2))
                .expect("loss capacity"),
            Money::from_minor_units(160)
        );
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("loss premium"),
            BasisPoints::new(2_050).expect("loss adjusted rate")
        );
        let history = world.lender_credit_history()[&ActorId::new(2)];
        assert_eq!(history.interest_income(), Money::from_minor_units(10));
        assert_eq!(history.successful_loans(), 1);
        assert_eq!(history.defaulted_loans(), 1);
    }

    #[test]
    fn borrower_payment_history_changes_future_credit_price() {
        let mut world = distressed_world();
        world
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        world
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(1_000));
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("clean borrower rate"),
            BasisPoints::new(600).expect("base rate")
        );
        let baseline = world.stable_fingerprint();
        world
            .record_borrower_credit_outcome(
                FirmId::new(1),
                Money::from_minor_units(100),
                Money::from_minor_units(50),
                true,
                false,
                false,
            )
            .expect("partial service");
        assert_ne!(world.stable_fingerprint(), baseline);
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("delinquent rate"),
            BasisPoints::new(1_700).expect("delinquent premium")
        );
        world
            .record_borrower_credit_outcome(
                FirmId::new(1),
                Money::from_minor_units(50),
                Money::from_minor_units(50),
                true,
                true,
                false,
            )
            .expect("successful resolution");
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("partly rehabilitated rate"),
            BasisPoints::new(1_316).expect("history adjusted rate")
        );
        world
            .record_borrower_credit_outcome(
                FirmId::new(1),
                Money::default(),
                Money::default(),
                false,
                true,
                true,
            )
            .expect("default resolution");
        assert_eq!(
            world
                .autonomous_credit_rate(ActorId::new(2), FirmId::new(1))
                .expect("post-default rate"),
            BasisPoints::new(2_316).expect("default premium")
        );
        let history = world.borrower_credit_history()[&FirmId::new(1)];
        assert_eq!(history.scheduled_due(), Money::from_minor_units(150));
        assert_eq!(history.scheduled_paid(), Money::from_minor_units(100));
        assert_eq!(history.on_time_payments(), 1);
        assert_eq!(history.delinquent_payments(), 1);
        assert_eq!(history.successful_loans(), 1);
        assert_eq!(history.defaulted_loans(), 1);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn observed_underwriting_prices_credit_and_interest_moves_real_cash() {
        let mut direct = distressed_world();
        direct
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        direct
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(1_000));
        for month in 0..3 {
            direct
                .record_firm_sale(FirmId::new(1), Money::from_minor_units(500))
                .expect("observed sales");
            direct
                .record_firm_production(FirmId::new(1), 1)
                .expect("observed production");
            direct
                .capture_monthly_firm_observation(FirmId::new(1))
                .expect("operating observation");
            direct.reset_monthly_firm_accounts();
            if month < 2 {
                direct.advance_month().expect("observation month");
            }
        }
        let mut replayed = direct.clone();
        let rate = BasisPoints::new(1_200).expect("interest rate");
        let offer = direct
            .underwrite_firm_credit_offer(
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(20),
                rate,
                12,
            )
            .expect("underwritten offer");
        WorldCommand::UnderwriteFirmCreditOffer {
            creditor: ActorId::new(2),
            firm: FirmId::new(1),
            priority: FirmCreditorPriority::Secured,
            requested_principal: Money::from_minor_units(20),
            annual_interest: rate,
            term_months: 12,
        }
        .apply(&mut replayed)
        .expect("replayed underwriting");
        assert_eq!(offer.principal(), Money::from_minor_units(20));
        assert_eq!(
            offer.observed_monthly_surplus(),
            Money::from_minor_units(400)
        );
        assert_eq!(offer.collateral_value(), Money::from_minor_units(35));
        assert_eq!(direct, replayed);

        let expectations = direct
            .derive_firm_expectations_from_observations(FirmId::new(1), 3)
            .expect("offer-aware expectations");
        WorldCommand::DeriveFirmExpectationsFromObservations {
            firm: FirmId::new(1),
            horizon_months: 3,
        }
        .apply(&mut replayed)
        .expect("replayed expectations");
        assert_eq!(
            expectations.expected_financing(),
            Money::from_minor_units(20)
        );
        assert_eq!(direct, replayed);

        direct
            .accept_firm_credit_offer(
                ActorId::new(1),
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
            )
            .expect("accepted offer");
        WorldCommand::AcceptFirmCreditOffer {
            actor: ActorId::new(1),
            creditor: ActorId::new(2),
            firm: FirmId::new(1),
            priority: FirmCreditorPriority::Secured,
        }
        .apply(&mut replayed)
        .expect("replayed acceptance");
        assert!(direct.firm_credit_offers().is_empty());
        assert_eq!(
            direct.firms()[&FirmId::new(1)].cash(),
            Money::from_minor_units(20)
        );
        assert_eq!(
            direct.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(980)
        );
        assert_eq!(direct, replayed);

        direct.advance_month().expect("first due month");
        replayed.advance_month().expect("replayed due month");
        let payment = direct
            .execute_monthly_firm_debt_service()
            .expect("interest-bearing service");
        WorldCommand::ExecuteMonthlyFirmDebtService
            .apply(&mut replayed)
            .expect("replayed service");
        assert_eq!(payment[0].interest_charged, Money::from_minor_units(1));
        assert_eq!(payment[0].interest_paid, Money::from_minor_units(1));
        assert_eq!(payment[0].paid, Money::from_minor_units(3));
        assert_eq!(payment[0].remaining_principal, Money::from_minor_units(18));
        assert_eq!(
            direct.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(983)
        );
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }

    #[test]
    fn payroll_is_senior_to_scheduled_principal_and_full_repayment_removes_claim() {
        let mut world = distressed_world();
        world
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        world
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(200));
        world
            .issue_scheduled_firm_credit(
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(200),
                2,
            )
            .expect("scheduled credit");

        world.advance_month().expect("first due month");
        let payroll = world.execute_monthly_payroll().expect("payroll first");
        assert_eq!(payroll[0].paid, Money::from_minor_units(100));
        let first = world
            .execute_monthly_firm_debt_service()
            .expect("first installment");
        assert_eq!(first[0].paid, Money::from_minor_units(100));
        assert_eq!(world.firms()[&FirmId::new(1)].cash(), Money::default());

        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("borrower")
            .set_cash(Money::from_minor_units(200));
        world.advance_month().expect("second due month");
        world.execute_monthly_payroll().expect("payroll second");
        let second = world
            .execute_monthly_firm_debt_service()
            .expect("final installment");
        assert_eq!(second[0].paid, Money::from_minor_units(100));
        assert_eq!(second[0].remaining_principal, Money::default());
        assert!(world.firm_creditor_claims().is_empty());
        assert_eq!(
            world.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(200)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn insolvent_firm_freezes_scheduled_service_until_worker_first_liquidation() {
        let mut world = distressed_world();
        world
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        world
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(120));
        world
            .issue_scheduled_firm_credit(
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(120),
                1,
            )
            .expect("scheduled credit");
        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("borrower")
            .set_cash(Money::default());
        world.actor_cash.insert(ActorId::new(1), Money::default());
        world.advance_month().expect("maturity month");
        let missed = world
            .execute_monthly_firm_debt_service()
            .expect("missed maturity");
        assert!(missed[0].overdue);
        assert_eq!(missed[0].paid, Money::default());

        for month in 1..=6 {
            world.execute_monthly_payroll().expect("payroll");
            world
                .execute_observed_firm_distress_response()
                .expect("distress response");
            if month < 6 {
                world.advance_month().expect("distress month");
            }
        }
        assert!(world.is_firm_insolvent(FirmId::new(1)));
        assert!(
            world
                .execute_monthly_firm_debt_service()
                .expect("frozen service")
                .is_empty()
        );
        assert_eq!(
            world.firm_creditor_claims()[&(
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                ActorId::new(2)
            )]
                .principal(),
            Money::from_minor_units(120)
        );

        world
            .firms
            .get_mut(&FirmId::new(1))
            .expect("estate")
            .set_cash(Money::from_minor_units(420));
        for _ in 0..12 {
            world.advance_month().expect("administration month");
        }
        let liquidation = world
            .execute_observed_firm_liquidations()
            .expect("liquidation")
            .pop()
            .expect("liquidation result");
        assert_eq!(liquidation.claims_paid, Money::from_minor_units(300));
        assert_eq!(
            liquidation.creditor_claims_paid,
            Money::from_minor_units(120)
        );
        assert_eq!(
            world.actor_cash()[&ActorId::new(2)],
            Money::from_minor_units(120)
        );
        assert!(world.firm_creditor_claims().is_empty());
    }

    #[test]
    fn scheduled_credit_amortizes_and_carries_missed_principal_past_maturity() {
        let mut direct = distressed_world();
        direct
            .register_actor(
                Actor::new(ActorId::new(2), "Lender", RegionId::new(1), 1975).expect("lender"),
            )
            .expect("register lender");
        direct
            .actor_cash
            .insert(ActorId::new(2), Money::from_minor_units(120));
        let mut replayed = direct.clone();
        direct
            .issue_scheduled_firm_credit(
                ActorId::new(2),
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                Money::from_minor_units(120),
                3,
            )
            .expect("scheduled credit");
        WorldCommand::IssueScheduledFirmCredit {
            creditor: ActorId::new(2),
            firm: FirmId::new(1),
            priority: FirmCreditorPriority::Secured,
            principal: Money::from_minor_units(120),
            term_months: 3,
        }
        .apply(&mut replayed)
        .expect("replay issuance");
        direct.advance_month().expect("first due month");
        replayed.advance_month().expect("replay due month");
        let first = direct
            .execute_monthly_firm_debt_service()
            .expect("first installment");
        WorldCommand::ExecuteMonthlyFirmDebtService
            .apply(&mut replayed)
            .expect("replay installment");
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].due, Money::from_minor_units(40));
        assert_eq!(first[0].paid, Money::from_minor_units(40));
        assert_eq!(first[0].remaining_principal, Money::from_minor_units(80));
        assert!(!first[0].overdue);
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());

        direct
            .firms
            .get_mut(&FirmId::new(1))
            .expect("borrower")
            .set_cash(Money::from_minor_units(10));
        direct.advance_month().expect("second due month");
        let second = direct
            .execute_monthly_firm_debt_service()
            .expect("partial installment");
        assert_eq!(second[0].due, Money::from_minor_units(40));
        assert_eq!(second[0].paid, Money::from_minor_units(10));
        assert_eq!(second[0].remaining_principal, Money::from_minor_units(70));
        assert!(!second[0].overdue);

        direct.advance_month().expect("maturity month");
        let maturity = direct
            .execute_monthly_firm_debt_service()
            .expect("maturity attempt");
        assert_eq!(maturity[0].due, Money::from_minor_units(70));
        assert_eq!(maturity[0].paid, Money::default());
        assert!(maturity[0].overdue);
        assert_eq!(
            direct.firm_creditor_claims()[&(
                FirmId::new(1),
                FirmCreditorPriority::Secured,
                ActorId::new(2)
            )]
                .principal(),
            Money::from_minor_units(70)
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn liquidation_transfers_capacity_to_a_funded_compatible_successor() {
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
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Successor farm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(150),
                    BTreeMap::new(),
                )
                .expect("successor"),
            )
            .expect("successor");
        world.firm_production_targets.insert(FirmId::new(2), 1);
        for _ in 0..12 {
            world.advance_month().expect("administration month");
        }
        let mut replayed = world.clone();
        let liquidation = world
            .execute_observed_firm_liquidations()
            .expect("liquidation")
            .pop()
            .expect("liquidation result");
        WorldCommand::ExecuteObservedFirmLiquidations
            .apply(&mut replayed)
            .expect("replayed liquidation");

        assert_eq!(liquidation.capacity_sales.len(), 1);
        assert_eq!(liquidation.capacity_sales[0].estate, FirmId::new(1));
        assert_eq!(liquidation.capacity_sales[0].buyer, FirmId::new(2));
        assert_eq!(liquidation.capacity_sales[0].capacity_batches, 1);
        assert_eq!(
            liquidation.capacity_sale_proceeds,
            Money::from_minor_units(10)
        );
        assert_eq!(liquidation.capacity_written_off, 0);
        assert_eq!(liquidation.claims_paid, Money::from_minor_units(10));
        assert_eq!(liquidation.claims_written_off, Money::from_minor_units(290));
        assert_eq!(world.firms()[&FirmId::new(1)].capacity_batches(), 0);
        assert_eq!(world.firms()[&FirmId::new(2)].capacity_batches(), 2);
        assert_eq!(
            world.firms()[&FirmId::new(2)].cash(),
            Money::from_minor_units(140)
        );
        assert_eq!(
            world.household_cohorts()[&CohortId::new(1)].liquid_wealth(),
            Money::from_minor_units(10)
        );
        assert_eq!(world, replayed);
        assert_eq!(world.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(world.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::FirmLiquidationCapacitySold {
                estate,
                buyer,
                capacity_batches: 1,
                proceeds,
            } if *estate == FirmId::new(1)
                && *buyer == FirmId::new(2)
                && *proceeds == Money::from_minor_units(10)
        )));
    }

    #[test]
    fn liquidation_pays_workers_before_residual_owner_value() {
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
            .firms
            .get_mut(&FirmId::new(1))
            .expect("firm")
            .set_cash(Money::from_minor_units(450));
        for _ in 0..12 {
            world.advance_month().expect("administration month");
        }
        let liquidation = world
            .execute_observed_firm_liquidations()
            .expect("liquidation")
            .pop()
            .expect("liquidation result");
        assert_eq!(liquidation.claims_paid, Money::from_minor_units(300));
        assert_eq!(liquidation.claims_written_off, Money::default());
        assert_eq!(liquidation.owner_distribution, Money::from_minor_units(150));
        assert_eq!(
            world.actor_cash()[&ActorId::new(1)],
            Money::from_minor_units(150)
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
                "1 firm was liquidated after a year without a viable plan: solvent producers bought 0 milli-units of estate inventory for 0 minor currency units, workers received 300",
            ) && entry.text.contains("owners received 150 residual cash")
        }));
    }
}
