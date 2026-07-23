use std::collections::BTreeMap;

use crate::{
    ActorId, CohortId, CountryId, DemandIntent, DomainEvent, MarketClearing, Money, NeedTier,
    PowerNodeKind, QuantityMilli, RegionId, World, WorldCommand, WorldError,
};

/// One treasury-funded transfer responding to survival goods that were available but unaffordable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmergencyReliefPayment {
    pub actor: ActorId,
    pub country: CountryId,
    pub cohort: CohortId,
    pub amount: Money,
    pub public_borrowing: Money,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EmergencyReliefStrategy {
    TreasuryOnly,
    BorrowWithinDebtLimit,
    Inaction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GovernmentEmergencyPolicy {
    strategy: EmergencyReliefStrategy,
}

impl Default for GovernmentEmergencyPolicy {
    fn default() -> Self {
        Self {
            strategy: EmergencyReliefStrategy::TreasuryOnly,
        }
    }
}

impl GovernmentEmergencyPolicy {
    #[must_use]
    pub const fn new(strategy: EmergencyReliefStrategy) -> Self {
        Self { strategy }
    }

    #[must_use]
    pub const fn strategy(self) -> EmergencyReliefStrategy {
        self.strategy
    }
}

impl World {
    pub(crate) fn capture_monthly_affordability_gaps(
        &mut self,
        intents: &[DemandIntent],
        clearing: &MarketClearing,
    ) -> Result<(), WorldError> {
        let mut unsold: BTreeMap<(RegionId, crate::GoodId), Vec<UnsoldLot>> = BTreeMap::new();
        for outcome in &clearing.offer_outcomes {
            if outcome.unsold.get() > 0 {
                unsold
                    .entry((outcome.region, outcome.good))
                    .or_default()
                    .push(UnsoldLot {
                        seller: outcome.seller,
                        unit_price: outcome.unit_price,
                        remaining: outcome.unsold,
                    });
            }
        }
        for lots in unsold.values_mut() {
            lots.sort_by_key(|lot| (lot.unit_price.minor_units(), lot.seller));
        }
        let mut gaps = BTreeMap::new();
        for intent in intents
            .iter()
            .copied()
            .filter(|intent| intent.tier() == NeedTier::Survival)
        {
            let missing = intent
                .desired()
                .get()
                .saturating_sub(intent.budgeted().get());
            if missing == 0 {
                continue;
            }
            let region = self
                .cohorts
                .get(&intent.cohort())
                .ok_or(WorldError::UnknownCohort(intent.cohort()))?
                .region();
            let Some(lots) = unsold.get_mut(&(region, intent.good())) else {
                continue;
            };
            let mut remaining = missing;
            let mut cost = 0_i128;
            for lot in lots {
                let quantity = remaining.min(lot.remaining.get());
                if quantity == 0 {
                    continue;
                }
                cost = cost
                    .checked_add(
                        i128::from(quantity)
                            .checked_mul(i128::from(lot.unit_price.minor_units()))
                            .ok_or(WorldError::ArithmeticOverflow("relief lot cost"))?
                            / i128::from(QuantityMilli::SCALE),
                    )
                    .ok_or(WorldError::ArithmeticOverflow("relief affordability gap"))?;
                lot.remaining = QuantityMilli::new(lot.remaining.get() - quantity);
                remaining -= quantity;
                if remaining == 0 {
                    break;
                }
            }
            if cost > 0 {
                let amount = i64::try_from(cost)
                    .map_err(|_| WorldError::ArithmeticOverflow("relief affordability amount"))?;
                let current = gaps.entry(intent.cohort()).or_insert(0_i64);
                *current = current
                    .checked_add(amount)
                    .ok_or(WorldError::ArithmeticOverflow("cohort relief gap"))?;
            }
        }
        self.monthly_affordability_gaps = gaps
            .into_iter()
            .map(|(cohort, amount)| (cohort, Money::from_minor_units(amount)))
            .collect();
        Ok(())
    }

    /// Transfers treasury cash to one cohort under a current political-office holder's authority.
    ///
    /// # Errors
    /// Returns an error for an unknown cohort, non-positive amount, unauthorized actor, or insufficient
    /// treasury without changing either ledger.
    pub fn fund_emergency_relief(
        &mut self,
        actor: ActorId,
        cohort: CohortId,
        amount: Money,
    ) -> Result<(), WorldError> {
        if amount.minor_units() <= 0 {
            return Err(WorldError::InvalidEmergencyRelief(
                "relief amount must be positive",
            ));
        }
        let region = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?
            .region();
        let country = self
            .regions
            .get(&region)
            .ok_or(WorldError::UnknownRegion(region))?
            .country();
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        let treasury = self.countries[&country].indicators().treasury();
        if treasury.minor_units() < amount.minor_units() {
            return Err(WorldError::InsufficientTreasury(country));
        }
        let mut countries = self.countries.clone();
        let mut cohorts = self.cohorts.clone();
        countries
            .get_mut(&country)
            .ok_or(WorldError::UnknownCountry(country))?
            .indicators_mut()
            .set_treasury(Money::from_minor_units(
                treasury.minor_units() - amount.minor_units(),
            ));
        cohorts
            .get_mut(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?
            .credit_wealth(amount)?;
        self.countries = countries;
        self.cohorts = cohorts;
        self.events.append(
            self.date,
            DomainEvent::EmergencyReliefFunded {
                actor,
                country,
                cohort,
                amount,
            },
        );
        Ok(())
    }

    /// Sets the emergency-response strategy under a current political-office holder's authority.
    /// # Errors
    /// Returns an error for an unknown country or unauthorized actor without changing policy.
    pub fn set_government_emergency_policy(
        &mut self,
        actor: ActorId,
        country: CountryId,
        policy: GovernmentEmergencyPolicy,
    ) -> Result<(), WorldError> {
        if !self.countries.contains_key(&country) {
            return Err(WorldError::UnknownCountry(country));
        }
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        self.government_emergency_policies.insert(country, policy);
        self.events.append(
            self.date,
            DomainEvent::GovernmentEmergencyPolicySet {
                actor,
                country,
                strategy: policy.strategy(),
            },
        );
        Ok(())
    }

    /// Issues bounded public debt and credits the proceeds to treasury for emergency relief.
    /// # Errors
    /// Returns an error for invalid amounts, unauthorized actors, disallowed strategy, or debt-limit excess.
    pub fn issue_emergency_relief_debt(
        &mut self,
        actor: ActorId,
        country: CountryId,
        amount: Money,
    ) -> Result<(), WorldError> {
        if amount.minor_units() <= 0 {
            return Err(WorldError::InvalidEmergencyRelief(
                "public borrowing amount must be positive",
            ));
        }
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        let policy = self
            .government_emergency_policies
            .get(&country)
            .copied()
            .unwrap_or_default();
        if policy.strategy() != EmergencyReliefStrategy::BorrowWithinDebtLimit {
            return Err(WorldError::InvalidEmergencyRelief(
                "current emergency policy does not authorize public borrowing",
            ));
        }
        let headroom = self.emergency_debt_headroom(country)?;
        if amount.minor_units() > headroom {
            return Err(WorldError::InvalidEmergencyRelief(
                "public emergency debt limit exceeded",
            ));
        }
        let indicators = self.countries[&country].indicators();
        let debt = indicators
            .public_debt()
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("emergency public debt"))?;
        let treasury = indicators
            .treasury()
            .minor_units()
            .checked_add(amount.minor_units())
            .ok_or(WorldError::ArithmeticOverflow("borrowed treasury cash"))?;
        let country_state = self
            .countries
            .get_mut(&country)
            .ok_or(WorldError::UnknownCountry(country))?;
        country_state
            .indicators_mut()
            .set_public_debt(Money::from_minor_units(debt));
        country_state
            .indicators_mut()
            .set_treasury(Money::from_minor_units(treasury));
        self.events.append(
            self.date,
            DomainEvent::EmergencyReliefDebtIssued {
                actor,
                country,
                amount,
            },
        );
        Ok(())
    }

    /// Funds next-month survival purchases according to the country's explicit emergency policy.
    /// # Errors
    /// Returns an error on duplicate execution or a failed debt/transfer command without partial response.
    pub fn execute_observed_emergency_relief(
        &mut self,
    ) -> Result<Vec<EmergencyReliefPayment>, WorldError> {
        if self.last_emergency_relief_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "observed emergency relief",
                date: self.date,
            });
        }
        let mut next = self.clone();
        let mut payments = Vec::new();
        for (cohort, gap) in &self.monthly_affordability_gaps {
            let region = self.cohorts[cohort].region();
            let country = self.regions[&region].country();
            let Some(actor) = self.emergency_relief_actor(country) else {
                continue;
            };
            let policy = next
                .government_emergency_policies
                .get(&country)
                .copied()
                .unwrap_or_default();
            if policy.strategy() == EmergencyReliefStrategy::Inaction {
                continue;
            }
            let treasury = next.countries[&country].indicators().treasury();
            let needed = gap
                .minor_units()
                .saturating_sub(treasury.minor_units())
                .max(0);
            let public_borrowing =
                if policy.strategy() == EmergencyReliefStrategy::BorrowWithinDebtLimit {
                    Money::from_minor_units(needed.min(next.emergency_debt_headroom(country)?))
                } else {
                    Money::default()
                };
            if public_borrowing.minor_units() > 0 {
                WorldCommand::IssueEmergencyReliefDebt {
                    actor,
                    country,
                    amount: public_borrowing,
                }
                .apply(&mut next)?;
            }
            let available = next.countries[&country].indicators().treasury();
            let amount = Money::from_minor_units(gap.min(&available).minor_units().max(0));
            if amount.minor_units() == 0 {
                continue;
            }
            WorldCommand::FundEmergencyRelief {
                actor,
                cohort: *cohort,
                amount,
            }
            .apply(&mut next)?;
            payments.push(EmergencyReliefPayment {
                actor,
                country,
                cohort: *cohort,
                amount,
                public_borrowing,
            });
        }
        next.last_emergency_relief_date = Some(next.date);
        next.events.append(
            next.date,
            DomainEvent::ObservedEmergencyReliefCompleted {
                exposed_cohorts: u64::try_from(self.monthly_affordability_gaps.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("relief exposed cohorts"))?,
                funded_cohorts: u64::try_from(payments.len())
                    .map_err(|_| WorldError::ArithmeticOverflow("relief funded cohorts"))?,
            },
        );
        *self = next;
        Ok(payments)
    }

    #[must_use]
    pub fn monthly_affordability_gaps(&self) -> &BTreeMap<CohortId, Money> {
        &self.monthly_affordability_gaps
    }

    #[must_use]
    pub fn government_emergency_policies(&self) -> &BTreeMap<CountryId, GovernmentEmergencyPolicy> {
        &self.government_emergency_policies
    }

    fn emergency_debt_headroom(&self, country: CountryId) -> Result<i64, WorldError> {
        let output = self
            .regions
            .values()
            .filter(|region| region.country() == country)
            .try_fold(0_i128, |sum, region| {
                sum.checked_add(i128::from(region.annual_output().minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "country relief debt capacity",
                    ))
            })?;
        let limit = output
            .checked_mul(2)
            .ok_or(WorldError::ArithmeticOverflow("relief debt limit"))?;
        let debt = i128::from(
            self.countries[&country]
                .indicators()
                .public_debt()
                .minor_units(),
        );
        i64::try_from(limit.saturating_sub(debt).max(0))
            .map_err(|_| WorldError::ArithmeticOverflow("relief debt headroom"))
    }

    fn can_authorize_emergency_relief(&self, actor: ActorId, country: CountryId) -> bool {
        self.power_nodes().values().any(|node| {
            node.country() == country
                && node.kind() == PowerNodeKind::PoliticalOffice
                && node.holder() == Some(actor)
        })
    }

    fn emergency_relief_actor(&self, country: CountryId) -> Option<ActorId> {
        self.power_nodes()
            .values()
            .filter(|node| {
                node.country() == country && node.kind() == PowerNodeKind::PoliticalOffice
            })
            .filter_map(crate::PowerNode::holder)
            .min()
    }
}

#[derive(Clone, Copy, Debug)]
struct UnsoldLot {
    seller: crate::FirmId,
    unit_price: Money,
    remaining: QuantityMilli,
}
