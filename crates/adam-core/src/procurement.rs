use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BasisPoints, CohortId, CountryId, FirmId, GoodId, MarketOffer, Money, NeedTier, QuantityMilli,
    RegionId, RouteId, World, WorldError,
};

/// Monthly grievance accrual applied when a country's firm ends procurement
/// with unmet input demand while a foreign supplier offered the good over a
/// route that could deliver it in peace.
const GRIEVANCE_ACCRUAL_BPS: u16 = 500;
/// Monthly grievance decay applied to pairs without fresh material evidence.
const GRIEVANCE_DECAY_BPS: u16 = 250;
/// Grievance level at which the aggrieved country activates the ordinary
/// journaled bilateral hostility relation.
const GRIEVANCE_HOSTILITY_THRESHOLD_BPS: u16 = 7_500;

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
    /// Total delivered payment debited from the buyer.
    pub spend: Money,
    /// Goods-price component credited to the supplier.
    pub goods_spend: Money,
    /// Route-tariff component credited to the carrier.
    pub freight_spend: Money,
    pub route: Option<RouteId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirmProcurementResult {
    pub orders: Vec<FirmProcurementOrder>,
    pub fills: Vec<FirmProcurementFill>,
    pub unmet: BTreeMap<(FirmId, GoodId), QuantityMilli>,
    /// Unmet inputs for which a reachable foreign offer remained, but the
    /// monthly route-capacity pool was exhausted before it could fill.
    pub(crate) capacity_limited: BTreeSet<(FirmId, GoodId)>,
    pub(crate) constrained_routes: BTreeSet<RouteId>,
    pub(crate) pristine_offers: Vec<MarketOffer>,
}

/// Outcome of the settle phase: what moved, what stayed unmet, and the
/// evidence the record phase needs. Behaviour is locked by
/// crates/adam-core/tests/intermediate_procurement.rs.
struct ProcurementSettlement {
    fills: Vec<FirmProcurementFill>,
    unmet: BTreeMap<(FirmId, GoodId), QuantityMilli>,
    capacity_limited: BTreeSet<(FirmId, GoodId)>,
    constrained_routes: BTreeSet<RouteId>,
    trade_prices: BTreeMap<(FirmId, GoodId), Money>,
    purchases: BTreeMap<(FirmId, GoodId), (u64, i64)>,
}

struct RecordedProcurementFills {
    goods_revenue: BTreeMap<FirmId, i128>,
    freight_revenue: BTreeMap<FirmId, i128>,
    sold: BTreeMap<(FirmId, GoodId), u64>,
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

