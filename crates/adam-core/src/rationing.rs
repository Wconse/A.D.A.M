use std::collections::BTreeMap;

use crate::{
    CohortId, CountryId, DomainEvent, GoodId, MarketClearing, MarketOffer, MarketOrder, NeedTier,
    PhysicalShortageStrategy, QuantityMilli, RegionId, World, WorldError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurvivalRationingAllocation {
    pub cohort: CohortId,
    pub requested: QuantityMilli,
    pub quota: QuantityMilli,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurvivalRationingOutcome {
    pub country: CountryId,
    pub region: RegionId,
    pub good: GoodId,
    pub requested: QuantityMilli,
    pub available: QuantityMilli,
    pub allocations: Vec<SurvivalRationingAllocation>,
}

impl World {
    /// Applies proportional survival quotas when stored government policy replaces canonical market priority.
    /// # Errors
    /// Returns an error on quantity overflow without partially mutating order quantities or events.
    pub fn apply_survival_rationing(
        &mut self,
        orders: &mut [MarketOrder],
        offers: &[MarketOffer],
    ) -> Result<Vec<SurvivalRationingOutcome>, WorldError> {
        let mut supply = BTreeMap::<(RegionId, GoodId), u64>::new();
        for offer in offers {
            let row = supply.entry((offer.region, offer.good)).or_default();
            *row = row
                .checked_add(offer.quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("rationable survival supply"))?;
        }
        let mut groups = BTreeMap::<(RegionId, GoodId), Vec<usize>>::new();
        for (index, order) in orders.iter().enumerate() {
            if order.tier == NeedTier::Survival {
                groups
                    .entry((order.region, order.good))
                    .or_default()
                    .push(index);
            }
        }
        let mut planned = Vec::new();
        for ((region, good), indexes) in groups {
            let country = self
                .regions
                .get(&region)
                .ok_or(WorldError::UnknownRegion(region))?
                .country();
            let policy = self
                .government_emergency_policies
                .get(&country)
                .copied()
                .unwrap_or_default();
            if policy.physical_shortage_strategy()
                != PhysicalShortageStrategy::ProportionalRationing
            {
                continue;
            }
            let requested = indexes.iter().try_fold(0_u64, |sum, index| {
                sum.checked_add(orders[*index].quantity.get()).ok_or(
                    WorldError::ArithmeticOverflow("rationing requested quantity"),
                )
            })?;
            let available = supply.get(&(region, good)).copied().unwrap_or(0);
            if requested == 0 || available >= requested {
                continue;
            }
            planned.push(plan_group(
                orders, &indexes, country, region, good, available, requested,
            )?);
        }
        for outcome in &planned {
            for allocation in &outcome.allocations {
                let order = orders
                    .iter_mut()
                    .find(|order| {
                        order.buyer == allocation.cohort
                            && order.region == outcome.region
                            && order.good == outcome.good
                            && order.tier == NeedTier::Survival
                    })
                    .ok_or(WorldError::InvalidMarketClearing(
                        "rationed survival order is missing",
                    ))?;
                order.quantity = allocation.quota;
            }
            self.events.append(
                self.date,
                DomainEvent::SurvivalRationingApplied {
                    country: outcome.country,
                    region: outcome.region,
                    good: outcome.good,
                    requested: outcome.requested,
                    available: outcome.available,
                    cohorts: u64::try_from(outcome.allocations.len())
                        .map_err(|_| WorldError::ArithmeticOverflow("rationed cohorts"))?,
                },
            );
        }
        Ok(planned)
    }

    /// Restores rationed-away demand to market unmet ledgers after quota-constrained clearing.
    /// # Errors
    /// Returns an error on quantity overflow.
    pub fn restore_rationed_unmet_demand(
        &self,
        clearing: &mut MarketClearing,
        outcomes: &[SurvivalRationingOutcome],
    ) -> Result<(), WorldError> {
        for outcome in outcomes {
            let mut withheld_total = 0_u64;
            for allocation in &outcome.allocations {
                let withheld = allocation.requested.get() - allocation.quota.get();
                withheld_total = withheld_total
                    .checked_add(withheld)
                    .ok_or(WorldError::ArithmeticOverflow("withheld ration demand"))?;
                let row = clearing
                    .unmet
                    .entry((allocation.cohort, outcome.good, NeedTier::Survival))
                    .or_default();
                *row = QuantityMilli::new(
                    row.get()
                        .checked_add(withheld)
                        .ok_or(WorldError::ArithmeticOverflow("rationed unmet demand"))?,
                );
            }
            for offer in clearing
                .offer_outcomes
                .iter_mut()
                .filter(|offer| offer.region == outcome.region && offer.good == outcome.good)
            {
                offer.unmet_market_demand = QuantityMilli::new(
                    offer
                        .unmet_market_demand
                        .get()
                        .checked_add(withheld_total)
                        .ok_or(WorldError::ArithmeticOverflow("rationed market demand"))?,
                );
            }
        }
        Ok(())
    }
}

fn plan_group(
    orders: &[MarketOrder],
    indexes: &[usize],
    country: CountryId,
    region: RegionId,
    good: GoodId,
    available: u64,
    requested: u64,
) -> Result<SurvivalRationingOutcome, WorldError> {
    let mut rows = Vec::with_capacity(indexes.len());
    let mut assigned = 0_u64;
    for index in indexes {
        let order = orders[*index];
        let numerator = u128::from(order.quantity.get()) * u128::from(available);
        let base = u64::try_from(numerator / u128::from(requested))
            .map_err(|_| WorldError::ArithmeticOverflow("ration quota"))?;
        assigned = assigned
            .checked_add(base)
            .ok_or(WorldError::ArithmeticOverflow("ration assignment"))?;
        rows.push((
            order.buyer,
            order.quantity,
            base,
            numerator % u128::from(requested),
        ));
    }
    let mut order: Vec<_> = (0..rows.len()).collect();
    order.sort_by(|left, right| {
        rows[*right]
            .3
            .cmp(&rows[*left].3)
            .then_with(|| rows[*left].0.cmp(&rows[*right].0))
    });
    let leftover = usize::try_from(available - assigned)
        .map_err(|_| WorldError::ArithmeticOverflow("ration remainder"))?;
    for index in order.into_iter().take(leftover) {
        rows[index].2 += 1;
    }
    Ok(SurvivalRationingOutcome {
        country,
        region,
        good,
        requested: QuantityMilli::new(requested),
        available: QuantityMilli::new(available),
        allocations: rows
            .into_iter()
            .map(
                |(cohort, requested, quota, _)| SurvivalRationingAllocation {
                    cohort,
                    requested,
                    quota: QuantityMilli::new(quota),
                },
            )
            .collect(),
    })
}
