use std::collections::{BTreeMap, BTreeSet};

use crate::money_ledger::MoneyAccountBalances;
use crate::{
    BasisPoints, CohortId, CountryId, FirmId, GoodId, MarketOffer, Money, MoneyAccountId,
    MoneyPosting, MoneyTransaction, MoneyTransactionPurpose, NeedTier, QuantityMilli, RegionId,
    RouteId, World, WorldError,
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
    /// Buyers that refused a price rather than failed to find a seller.
    pub(crate) refused_on_price: BTreeSet<(FirmId, GoodId)>,
    /// Buyers that ran out of money rather than failed to find a seller.
    pub(crate) starved_of_cash: BTreeSet<(FirmId, GoodId)>,
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
    /// Buyer demand that left this market empty-handed, keyed the way the
    /// household clearing keys it: by the region that wanted the good.
    unmet_by_market: BTreeMap<(RegionId, GoodId), u64>,
    /// Sellers whose entire offer of a good was taken this month. Only a
    /// seller who actually ran out can claim the queue as its own evidence.
    exhausted_sellers: BTreeSet<(FirmId, GoodId)>,
    /// The part of `unmet_by_market` that was effective demand: the buyer had
    /// money left, refused nothing on price, and still found no seller within
    /// reach holding the good. Only this may move a price.
    effective_unmet_by_market: BTreeMap<(RegionId, GoodId), u64>,
    /// Buyers that walked away from a good someone was holding because the
    /// asking price had passed what the good is worth to them.
    refused_on_price: BTreeSet<(FirmId, GoodId)>,
    /// Buyers whose till ran dry before their need was covered.
    starved_of_cash: BTreeSet<(FirmId, GoodId)>,
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

    /// The most a buyer can justify paying for one unit of an input, judged by
    /// the local price of what that input becomes.
    ///
    /// A batch is worth its output at local prices; the other materials it needs
    /// are already owed to their own sellers; what is left is the most this
    /// input can earn back. Paying above it is a loss agreed to in advance, and
    /// a buyer that does it every month bleeds to death while telling the price
    /// mechanism it is thriving.
    ///
    /// Wages are deliberately not deducted here: what the work is worth is
    /// judged by its own ceiling in `labor`, and deducting a wage this buyer may
    /// not even be paying would put two rules in charge of one decision. The
    /// bound is therefore generous, which is the right direction for a refusal.
    ///
    /// Returns `None` when the output or another input has no local price, in
    /// which case there is no evidence to judge the purchase by and the buyer
    /// bids as it always did.
    pub(crate) fn input_value_ceiling(
        &self,
        buyer: FirmId,
        good: GoodId,
    ) -> Result<Option<Money>, WorldError> {
        let firm = self
            .firms
            .get(&buyer)
            .ok_or(WorldError::UnknownFirm(buyer))?;
        let region = firm.region();
        let Some(recipe) = self.production_recipes.get(&firm.recipe()) else {
            return Ok(None);
        };
        let Some(output_price) = self
            .regional_prices
            .get(&(region, recipe.output_good()))
            .copied()
        else {
            return Ok(None);
        };
        let mut per_batch = 0_u64;
        let mut other_materials = 0_i64;
        for input in recipe.inputs() {
            if input.good() == good {
                per_batch = per_batch.saturating_add(input.quantity_per_batch().get());
                continue;
            }
            let Some(price) = self.regional_prices.get(&(region, input.good())).copied() else {
                return Ok(None);
            };
            other_materials = other_materials
                .checked_add(
                    crate::firm_entry::quantity_value(price, input.quantity_per_batch())?
                        .minor_units(),
                )
                .ok_or(WorldError::ArithmeticOverflow(
                    "input value other materials",
                ))?;
        }
        if per_batch == 0 {
            return Ok(None);
        }
        let revenue = crate::firm_entry::quantity_value(output_price, recipe.output_per_batch())?
            .minor_units();
        let residual = revenue.saturating_sub(other_materials).max(0);
        let ceiling = i128::from(residual)
            .checked_mul(i128::from(QuantityMilli::SCALE))
            .ok_or(WorldError::ArithmeticOverflow("input value ceiling"))?
            / i128::from(per_batch);
        Ok(Some(Money::from_minor_units(
            i64::try_from(ceiling)
                .map_err(|_| WorldError::ArithmeticOverflow("input value ceiling"))?,
        )))
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
        let mut transactions = Vec::new();
        let mut unmet = BTreeMap::new();
        let mut capacity_limited = BTreeSet::new();
        let mut refused_purchases = BTreeSet::new();
        let mut cash_starved = BTreeSet::new();
        let mut constrained_routes = BTreeSet::new();
        let mut trade_prices: BTreeMap<(FirmId, GoodId), Money> = BTreeMap::new();
        let mut purchases: BTreeMap<(FirmId, GoodId), (u64, i64)> = BTreeMap::new();
        let mut unmet_by_market: BTreeMap<(RegionId, GoodId), u64> = BTreeMap::new();
        let mut effective_unmet_by_market: BTreeMap<(RegionId, GoodId), u64> = BTreeMap::new();
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
            let mut refused_on_price = false;
            let mut cash_bound = false;
            // What one unit of this input is worth to this buyer at local
            // prices. A mill that pays more for grain than the flour it becomes
            // can fetch has booked its loss before it starts work, and no mill
            // does that twice. Refusing is not a shortage; it is the buyer
            // telling the market the price has passed the point of sense.
            let ceiling = self.input_value_ceiling(order.buyer, order.good)?;
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
                    // An empty till is not a bare shelf. A buyer with nothing to
                    // spend cannot bid anything up, however badly it needs the
                    // good, which is the same rule households already live by.
                    cash_bound = true;
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
                if let Some(limit) = ceiling {
                    if price > limit.minor_units() {
                        refused_on_price = true;
                        continue;
                    }
                }
                let affordable = i128::from(buyer_cash)
                    .checked_mul(i128::from(QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow("firm procurement budget"))?
                    / i128::from(price);
                let route_limit = route.map_or(u64::MAX, |id| {
                    route_capacity.get(&id).copied().unwrap_or(u64::MAX)
                });
                let affordable_units = u64::try_from(affordable).unwrap_or(u64::MAX);
                let wanted_here = remaining.min(offer.quantity.get());
                if affordable_units < wanted_here {
                    // The shelf held what was wanted; the purse did not reach it.
                    cash_bound = true;
                }
                let unconstrained = wanted_here.min(affordable_units);
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
                let carrier = route
                    .map(|route_id| {
                        self.logistics_routes
                            .get(&route_id)
                            .and_then(crate::LogisticsRoute::carrier)
                            .ok_or(WorldError::InvalidLogistics(
                                "procurement import route requires a carrier",
                            ))
                    })
                    .transpose()?;
                firms
                    .get_mut(&offer.seller)
                    .ok_or(WorldError::UnknownFirm(offer.seller))?
                    .debit_inventory(order.good, QuantityMilli::new(quantity))?;
                firms
                    .get_mut(&order.buyer)
                    .ok_or(WorldError::UnknownFirm(order.buyer))?
                    .credit_inventory(order.good, QuantityMilli::new(quantity))?;
                if spend.minor_units() == 0 {
                    if goods_spend.minor_units() != 0 || freight_spend.minor_units() != 0 {
                        return Err(WorldError::InvalidMarketClearing(
                            "zero-spend procurement fill cannot contain monetary components",
                        ));
                    }
                } else {
                    let mut postings = vec![MoneyPosting::decrease(
                        MoneyAccountId::FirmOperating(order.buyer),
                        spend,
                    )];
                    if goods_spend.minor_units() > 0 {
                        postings.push(MoneyPosting::increase(
                            MoneyAccountId::FirmOperating(offer.seller),
                            goods_spend,
                        ));
                    }
                    if freight_spend.minor_units() > 0 {
                        postings.push(MoneyPosting::increase(
                            MoneyAccountId::FirmOperating(carrier.ok_or(
                                WorldError::InvalidLogistics(
                                    "procurement freight spend requires a carrier account",
                                ),
                            )?),
                            freight_spend,
                        ));
                    }
                    let transaction = MoneyTransaction::new(
                        MoneyTransactionPurpose::FirmPurchase {
                            buyer: order.buyer,
                            seller: offer.seller,
                            good: order.good,
                            carrier,
                        },
                        postings,
                    )?;
                    MoneyAccountBalances::Firms { firms: &mut firms }.settle(&transaction)?;
                    transactions.push(transaction);
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
                // A buyer that went away empty is the same evidence here as in
                // the household clearing, and it belongs to the market that
                // wanted the good.
                let market = unmet_by_market
                    .entry((order.region, order.good))
                    .or_default();
                *market = market
                    .checked_add(remaining)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "procurement unmet market demand",
                    ))?;
                // Only a buyer who held the money, would have paid the going
                // price, and still found every shelf within reach empty is
                // evidence of scarcity. The rest went away for reasons of their
                // own, and a price that rises on those reasons rises on nothing.
                if refused_on_price {
                    // Walking away from a price is not the seller withholding
                    // the good, and charging a foreign supplier with grievance
                    // for a purchase the buyer declined would make a quarrel out
                    // of the buyer's own arithmetic.
                    refused_purchases.insert((order.buyer, order.good));
                }
                if cash_bound {
                    // An empty till is a failure of the buyer's own trade, not
                    // a good withheld abroad. A country cannot be resented for
                    // a purchase its neighbour could not afford.
                    cash_starved.insert((order.buyer, order.good));
                }
                if !exhausted_route_capacity && !refused_on_price && !cash_bound {
                    let effective = effective_unmet_by_market
                        .entry((order.region, order.good))
                        .or_default();
                    *effective =
                        effective
                            .checked_add(remaining)
                            .ok_or(WorldError::ArithmeticOverflow(
                                "procurement effective unmet demand",
                            ))?;
                }
            }
        }
        let journal_entries = self.prepare_money_journal_entries(&transactions)?;
        self.firms = firms;
        self.append_money_journal_entries(journal_entries);
        // A seller still holding stock was not the reason anybody left empty;
        // only a seller whose whole offer was taken may read the queue as its
        // own evidence. Every entry in the book started above zero.
        let mut offered_sellers: BTreeSet<(FirmId, GoodId)> = BTreeSet::new();
        let mut sellers_with_stock: BTreeSet<(FirmId, GoodId)> = BTreeSet::new();
        for offer in offers.iter() {
            offered_sellers.insert((offer.seller, offer.good));
            if offer.quantity.get() > 0 {
                sellers_with_stock.insert((offer.seller, offer.good));
            }
        }
        let exhausted_sellers = offered_sellers
            .difference(&sellers_with_stock)
            .copied()
            .collect();
        Ok(ProcurementSettlement {
            fills,
            unmet,
            capacity_limited,
            constrained_routes,
            trade_prices,
            purchases,
            unmet_by_market,
            exhausted_sellers,
            effective_unmet_by_market,
            refused_on_price: refused_purchases,
            starved_of_cash: cash_starved,
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
            // Remember the shortage against the buyer's region, whatever its
            // cause. A good nobody could deliver and a good nobody had are the
            // same standing invitation to produce it here, and until now that
            // invitation was thrown away the moment the event was journaled.
            let buyer_region = self
                .firms
                .get(&buyer)
                .ok_or(WorldError::UnknownFirm(buyer))?
                .region();
            let row = self
                .monthly_firm_input_shortfalls
                .entry((buyer_region, good))
                .or_default();
            *row = QuantityMilli::new(row.get().checked_add(quantity.get()).ok_or(
                WorldError::ArithmeticOverflow("regional firm input shortfall"),
            )?);
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
        for (&(region, good), &quantity) in &settlement.effective_unmet_by_market {
            let row = self
                .monthly_firm_effective_input_shortfalls
                .entry((region, good))
                .or_default();
            *row = QuantityMilli::new(row.get().checked_add(quantity).ok_or(
                WorldError::ArithmeticOverflow("regional effective firm input shortfall"),
            )?);
        }
        for ((firm, good), quantity) in recorded.sold {
            let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
            let unit_price = settlement
                .trade_prices
                .get(&(firm, good))
                .copied()
                .expect("settled procurement sale must have a recorded trade price");
            let region = definition.region();
            // Until now an intermediate seller was told nothing about the
            // buyers it could not serve, so a farm with a queue of mills at the
            // gate read its month as "sold everything, nobody left wanting" and
            // never had a reason to build another field. Households have always
            // handed their sellers this signal; firms may hand it too.
            let unmet_market_demand = if settlement.exhausted_sellers.contains(&(firm, good)) {
                settlement
                    .unmet_by_market
                    .get(&(region, good))
                    .copied()
                    .unwrap_or_default()
            } else {
                0
            };
            self.monthly_firm_market_outcomes
                .entry(firm)
                .or_default()
                .push(crate::MarketOfferOutcome {
                    seller: firm,
                    region,
                    good,
                    unit_price,
                    offered: QuantityMilli::new(quantity),
                    sold: QuantityMilli::new(quantity),
                    unsold: QuantityMilli::default(),
                    unmet_market_demand: QuantityMilli::new(unmet_market_demand),
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
        procurement: &FirmProcurementResult,
        household_unmet: &BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli>,
        household_offers: &[MarketOffer],
    ) -> Result<(), WorldError> {
        // The month's procurement carries both the shortage and the reasons it
        // happened. They are read together because a shortage without its
        // reason is exactly the mistake this function exists to avoid.
        let firm_unmet = &procurement.unmet;
        let firm_capacity_limited = &procurement.capacity_limited;
        let firm_refused_on_price = &procurement.refused_on_price;
        let firm_starved_of_cash = &procurement.starved_of_cash;
        let firm_offers = &procurement.pristine_offers;
        let mut accrued = BTreeSet::new();
        for &(buyer, good) in firm_unmet.keys() {
            // An exhausted shared route pool is a domestic/logistics scarcity,
            // not evidence that the foreign supplier withheld a deliverable good.
            if firm_capacity_limited.contains(&(buyer, good)) {
                continue;
            }
            // Nor is a price the buyer declined to pay. The good was there and
            // deliverable; the buyer judged it not worth what was asked. A
            // quarrel between countries cannot be built out of that judgement.
            if firm_refused_on_price.contains(&(buyer, good)) {
                continue;
            }
            // Nor is an empty till. The buyer's own trade failed to earn what
            // the purchase required; a neighbour who stood ready to sell has
            // withheld nothing.
            if firm_starved_of_cash.contains(&(buyer, good)) {
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
        // Last month's shortage evidence has been read by everyone entitled to
        // read it; this month writes its own.
        self.monthly_firm_input_shortfalls.clear();
        self.monthly_firm_effective_input_shortfalls.clear();
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
            refused_on_price: settlement.refused_on_price,
            starved_of_cash: settlement.starved_of_cash,
            constrained_routes: settlement.constrained_routes,
            pristine_offers,
        })
    }
}
