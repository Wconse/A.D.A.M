use crate::{CohortId, FirmId, GoodId, Money, NeedTier, QuantityMilli, RegionId, WorldError};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarketOffer {
    pub seller: FirmId,
    pub region: RegionId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub unit_price: Money,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarketOrder {
    pub buyer: CohortId,
    pub tier: NeedTier,
    pub region: RegionId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub max_spend: Money,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarketFill {
    pub buyer: CohortId,
    pub tier: NeedTier,
    pub seller: FirmId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub spend: Money,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarketClearing {
    pub fills: Vec<MarketFill>,
    pub unmet: BTreeUnmet,
}
pub type BTreeUnmet = std::collections::BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli>;
/// Clears local goods markets deterministically by region/good, then order ID, price, and seller ID.
/// # Errors
/// Returns an error for non-positive prices or arithmetic overflow.
pub fn clear_local_market(
    orders: &[MarketOrder],
    offers: &[MarketOffer],
) -> Result<MarketClearing, WorldError> {
    let mut orders = orders.to_vec();
    orders.sort_by_key(|o| (o.region, o.good, o.tier, o.buyer));
    let mut supply = offers.to_vec();
    supply.sort_by_key(|o| (o.region, o.good, o.unit_price.minor_units(), o.seller));
    let mut remaining: Vec<u64> = supply.iter().map(|o| o.quantity.get()).collect();
    let mut fills = Vec::new();
    let mut unmet = BTreeUnmet::new();
    for order in orders {
        let mut need = order.quantity.get();
        let mut budget = order.max_spend.minor_units();
        for (index, offer) in supply.iter().enumerate() {
            if need == 0 {
                break;
            }
            if offer.region != order.region || offer.good != order.good || remaining[index] == 0 {
                continue;
            }
            let price = offer.unit_price.minor_units();
            if price <= 0 {
                return Err(WorldError::InvalidPrice);
            }
            let affordable = u64::try_from(
                (i128::from(budget) * i128::from(QuantityMilli::SCALE) / i128::from(price)).max(0),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("market affordability"))?;
            let quantity = need.min(remaining[index]).min(affordable);
            if quantity == 0 {
                continue;
            }
            let spend = i64::try_from(
                i128::from(price) * i128::from(quantity) / i128::from(QuantityMilli::SCALE),
            )
            .map_err(|_| WorldError::ArithmeticOverflow("market spend"))?;
            need -= quantity;
            remaining[index] -= quantity;
            budget -= spend;
            fills.push(MarketFill {
                buyer: order.buyer,
                tier: order.tier,
                seller: offer.seller,
                good: order.good,
                quantity: QuantityMilli::new(quantity),
                spend: Money::from_minor_units(spend),
            });
        }
        if need > 0 {
            unmet.insert(
                (order.buyer, order.good, order.tier),
                QuantityMilli::new(need),
            );
        }
    }
    Ok(MarketClearing { fills, unmet })
}
impl crate::World {
    /// Atomically settles pre-cleared market fills against household wealth and firm inventories.
    /// # Errors
    /// Returns an error without mutation for insufficient cash, stock, or arithmetic overflow.
    pub fn settle_local_market(&mut self, clearing: &MarketClearing) -> Result<(), WorldError> {
        let mut cohorts = self.cohorts.clone();
        let mut firms = self.firms.clone();
        for fill in &clearing.fills {
            cohorts
                .get_mut(&fill.buyer)
                .ok_or(WorldError::UnknownCohort(fill.buyer))?
                .debit_wealth(fill.spend)?;
            let firm = firms
                .get_mut(&fill.seller)
                .ok_or(WorldError::UnknownFirm(fill.seller))?;
            firm.debit_inventory(fill.good, fill.quantity)?;
            firm.apply_cash_delta(fill.spend)?;
        }
        let mut consumption: std::collections::BTreeMap<
            (CohortId, GoodId, NeedTier),
            QuantityMilli,
        > = std::collections::BTreeMap::new();
        for fill in &clearing.fills {
            let key = (fill.buyer, fill.good, fill.tier);
            let current = consumption.get(&key).copied().unwrap_or_default();
            let next = QuantityMilli::new(
                current
                    .get()
                    .checked_add(fill.quantity.get())
                    .ok_or(WorldError::ArithmeticOverflow("monthly consumption"))?,
            );
            consumption.insert(key, next);
        }
        self.cohorts = cohorts;
        self.firms = firms;
        let mut weighted_desired: std::collections::BTreeMap<CohortId, u128> =
            std::collections::BTreeMap::new();
        let mut weighted_unmet: std::collections::BTreeMap<CohortId, u128> =
            std::collections::BTreeMap::new();
        let weight = |tier: NeedTier| -> u128 {
            match tier {
                NeedTier::Survival => 4,
                NeedTier::Participation => 3,
                NeedTier::Development => 2,
                NeedTier::Discretionary => 1,
            }
        };
        for ((cohort, _good, tier), quantity) in &consumption {
            *weighted_desired.entry(*cohort).or_default() +=
                u128::from(quantity.get()) * weight(*tier);
        }
        for ((cohort, _good, tier), quantity) in &clearing.unmet {
            let value = u128::from(quantity.get()) * weight(*tier);
            *weighted_desired.entry(*cohort).or_default() += value;
            *weighted_unmet.entry(*cohort).or_default() += value;
        }
        let mut pressure = std::collections::BTreeMap::new();
        for cohort in weighted_desired.keys() {
            let desired = weighted_desired[cohort];
            let unmet = weighted_unmet.get(cohort).copied().unwrap_or_default();
            let bps = if desired == 0 {
                0
            } else {
                u16::try_from(unmet * 10_000 / desired)
                    .map_err(|_| WorldError::ArithmeticOverflow("deprivation pressure"))?
            };
            pressure.insert(
                *cohort,
                crate::BasisPoints::new(bps)
                    .map_err(|_| WorldError::ArithmeticOverflow("deprivation bounds"))?,
            );
        }
        self.monthly_consumption = consumption;
        self.unmet_demand = clearing.unmet.clone();
        self.deprivation_pressure = pressure;
        for fill in &clearing.fills {
            self.events.append(
                self.date,
                crate::DomainEvent::MarketTrade {
                    buyer: fill.buyer,
                    seller: fill.seller,
                    good: fill.good,
                    quantity: fill.quantity,
                    spend: fill.spend,
                },
            );
        }
        Ok(())
    }
}

impl crate::World {
    #[must_use]
    pub fn monthly_consumption(
        &self,
    ) -> &std::collections::BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli> {
        &self.monthly_consumption
    }
    #[must_use]
    pub fn unmet_demand(&self) -> &BTreeUnmet {
        &self.unmet_demand
    }
    #[must_use]
    pub fn deprivation_pressure(
        &self,
    ) -> &std::collections::BTreeMap<CohortId, crate::BasisPoints> {
        &self.deprivation_pressure
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scarce_supply_fills_buyers_in_canonical_order() {
        let orders = [
            MarketOrder {
                buyer: CohortId::new(2),
                tier: NeedTier::Survival,
                region: RegionId::new(1),
                good: GoodId::new(1),
                quantity: QuantityMilli::new(700),
                max_spend: Money::from_minor_units(100),
            },
            MarketOrder {
                buyer: CohortId::new(1),
                tier: NeedTier::Survival,
                region: RegionId::new(1),
                good: GoodId::new(1),
                quantity: QuantityMilli::new(700),
                max_spend: Money::from_minor_units(100),
            },
        ];
        let offers = [MarketOffer {
            seller: FirmId::new(1),
            region: RegionId::new(1),
            good: GoodId::new(1),
            quantity: QuantityMilli::new(1000),
            unit_price: Money::from_minor_units(10),
        }];
        let result = clear_local_market(&orders, &offers).expect("clear");
        assert_eq!(result.fills[0].buyer, CohortId::new(1));
        assert_eq!(
            result.unmet[&(CohortId::new(2), GoodId::new(1), NeedTier::Survival)].get(),
            400
        );
    }
}