    /// Selects the lowest-tariff direct peaceful route identity and tariff for
    /// an immediate market import. Equal tariffs resolve by stable route ID.
    pub(crate) fn direct_market_route(
        &self,
        origin: RegionId,
        destination: RegionId,
    ) -> Option<(crate::RouteId, Money)> {
        let origin_country = self.regions.get(&origin)?.country();
        let destination_country = self.regions.get(&destination)?.country();
        if self.countries_are_hostile(origin_country, destination_country) {
            return None;
        }
        self.logistics_routes
            .values()
            .filter(|route| route.origin() == origin && route.destination() == destination)
            .map(|route| (route.cost_per_unit(), route.id()))
            .min_by_key(|(cost, route)| (cost.minor_units(), *route))
            .map(|(cost, route)| (route, cost))
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
        route_capacity: &mut BTreeMap<RouteId, u64>,
    ) -> Result<ProcurementSettlement, WorldError> {
        let mut firms = self.firms().clone();
        let mut fills = Vec::new();
        let mut unmet = BTreeMap::new();
        let mut capacity_limited = BTreeSet::new();
        let mut constrained_routes = BTreeSet::new();
        let mut trade_prices: BTreeMap<(FirmId, GoodId), Money> = BTreeMap::new();
        let mut purchases: BTreeMap<(FirmId, GoodId), (u64, i64)> = BTreeMap::new();
        for order in orders {
            let mut remaining = order.quantity.get();
            // Local offers keep priority in book order (price ascending);
            // import offers follow, ordered by delivered price (offer price
            // plus route tariff) with stable region and seller tie-breaks.
            // Imports only ever see the remainder local sellers cannot fill.
            // Import fills are capped by the shared monthly spot route
            // capacity pool that firms and households compete over together.
            let mut candidates: Vec<(usize, Money, Option<RouteId>)> = Vec::new();
            let mut imports: Vec<(i64, RegionId, FirmId, usize, Money, RouteId)> = Vec::new();
            for (index, offer) in offers.iter().enumerate() {
                if offer.good != order.good
                    || offer.seller == order.buyer
                    || offer.quantity.get() == 0
                {
                    continue;
                }
                if offer.region == order.region {
                    candidates.push((index, Money::default(), None));
                } else if let Some((route, tariff)) =
                    self.direct_market_route(offer.region, order.region)
                {
                    let delivered = offer
                        .unit_price
                        .minor_units()
                        .checked_add(tariff.minor_units())
                        .ok_or(WorldError::ArithmeticOverflow(
                            "firm procurement delivered price",
                        ))?;
                    imports.push((delivered, offer.region, offer.seller, index, tariff, route));
                }
            }
            imports
                .sort_by_key(|&(delivered, region, seller, _, _, _)| (delivered, region, seller));
            candidates.extend(
                imports
                    .into_iter()
                    .map(|(_, _, _, index, tariff, route)| (index, tariff, Some(route))),
            );
            let mut exhausted_route_capacity = false;
            for (index, tariff, route) in candidates {
                if remaining == 0 {
                    break;
                }
                let offer = &mut offers[index];
                if offer.quantity.get() == 0 {
                    continue;
                }
                let buyer_cash = firms[&order.buyer].cash().minor_units().max(0);
                if buyer_cash == 0 {
                    break;
                }
                let price = offer
                    .unit_price
                    .minor_units()
                    .checked_add(tariff.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow(
                        "firm procurement delivered price",
                    ))?;
                if price <= 0 {
                    continue;
                }
                let affordable = i128::from(buyer_cash)
                    .checked_mul(i128::from(QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow("firm procurement budget"))?
                    / i128::from(price);
                let route_limit = route.map_or(u64::MAX, |id| {
                    route_capacity.get(&id).copied().unwrap_or(u64::MAX)
                });
                let unconstrained = remaining
                    .min(offer.quantity.get())
                    .min(u64::try_from(affordable).unwrap_or(u64::MAX));
                if let Some(route_id) = route {
                    if route_limit < unconstrained {
                        constrained_routes.insert(route_id);
                    }
                }
                let quantity = unconstrained.min(route_limit);
                if route.is_some() && route_limit == 0 {
                    exhausted_route_capacity = true;
                }
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
                let goods_spend = Money::from_minor_units(
                    i64::try_from(
                        i128::from(offer.unit_price.minor_units())
                            .checked_mul(i128::from(quantity))
                            .ok_or(WorldError::ArithmeticOverflow(
                                "firm procurement goods spend",
                            ))?
                            / i128::from(QuantityMilli::SCALE),
                    )
                    .map_err(|_| WorldError::ArithmeticOverflow("firm procurement goods spend"))?,
                );
                let freight_spend = Money::from_minor_units(
                    spend
                        .minor_units()
                        .checked_sub(goods_spend.minor_units())
                        .ok_or(WorldError::ArithmeticOverflow(
                            "firm procurement freight spend",
                        ))?,
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
                    .apply_cash_delta(goods_spend)?;
                if let Some(route_id) = route {
                    let carrier = self
                        .logistics_routes
                        .get(&route_id)
                        .and_then(crate::LogisticsRoute::carrier)
                        .ok_or(WorldError::InvalidLogistics(
                            "procurement import route requires a carrier",
                        ))?;
                    firms
                        .get_mut(&carrier)
                        .ok_or(WorldError::UnknownFirm(carrier))?
                        .apply_cash_delta(freight_spend)?;
                }
                offer.quantity = QuantityMilli::new(offer.quantity.get() - quantity);
                if let Some(id) = route {
                    if let Some(cap) = route_capacity.get_mut(&id) {
                        *cap -= quantity;
                    }
                    exhausted_route_capacity |= route_capacity
                        .get(&id)
                        .is_some_and(|capacity| *capacity == 0)
                        && offer.quantity.get() > 0;
                }
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
                    goods_spend,
                    freight_spend,
                    route,
                });
            }
            if remaining > 0 {
                if exhausted_route_capacity {
                    capacity_limited.insert((order.buyer, order.good));
                }
                unmet.insert((order.buyer, order.good), QuantityMilli::new(remaining));
            }
        }
        self.firms = firms;
        Ok(ProcurementSettlement {
            fills,
            unmet,
            capacity_limited,
            constrained_routes,
            trade_prices,
            purchases,
        })
    }

