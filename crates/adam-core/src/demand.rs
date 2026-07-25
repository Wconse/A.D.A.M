use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CohortId, DomainEvent, GoodId, MarketOffer, Money, NeedProfileId, QuantityMilli, RegionId,
    World, WorldError,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum NeedTier {
    Survival,
    Participation,
    Development,
    Discretionary,
}
impl NeedTier {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::Survival => 1,
            Self::Participation => 2,
            Self::Development => 3,
            Self::Discretionary => 4,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum DemandBasis {
    PerPerson,
    PerHousehold,
}
impl DemandBasis {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::PerPerson => 1,
            Self::PerHousehold => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Good {
    id: GoodId,
    name: String,
}

impl Good {
    /// Creates a tradable good identity.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyName`] for an empty name.
    pub fn new(id: GoodId, name: impl Into<String>) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorldError::EmptyName("good"));
        }
        Ok(Self { id, name })
    }
    #[must_use]
    pub const fn id(&self) -> GoodId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConsumptionTarget {
    good: GoodId,
    tier: NeedTier,
    basis: DemandBasis,
    monthly_quantity: QuantityMilli,
}

impl ConsumptionTarget {
    #[must_use]
    pub const fn new(
        good: GoodId,
        tier: NeedTier,
        basis: DemandBasis,
        monthly_quantity: QuantityMilli,
    ) -> Self {
        Self {
            good,
            tier,
            basis,
            monthly_quantity,
        }
    }
    #[must_use]
    pub const fn good(self) -> GoodId {
        self.good
    }
    #[must_use]
    pub const fn tier(self) -> NeedTier {
        self.tier
    }
    #[must_use]
    pub const fn basis(self) -> DemandBasis {
        self.basis
    }
    #[must_use]
    pub const fn monthly_quantity(self) -> QuantityMilli {
        self.monthly_quantity
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConsumptionProfile {
    id: NeedProfileId,
    name: String,
    targets: Vec<ConsumptionTarget>,
}

impl ConsumptionProfile {
    /// Creates a deterministic hierarchy of consumption targets.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::InvalidConsumptionProfile`] for empty or duplicate targets.
    pub fn new(
        id: NeedProfileId,
        name: impl Into<String>,
        mut targets: Vec<ConsumptionTarget>,
    ) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty() || targets.is_empty() {
            return Err(WorldError::InvalidConsumptionProfile(
                "profile name and targets must be non-empty",
            ));
        }
        if targets
            .iter()
            .any(|target| target.monthly_quantity().get() == 0)
        {
            return Err(WorldError::InvalidConsumptionProfile(
                "target quantity must be positive",
            ));
        }
        let unique: BTreeSet<_> = targets.iter().map(|target| target.good()).collect();
        if unique.len() != targets.len() {
            return Err(WorldError::InvalidConsumptionProfile(
                "a good may appear only once in a profile",
            ));
        }
        targets.sort_by_key(|target| (target.tier(), target.good()));
        Ok(Self { id, name, targets })
    }
    #[must_use]
    pub const fn id(&self) -> NeedProfileId {
        self.id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn targets(&self) -> &[ConsumptionTarget] {
        &self.targets
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DemandIntent {
    cohort: CohortId,
    good: GoodId,
    tier: NeedTier,
    desired: QuantityMilli,
    budgeted: QuantityMilli,
    reserved_spend: Money,
}

impl DemandIntent {
    #[must_use]
    pub const fn cohort(self) -> CohortId {
        self.cohort
    }
    #[must_use]
    pub const fn good(self) -> GoodId {
        self.good
    }
    #[must_use]
    pub const fn tier(self) -> NeedTier {
        self.tier
    }
    #[must_use]
    pub const fn desired(self) -> QuantityMilli {
        self.desired
    }
    #[must_use]
    pub const fn budgeted(self) -> QuantityMilli {
        self.budgeted
    }
    #[must_use]
    pub const fn reserved_spend(self) -> Money {
        self.reserved_spend
    }
}

impl World {
    /// Registers a good.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::DuplicateGood`] for a repeated identity.
    pub fn register_good(&mut self, good: Good) -> Result<(), WorldError> {
        if self.goods.contains_key(&good.id()) {
            return Err(WorldError::DuplicateGood(good.id()));
        }
        self.events.append(
            self.date,
            DomainEvent::GoodRegistered {
                good: good.id(),
                name: good.name().to_owned(),
            },
        );
        self.goods.insert(good.id(), good);
        Ok(())
    }

    /// Registers a need profile after validating every good reference.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] for duplicate profiles or unknown goods.
    pub fn register_consumption_profile(
        &mut self,
        profile: ConsumptionProfile,
    ) -> Result<(), WorldError> {
        if self.consumption_profiles.contains_key(&profile.id()) {
            return Err(WorldError::DuplicateNeedProfile(profile.id()));
        }
        for target in profile.targets() {
            if !self.goods.contains_key(&target.good()) {
                return Err(WorldError::UnknownGood(target.good()));
            }
        }
        self.events.append(
            self.date,
            DomainEvent::ConsumptionProfileRegistered {
                profile: profile.id(),
                name: profile.name().to_owned(),
            },
        );
        self.consumption_profiles.insert(profile.id(), profile);
        Ok(())
    }

    /// Sets the observed regional price in minor units per whole quantity unit.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] for unknown references or a non-positive price.
    pub fn set_regional_price(
        &mut self,
        region: RegionId,
        good: GoodId,
        price: Money,
    ) -> Result<(), WorldError> {
        if !self.regions.contains_key(&region) {
            return Err(WorldError::UnknownRegion(region));
        }
        if !self.goods.contains_key(&good) {
            return Err(WorldError::UnknownGood(good));
        }
        if price.minor_units() <= 0 {
            return Err(WorldError::InvalidPrice);
        }
        self.regional_prices.insert((region, good), price);
        self.events.append(
            self.date,
            DomainEvent::RegionalPriceSet {
                region,
                good,
                price,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn goods(&self) -> &BTreeMap<GoodId, Good> {
        &self.goods
    }
    #[must_use]
    pub fn consumption_profiles(&self) -> &BTreeMap<NeedProfileId, ConsumptionProfile> {
        &self.consumption_profiles
    }
    #[must_use]
    pub fn regional_prices(&self) -> &BTreeMap<(RegionId, GoodId), Money> {
        &self.regional_prices
    }

    /// Forms monthly household demand from cohort budgets and need priorities.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] for missing profiles/prices or arithmetic overflow.
    pub fn plan_monthly_household_demand(&self) -> Result<Vec<DemandIntent>, WorldError> {
        self.plan_monthly_household_demand_against_offers(&[])
    }

    /// Forms monthly household demand against the current market offer book.
    ///
    /// Survival targets quote the local offer price when one exists; otherwise
    /// they use the cheapest peaceful direct foreign delivery price.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] for missing profiles/prices or arithmetic overflow.
    pub(crate) fn plan_monthly_household_demand_against_offers(
        &self,
        offers: &[MarketOffer],
    ) -> Result<Vec<DemandIntent>, WorldError> {
        let mut intents = Vec::new();
        let mut cohorts: Vec<_> = self.cohorts.values().collect();
        cohorts.sort_by_key(|cohort| (cohort.region(), cohort.id()));
        for cohort in cohorts {
            let profile = self
                .consumption_profiles
                .get(&cohort.need_profile())
                .ok_or(WorldError::UnknownNeedProfile(cohort.need_profile()))?;
            let mut remaining = i128::from(cohort.liquid_wealth().minor_units()).max(0);
            for tier in [
                NeedTier::Survival,
                NeedTier::Participation,
                NeedTier::Development,
                NeedTier::Discretionary,
            ] {
                let rows = self.price_targets(cohort, profile, tier, offers)?;
                let tier_cost: i128 = rows.iter().map(|row| row.cost).sum();
                let available = remaining.min(tier_cost);
                let spends = proportional_spends(&rows, available)?;
                for (row, spend) in rows.into_iter().zip(spends) {
                    let budgeted = if row.cost == 0 {
                        0
                    } else {
                        spend * row.quantity / row.cost
                    };
                    intents.push(DemandIntent {
                        cohort: cohort.id(),
                        good: row.good,
                        tier,
                        desired: QuantityMilli::new(to_u64(row.quantity, "desired quantity")?),
                        budgeted: QuantityMilli::new(to_u64(budgeted, "budgeted quantity")?),
                        reserved_spend: Money::from_minor_units(to_i64(spend, "reserved spend")?),
                    });
                }
                remaining -= available;
            }
        }
        Ok(intents)
    }

    pub(crate) fn monthly_survival_cost(&self, cohort: CohortId) -> Result<Money, WorldError> {
        let cohort = self
            .cohorts
            .get(&cohort)
            .ok_or(WorldError::UnknownCohort(cohort))?;
        let profile = self
            .consumption_profiles
            .get(&cohort.need_profile())
            .ok_or(WorldError::UnknownNeedProfile(cohort.need_profile()))?;
        let cost = self
            .price_targets(cohort, profile, NeedTier::Survival, &[])?
            .into_iter()
            .try_fold(0_i128, |sum, row| {
                sum.checked_add(row.cost)
                    .ok_or(WorldError::ArithmeticOverflow("monthly survival cost"))
            })?;
        Ok(Money::from_minor_units(i64::try_from(cost).map_err(
            |_| WorldError::ArithmeticOverflow("monthly survival cost"),
        )?))
    }

    fn price_targets(
        &self,
        cohort: &crate::HouseholdCohort,
        profile: &ConsumptionProfile,
        tier: NeedTier,
        offers: &[MarketOffer],
    ) -> Result<Vec<PricedTarget>, WorldError> {
        profile
            .targets()
            .iter()
            .copied()
            .filter(|target| target.tier() == tier)
            .map(|target| {
                let count = match target.basis() {
                    DemandBasis::PerPerson => cohort.people().people(),
                    DemandBasis::PerHousehold => cohort.households(),
                };
                let quantity = i128::from(target.monthly_quantity().get()) * i128::from(count);
                let cost = self.household_target_cost(
                    cohort.region(),
                    target.good(),
                    tier,
                    quantity,
                    offers,
                )?;
                Ok(PricedTarget {
                    good: target.good(),
                    quantity,
                    cost,
                })
            })
            .collect()
    }

    fn household_target_cost(
        &self,
        region: RegionId,
        good: GoodId,
        tier: NeedTier,
        quantity: i128,
        offers: &[MarketOffer],
    ) -> Result<i128, WorldError> {
        let reference = self
            .regional_prices
            .get(&(region, good))
            .copied()
            .ok_or(WorldError::MissingRegionalPrice { region, good })?;
        if tier != NeedTier::Survival {
            return Ok(
                quantity * i128::from(reference.minor_units()) / i128::from(QuantityMilli::SCALE)
            );
        }
        let mut candidates = Vec::new();
        for offer in offers {
            if offer.good != good || offer.quantity.get() == 0 {
                continue;
            }
            if offer.region == region {
                candidates.push((
                    false,
                    offer.unit_price,
                    offer.region,
                    offer.seller,
                    offer.quantity,
                ));
            } else if let Some(tariff) = self.direct_market_route_cost(offer.region, region) {
                let delivered = offer
                    .unit_price
                    .minor_units()
                    .checked_add(tariff.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow("household delivered price"))?;
                candidates.push((
                    true,
                    Money::from_minor_units(delivered),
                    offer.region,
                    offer.seller,
                    offer.quantity,
                ));
            }
        }
        candidates.sort_by_key(|(imported, price, origin, seller, _)| {
            (*imported, price.minor_units(), *origin, *seller)
        });
        let mut remaining = to_u64(quantity, "survival target quantity")?;
        let mut cost = 0_i128;
        for (_, price, _, _, available) in candidates {
            let purchased = remaining.min(available.get());
            cost = cost
                .checked_add(
                    (i128::from(price.minor_units()) * i128::from(purchased)
                        + i128::from(QuantityMilli::SCALE - 1))
                        / i128::from(QuantityMilli::SCALE),
                )
                .ok_or(WorldError::ArithmeticOverflow("household survival cost"))?;
            remaining -= purchased;
            if remaining == 0 {
                return Ok(cost);
            }
        }
        cost.checked_add(
            i128::from(reference.minor_units()) * i128::from(remaining)
                / i128::from(QuantityMilli::SCALE),
        )
        .ok_or(WorldError::ArithmeticOverflow("household survival cost"))
    }
}

#[derive(Clone, Copy, Debug)]
struct PricedTarget {
    good: GoodId,
    quantity: i128,
    cost: i128,
}

fn proportional_spends(rows: &[PricedTarget], available: i128) -> Result<Vec<i128>, WorldError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let total: i128 = rows.iter().map(|row| row.cost).sum();
    if total == 0 || available >= total {
        return Ok(rows.iter().map(|row| row.cost).collect());
    }
    let mut result = Vec::with_capacity(rows.len());
    let mut remainders = Vec::with_capacity(rows.len());
    let mut assigned = 0_i128;
    for row in rows {
        let numerator = available * row.cost;
        let base = numerator / total;
        result.push(base);
        remainders.push(numerator % total);
        assigned += base;
    }
    let leftover = usize::try_from(available - assigned)
        .map_err(|_| WorldError::ArithmeticOverflow("demand budget remainder"))?;
    let mut order: Vec<_> = (0..rows.len()).collect();
    order.sort_by(|&a, &b| {
        remainders[b]
            .cmp(&remainders[a])
            .then_with(|| rows[a].good.cmp(&rows[b].good))
    });
    for index in order.into_iter().take(leftover) {
        result[index] += 1;
    }
    Ok(result)
}

fn to_u64(value: i128, operation: &'static str) -> Result<u64, WorldError> {
    u64::try_from(value).map_err(|_| WorldError::ArithmeticOverflow(operation))
}
fn to_i64(value: i128, operation: &'static str) -> Result<i64, WorldError> {
    i64::try_from(value).map_err(|_| WorldError::ArithmeticOverflow(operation))
}

#[cfg(test)]
mod tests {
    use crate::{
        AgeBand, Country, CountryId, EducationLevel, EmploymentStatus, HouseholdCohort,
        HouseholdType, Population, Region, SimDate, WorldSeed,
    };

    use super::*;

    fn demand_world(annual_income: i64) -> World {
        let mut world = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        world
            .register_good(Good::new(GoodId::new(2), "Leisure").expect("good"))
            .expect("good");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Profile",
                    vec![
                        ConsumptionTarget::new(
                            GoodId::new(1),
                            NeedTier::Survival,
                            DemandBasis::PerPerson,
                            QuantityMilli::new(1_000),
                        ),
                        ConsumptionTarget::new(
                            GoodId::new(2),
                            NeedTier::Discretionary,
                            DemandBasis::PerPerson,
                            QuantityMilli::new(1_000),
                        ),
                    ],
                )
                .expect("profile"),
            )
            .expect("profile");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Capital",
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
                Money::from_minor_units(1_000),
            )
            .expect("price");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(2),
                Money::from_minor_units(1_000),
            )
            .expect("price");
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
                    Money::from_minor_units(annual_income),
                    Money::from_minor_units(0),
                    Money::from_minor_units(0),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
            .execute_monthly_household_cashflows()
            .expect("monthly cashflow");
        world
    }

    #[test]
    fn survival_demand_is_funded_before_discretionary_demand() {
        let world = demand_world(12_000);
        let intents = world.plan_monthly_household_demand().expect("demand");
        assert_eq!(intents[0].tier(), NeedTier::Survival);
        assert_eq!(intents[0].budgeted(), QuantityMilli::new(1_000));
        assert_eq!(intents[1].tier(), NeedTier::Discretionary);
        assert_eq!(intents[1].budgeted(), QuantityMilli::new(0));
    }

    #[test]
    fn demand_never_reserves_more_than_disposable_monthly_resources() {
        let world = demand_world(18_000);
        let intents = world.plan_monthly_household_demand().expect("demand");
        let reserved: i64 = intents
            .iter()
            .map(|intent| intent.reserved_spend().minor_units())
            .sum();
        assert_eq!(reserved, 1_500);
    }

    #[test]
    fn missing_price_is_an_explicit_error() {
        let mut world = demand_world(12_000);
        world
            .regional_prices
            .remove(&(RegionId::new(1), GoodId::new(2)));
        assert!(matches!(
            world.plan_monthly_household_demand(),
            Err(WorldError::MissingRegionalPrice { region, good })
                if region==RegionId::new(1) && good==GoodId::new(2)
        ));
    }
}
