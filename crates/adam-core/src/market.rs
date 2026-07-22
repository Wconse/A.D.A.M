use crate::{CohortId, FirmId, GoodId, Money, QuantityMilli, RegionId, WorldError};
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
    pub region: RegionId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub max_spend: Money,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarketFill {
    pub buyer: CohortId,
    pub seller: FirmId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub spend: Money,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketClearing {
    pub fills: Vec<MarketFill>,
    pub unmet: BTreeUnmet,
}
pub type BTreeUnmet = std::collections::BTreeMap<(CohortId, GoodId), QuantityMilli>;
/// Clears local goods markets deterministically by region/good, then order ID, price, and seller ID.
/// # Errors
/// Returns an error for non-positive prices or arithmetic overflow.
pub fn clear_local_market(
    orders: &[MarketOrder],
    offers: &[MarketOffer],
) -> Result<MarketClearing, WorldError> {
    let mut orders = orders.to_vec();
    orders.sort_by_key(|o| (o.region, o.good, o.buyer));
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
                seller: offer.seller,
                good: order.good,
                quantity: QuantityMilli::new(quantity),
                spend: Money::from_minor_units(spend),
            });
        }
        if need > 0 {
            unmet.insert((order.buyer, order.good), QuantityMilli::new(need));
        }
    }
    Ok(MarketClearing { fills, unmet })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scarce_supply_fills_buyers_in_canonical_order() {
        let orders = [
            MarketOrder {
                buyer: CohortId::new(2),
                region: RegionId::new(1),
                good: GoodId::new(1),
                quantity: QuantityMilli::new(700),
                max_spend: Money::from_minor_units(100),
            },
            MarketOrder {
                buyer: CohortId::new(1),
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
        assert_eq!(result.unmet[&(CohortId::new(2), GoodId::new(1))].get(), 400);
    }
}