    fn record_procurement_fills(
        &mut self,
        fills: &[FirmProcurementFill],
    ) -> Result<RecordedProcurementFills, WorldError> {
        let mut goods_revenue = BTreeMap::<FirmId, i128>::new();
        let mut freight_revenue = BTreeMap::<FirmId, i128>::new();
        let mut sold = BTreeMap::<(FirmId, GoodId), u64>::new();
        for fill in fills {
            *goods_revenue.entry(fill.seller).or_default() +=
                i128::from(fill.goods_spend.minor_units());
            if let Some(route) = fill.route {
                let carrier = self
                    .logistics_routes
                    .get(&route)
                    .and_then(crate::LogisticsRoute::carrier)
                    .ok_or(WorldError::InvalidLogistics(
                        "procurement import route requires a carrier",
                    ))?;
                *freight_revenue.entry(carrier).or_default() +=
                    i128::from(fill.freight_spend.minor_units());
            }
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
            if let Some(route) = fill.route {
                let carrier = self
                    .logistics_routes
                    .get(&route)
                    .and_then(crate::LogisticsRoute::carrier)
                    .ok_or(WorldError::InvalidLogistics(
                        "procurement import route requires a carrier",
                    ))?;
                self.events.append(
                    self.date,
                    crate::DomainEvent::FirmProcurementFreightPaid {
                        buyer: fill.buyer,
                        seller: fill.seller,
                        carrier,
                        route,
                        amount: fill.freight_spend,
                    },
                );
            }
        }
        Ok(RecordedProcurementFills {
            goods_revenue,
            freight_revenue,
            sold,
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
        let recorded = self.record_procurement_fills(&settlement.fills)?;
        for (&(buyer, good), &quantity) in &settlement.unmet {
            let event = if settlement.capacity_limited.contains(&(buyer, good)) {
                crate::DomainEvent::FirmProcurementRouteCapacityShortfall {
                    buyer,
                    good,
                    quantity,
                }
            } else {
                crate::DomainEvent::FirmProcurementShortfall {
                    buyer,
                    good,
                    quantity,
                }
            };
            self.events.append(self.date, event);
        }
        for ((firm, good), quantity) in recorded.sold {
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
        for (firm, value) in recorded.goods_revenue {
            self.record_firm_sale(
                firm,
                Money::from_minor_units(i64::try_from(value).map_err(|_| {
                    WorldError::ArithmeticOverflow("firm procurement goods revenue")
                })?),
            )?;
        }
        for (firm, value) in recorded.freight_revenue {
            self.record_firm_sale(
                firm,
                Money::from_minor_units(i64::try_from(value).map_err(|_| {
                    WorldError::ArithmeticOverflow("firm procurement freight revenue")
                })?),
            )?;
        }
        Ok(())
    }

    /// True when any registered route connects the regions directly, ignoring
    /// hostility. Grievance accrual asks what peace could have delivered.
    fn peaceful_direct_route_exists(&self, origin: RegionId, destination: RegionId) -> bool {
        self.logistics_routes
            .values()
            .any(|route| route.origin() == origin && route.destination() == destination)
    }

    /// Adds directed grievance evidence from unmet household survival needs.
    fn collect_household_grievance_evidence(
        &self,
        household_unmet: &BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli>,
        household_offers: &[MarketOffer],
        accrued: &mut BTreeSet<(CountryId, CountryId)>,
    ) -> Result<(), WorldError> {
        for &(cohort, good, tier) in household_unmet.keys() {
            if tier != NeedTier::Survival {
                continue;
            }
            let household = self
                .cohorts
                .get(&cohort)
                .ok_or(WorldError::UnknownCohort(cohort))?;
            let household_region = household.region();
            let household_country = self
                .regions
                .get(&household_region)
                .ok_or(WorldError::UnknownRegion(household_region))?
                .country();
            for offer in household_offers {
                if offer.good != good || offer.quantity.get() == 0 {
                    continue;
                }
                let seller_country = self
                    .regions
                    .get(&offer.region)
                    .ok_or(WorldError::UnknownRegion(offer.region))?
                    .country();
                if seller_country != household_country
                    && self.peaceful_direct_route_exists(offer.region, household_region)
                {
                    accrued.insert((household_country, seller_country));
                }
            }
        }
        Ok(())
    }

    /// Accrues, decays, and escalates bounded bilateral grievances from this
    /// month's firm and household shortage evidence.
    ///
    /// A country accrues only when a firm input or household survival need
    /// stayed unmet while a foreign firm offered that good over a direct route
    /// that could deliver it in peace. Pairs without fresh evidence decay
    /// toward zero and are dropped at zero. Crossing the threshold activates
    /// the ordinary journaled hostility transition, so conflict keeps a
    /// material cause.
    pub(crate) fn update_bilateral_grievances(
        &mut self,
        firm_unmet: &BTreeMap<(FirmId, GoodId), QuantityMilli>,
        firm_capacity_limited: &BTreeSet<(FirmId, GoodId)>,
        firm_offers: &[MarketOffer],
        household_unmet: &BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli>,
        household_offers: &[MarketOffer],
    ) -> Result<(), WorldError> {
        let mut accrued = BTreeSet::new();
        for &(buyer, good) in firm_unmet.keys() {
            // An exhausted shared route pool is a domestic/logistics scarcity,
            // not evidence that the foreign supplier withheld a deliverable good.
            if firm_capacity_limited.contains(&(buyer, good)) {
                continue;
            }
            let buyer_firm = self
                .firms
                .get(&buyer)
                .ok_or(WorldError::UnknownFirm(buyer))?;
            let buyer_region = buyer_firm.region();
            let buyer_country = self
                .regions
                .get(&buyer_region)
                .ok_or(WorldError::UnknownRegion(buyer_region))?
                .country();
            for offer in firm_offers {
                if offer.good != good || offer.seller == buyer || offer.quantity.get() == 0 {
                    continue;
                }
                let seller_country = self
                    .regions
                    .get(&offer.region)
                    .ok_or(WorldError::UnknownRegion(offer.region))?
                    .country();
                if seller_country == buyer_country {
                    continue;
                }
                if self.peaceful_direct_route_exists(offer.region, buyer_region) {
                    accrued.insert((buyer_country, seller_country));
                }
            }
        }
        self.collect_household_grievance_evidence(household_unmet, household_offers, &mut accrued)?;
        let tracked: BTreeSet<(CountryId, CountryId)> = self
            .bilateral_grievances
            .keys()
            .copied()
            .chain(accrued.iter().copied())
            .collect();
        for pair in tracked {
            let current = self
                .bilateral_grievances
                .get(&pair)
                .copied()
                .map_or(0, BasisPoints::get);
            let next = if accrued.contains(&pair) {
                current
                    .saturating_add(GRIEVANCE_ACCRUAL_BPS)
                    .min(BasisPoints::MAX)
            } else {
                current.saturating_sub(GRIEVANCE_DECAY_BPS)
            };
            if next != current {
                let level = BasisPoints::new(next)
                    .expect("grievance level is clamped to the basis point range");
                if next == 0 {
                    self.bilateral_grievances.remove(&pair);
                } else {
                    self.bilateral_grievances.insert(pair, level);
                }
                self.events.append(
                    self.date,
                    crate::DomainEvent::BilateralGrievanceChanged {
                        aggrieved: pair.0,
                        target: pair.1,
                        level,
                    },
                );
            }
            if next >= GRIEVANCE_HOSTILITY_THRESHOLD_BPS
                && !self.countries_are_hostile(pair.0, pair.1)
            {
                self.set_country_hostility(pair.0, pair.1, true)?;
                self.mark_emergent_hostility(pair.0, pair.1);
            }
        }
        self.deescalate_resolved_emergent_hostilities()?;
        Ok(())
    }

    /// Clears emergent hostility once its material cause has decayed away.
    ///
    /// A pair de-escalates only when neither directed grievance survives, and
    /// only when the hostility entered through grievance escalation; commanded
    /// hostility stays until it is commanded away.
    fn deescalate_resolved_emergent_hostilities(&mut self) -> Result<(), WorldError> {
        let resolved: Vec<(CountryId, CountryId)> = self
            .emergent_hostilities
            .iter()
            .copied()
            .filter(|&(first, second)| {
                !self.bilateral_grievances.contains_key(&(first, second))
                    && !self.bilateral_grievances.contains_key(&(second, first))
            })
            .collect();
        for (first, second) in resolved {
            self.set_country_hostility(first, second, false)?;
        }
        Ok(())
    }

    /// One monthly procurement cycle: plan the offer book, settle orders
    /// against it, record the evidence trail, then update material grievances.
    pub(crate) fn execute_monthly_firm_procurement(
        &mut self,
        route_capacity: &mut BTreeMap<RouteId, u64>,
    ) -> Result<FirmProcurementResult, WorldError> {
        let orders = self.plan_monthly_firm_procurement()?;
        let mut offers = self.plan_procurement_offer_book()?;
        let pristine_offers = offers.clone();
        let settlement = self.settle_procurement_orders(&orders, &mut offers, route_capacity)?;
        self.record_procurement_outcomes(&settlement)?;
        Ok(FirmProcurementResult {
            orders,
            fills: settlement.fills,
            unmet: settlement.unmet,
            capacity_limited: settlement.capacity_limited,
            constrained_routes: settlement.constrained_routes,
            pristine_offers,
        })
    }
}
