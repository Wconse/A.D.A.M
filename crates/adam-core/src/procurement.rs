use std::collections::BTreeMap;

use crate::{FirmId, GoodId, MarketOffer, Money, QuantityMilli, RegionId, World, WorldError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmProcurementOrder {
    pub buyer: FirmId,
    pub region: RegionId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmProcurementFill {
    pub buyer: FirmId,
    pub seller: FirmId,
    pub good: GoodId,
    pub quantity: QuantityMilli,
    pub spend: Money,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmProcurementResult {
    pub orders: Vec<FirmProcurementOrder>,
    pub fills: Vec<FirmProcurementFill>,
    pub unmet: BTreeMap<(FirmId, GoodId), QuantityMilli>,
}

/// Outcome of the settle phase: what moved, what stayed unmet, and the
/// evidence the record phase needs. Behaviour is locked by
/// crates/adam-core/tests/intermediate_procurement.rs.
struct ProcurementSettlement {
    fills: Vec<FirmProcurementFill>,
    unmet: BTreeMap<(FirmId, GoodId), QuantityMilli>,
    trade_prices: BTreeMap<(FirmId, GoodId), Money>,
    purchases: BTreeMap<(FirmId, GoodId), (u64, i64)>,
}

impl World {
    pub(crate) fn plan_monthly_firm_procurement(
        &self,
    ) -> Result<Vec<FirmProcurementOrder>, WorldError> {
        let plans = self.plan_monthly_production_requirements()?;
        let mut orders = Vec::new();
        for plan in plans {
            let firm = self
                .firms()
                .get(&plan.firm())
                .ok_or(WorldError::UnknownFirm(plan.firm()))?;
            let recipe = self
                .production_recipes()
                .get(&firm.recipe())
                .ok_or(WorldError::UnknownRecipe(firm.recipe()))?;
            for input in recipe.inputs() {
                let required = input
                    .quantity_per_batch()
                    .get()
                    .checked_mul(plan.batches())
                    .ok_or(WorldError::ArithmeticOverflow(
                        "firm procurement requirement",
                    ))?;
                let available = firm
                    .inventories()
                    .get(&input.good())
                    .copied()
                    .unwrap_or_default()
                    .get();
                if required > available {
                    orders.push(FirmProcurementOrder {
                        buyer: firm.id(),
                        region: firm.region(),
                        good: input.good(),
                        quantity: QuantityMilli::new(required - available),
                    });
                }
            }
        }
        orders.sort_by_key(|order| (order.region, order.good, order.buyer));
        Ok(orders)
    }

    /// Plan phase: the canonical, deterministically ordered offer book for
    /// intermediate firm-to-firm trade. Pure read; no state changes.
    fn plan_procurement_offer_book(&self) -> Result<Vec<MarketOffer>, WorldError> {
        let mut offers: Vec<MarketOffer> = self
            .plan_firm_market_offers()?
            .into_iter()
            .filter_map(|plan| {
                (plan.inventory.get() > 0).then_some(MarketOffer {
                    seller: plan.seller,
                    region: plan.region,
                    good: plan.good,
                    quantity: plan.inventory,
                    unit_price: plan.unit_price,
                })
            })
            .collect();
        offers.sort_by_key(|offer| {
            (
                offer.region,
                offer.good,
                offer.unit_price.minor_units(),
                offer.seller,
            )
        });
        Ok(offers)
    }

    /// Settle phase: match orders against the offer book and move cash and
    /// inventories. All movement happens on a clone of the firm ledger and is
    /// committed only after the whole month settles, so any error leaves the
    /// world untouched.
    #[allow(clippy::too_many_lines)]
    fn settle_procurement_orders(
        &mut self,
        orders: &[FirmProcurementOrder],
        offers: &mut [MarketOffer],
    ) -> Result<ProcurementSettlement, WorldError> {
        let mut firms = self.firms().clone();
        let mut fills = Vec::new();
        let mut unmet = BTreeMap::new();
        let mut trade_prices: BTreeMap<(FirmId, GoodId), Money> = BTreeMap::new();
        let mut purchases: BTreeMap<(FirmId, GoodId), (u64, i64)> = BTreeMap::new();
        for order in orders {
            let mut remaining = order.quantity.get();
            for offer in offers.iter_mut().filter(|offer| {
                offer.region == order.region
                    && offer.good == order.good
                    && offer.seller != order.buyer
            }) {
                if remaining == 0 || offer.quantity.get() == 0 {
                    break;
                }
                let buyer_cash = firms[&order.buyer].cash().minor_units().max(0);
                let price = offer.unit_price.minor_units();
                if price <= 0 || buyer_cash == 0 {
                    break;
                }
                let affordable = i128::from(buyer_cash)
                    .checked_mul(i128::from(QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow("firm procurement budget"))?
                    / i128::from(price);
                let quantity = remaining
                    .min(offer.quantity.get())
                    .min(u64::try_from(affordable).unwrap_or(u64::MAX));
                if quantity == 0 {
                    continue;
                }
                let spend = i128::from(price)
                    .checked_mul(i128::from(quantity))
                    .ok_or(WorldError::ArithmeticOverflow("firm procurement spend"))?
                    / i128::from(QuantityMilli::SCALE);
                let spend = Money::from_minor_units(
                    i64::try_from(spend)
                        .map_err(|_| WorldError::ArithmeticOverflow("firm procurement spend"))?,
                );
                firms
                    .get_mut(&order.buyer)
                    .ok_or(WorldError::UnknownFirm(order.buyer))?
                    .debit_cash(spend)?;
                firms
                    .get_mut(&offer.seller)
                    .ok_or(WorldError::UnknownFirm(offer.seller))?
                    .debit_inventory(order.good, QuantityMilli::new(quantity))?;
                firms
                    .get_mut(&order.buyer)
                    .ok_or(WorldError::UnknownFirm(order.buyer))?
                    .credit_inventory(order.good, QuantityMilli::new(quantity))?;
                firms
                    .get_mut(&offer.seller)
                    .ok_or(WorldError::UnknownFirm(offer.seller))?
                    .apply_cash_delta(spend)?;
                offer.quantity = QuantityMilli::new(offer.quantity.get() - quantity);
                trade_prices.insert((offer.seller, order.good), offer.unit_price);
                let purchase = purchases.entry((order.buyer, order.good)).or_default();
                purchase.0 =
                    purchase
                        .0
                        .checked_add(quantity)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "firm procurement purchase quantity",
                        ))?;
                purchase.1 = purchase.1.checked_add(spend.minor_units()).ok_or(
                    WorldError::ArithmeticOverflow("firm procurement purchase spend"),
                )?;
                remaining -= quantity;
                fills.push(FirmProcurementFill {
                    buyer: order.buyer,
                    seller: offer.seller,
                    good: order.good,
                    quantity: QuantityMilli::new(quantity),
                    spend,
                });
            }
            if remaining > 0 {
                unmet.insert((order.buyer, order.good), QuantityMilli::new(remaining));
            }
        }
        self.firms = firms;
        Ok(ProcurementSettlement {
            fills,
            unmet,
            trade_prices,
            purchases,
        })
    }

    /// Record phase: persist purchase aggregates, append trade events, and
    /// fold settled sales into firm observations and revenue.
    fn record_procurement_outcomes(
        &mut self,
        settlement: &ProcurementSettlement,
    ) -> Result<(), WorldError> {
        for (&(buyer, good), &(quantity, spend)) in &settlement.purchases {
            self.monthly_firm_procurement_purchases.insert(
                (buyer, good),
                (QuantityMilli::new(quantity), Money::from_minor_units(spend)),
            );
        }
        let mut revenue = BTreeMap::<FirmId, i128>::new();
        let mut sold = BTreeMap::<(FirmId, GoodId), u64>::new();
        for fill in &settlement.fills {
            *revenue.entry(fill.seller).or_default() += i128::from(fill.spend.minor_units());
            let quantity = sold.entry((fill.seller, fill.good)).or_default();
            *quantity = quantity
                .checked_add(fill.quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("firm procurement sales"))?;
            self.events.append(
                self.date,
                crate::DomainEvent::FirmProcurementTrade {
                    buyer: fill.buyer,
                    seller: fill.seller,
                    good: fill.good,
                    quantity: fill.quantity,
                    spend: fill.spend,
                },
            );
        }
        for ((firm, good), quantity) in sold {
            let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
            let unit_price = settlement
                .trade_prices
                .get(&(firm, good))
                .copied()
                .expect("settled procurement sale must have a recorded trade price");
            self.monthly_firm_market_outcomes
                .entry(firm)
                .or_default()
                .push(crate::MarketOfferOutcome {
                    seller: firm,
                    region: definition.region(),
                    good,
                    unit_price,
                    offered: QuantityMilli::new(quantity),
                    sold: QuantityMilli::new(quantity),
                    unsold: QuantityMilli::default(),
                    unmet_market_demand: QuantityMilli::default(),
                });
        }
        for (firm, value) in revenue {
            self.record_firm_sale(
                firm,
                Money::from_minor_units(
                    i64::try_from(value)
                        .map_err(|_| WorldError::ArithmeticOverflow("firm procurement revenue"))?,
                ),
            )?;
        }
        Ok(())
    }

    /// One monthly procurement cycle: plan the offer book, settle orders
    /// against it, then record the evidence trail.
    pub(crate) fn execute_monthly_firm_procurement(
        &mut self,
    ) -> Result<FirmProcurementResult, WorldError> {
        let orders = self.plan_monthly_firm_procurement()?;
        let mut offers = self.plan_procurement_offer_book()?;
        let settlement = self.settle_procurement_orders(&orders, &mut offers)?;
        self.record_procurement_outcomes(&settlement)?;
        Ok(FirmProcurementResult {
            orders,
            fills: settlement.fills,
            unmet: settlement.unmet,
        })
    }
}
