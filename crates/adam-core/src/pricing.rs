use crate::{DomainEvent, FirmId, GoodId, Money, QuantityMilli, RegionId, World, WorldError};
use std::collections::BTreeMap;

/// Price response when every unit demanded in the region went unfilled.
///
/// This is an elasticity, not a safety bound: a market that emptied completely
/// reprices by this much, and nothing forbids that from being a large move.
const SHORTAGE_SENSITIVITY_BPS: i128 = 2_500;
/// Price response when every unit offered in the region went unsold.
const SURPLUS_SENSITIVITY_BPS: i128 = 700;
/// Recipes state labor in milli worker-months against a whole monthly wage.
const LABOR_MILLI_SCALE: i128 = 1_000;
/// Fractional price pressure is carried between months in thousandths of a minor unit.
const CARRY_SCALE: i128 = 1_000;

/// Aggregated market outcome for one region and good over the settled month.
///
/// `offered`, `sold`, and `unsold` are summed across sellers because each seller
/// reports its own book. `unmet` is a market-wide figure that every seller in the
/// region repeats, so it is taken as a maximum rather than a sum.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObservedMarketPressure {
    pub offered: u64,
    pub sold: u64,
    pub unsold: u64,
    pub unmet: u64,
}

/// Auditable record of one regional reference price revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionalPriceAdjustment {
    pub region: RegionId,
    pub good: GoodId,
    pub previous: Money,
    pub price: Money,
    pub offered: QuantityMilli,
    pub sold: QuantityMilli,
    pub unsold: QuantityMilli,
    pub unmet_demand: QuantityMilli,
}

impl World {
    /// Revises regional reference prices from the settled month's fills and unsold offers.
    ///
    /// A market that sold out while demand remained unfilled raises its price, and a
    /// market that left goods unsold cuts its price even when needs went unmet beside
    /// them. Nothing here consults anybody's costs: a producer that cannot cover its
    /// variable cost stops offering the good rather than pricing under it, so its costs
    /// reach this reading as supply that never arrived. The fraction of a minor unit
    /// that a move cannot express is carried into the following months, so a price is a
    /// slow-moving observable rather than an instantaneous market-clearing solution.
    ///
    /// # Errors
    /// Returns [`WorldError::ArithmeticOverflow`] when a price or cost computation overflows.
    pub fn execute_monthly_price_formation(
        &mut self,
    ) -> Result<Vec<RegionalPriceAdjustment>, WorldError> {
        let pressures = self.observed_market_pressure();
        let mut planned = Vec::new();
        let mut carries = Vec::new();
        for (&(region, good), pressure) in &pressures {
            let Some(previous) = self.regional_prices.get(&(region, good)).copied() else {
                continue;
            };
            if previous.minor_units() <= 0 {
                continue;
            }
            let carry = self
                .regional_price_carry
                .get(&(region, good))
                .copied()
                .unwrap_or_default();
            let (revision, next_carry) = revised_price(previous, *pressure, carry)?;
            carries.push(((region, good), next_carry));
            let Some(price) = revision else {
                continue;
            };
            planned.push(RegionalPriceAdjustment {
                region,
                good,
                previous,
                price,
                offered: QuantityMilli::new(pressure.offered),
                sold: QuantityMilli::new(pressure.sold),
                unsold: QuantityMilli::new(pressure.unsold),
                unmet_demand: QuantityMilli::new(pressure.unmet),
            });
        }
        for (key, carry) in carries {
            if carry == 0 {
                self.regional_price_carry.remove(&key);
            } else {
                self.regional_price_carry.insert(key, carry);
            }
        }
        let date = self.date;
        for adjustment in &planned {
            self.regional_prices
                .insert((adjustment.region, adjustment.good), adjustment.price);
            self.events.append(
                date,
                DomainEvent::RegionalPriceAdjusted {
                    region: adjustment.region,
                    good: adjustment.good,
                    previous: adjustment.previous,
                    price: adjustment.price,
                    offered: adjustment.offered,
                    sold: adjustment.sold,
                    unsold: adjustment.unsold,
                    unmet_demand: adjustment.unmet_demand,
                },
            );
        }
        Ok(planned)
    }

    /// Aggregates the settled month's trade into one pressure reading per market.
    ///
    /// Both kinds of buyer are counted. Household sellers report their own
    /// books, and the wholesale round that ran earlier in the same month
    /// contributes what firms bought and what they queued for and did not get.
    /// An intermediate good is never anybody's survival need, so while only the
    /// shop counter was observed a permanently starved input channel could go
    /// unfilled every month without its price ever moving - and a price that
    /// cannot rise is a wage its producers cannot justify paying, which is why
    /// no farm was ever worth founding beside a bakery that outbid it.
    #[must_use]
    pub fn observed_market_pressure(&self) -> BTreeMap<(RegionId, GoodId), ObservedMarketPressure> {
        let mut pressures: BTreeMap<(RegionId, GoodId), ObservedMarketPressure> = BTreeMap::new();
        for outcome in self.monthly_firm_market_outcomes.values().flatten() {
            let entry = pressures.entry((outcome.region, outcome.good)).or_default();
            entry.offered = entry.offered.saturating_add(outcome.offered.get());
            entry.sold = entry.sold.saturating_add(outcome.sold.get());
            entry.unsold = entry.unsold.saturating_add(outcome.unsold.get());
            entry.unmet = entry.unmet.max(outcome.unmet_market_demand.get());
        }
        // Firms buy too, and their custom was invisible to price until now. A
        // mill that bought nine tenths of the grain it needed is part of what
        // this market traded, and belongs in the denominator beside household
        // sales; otherwise a small wholesale queue beside a large one would
        // read as the same scarcity.
        for (&(buyer, good), &(quantity, _)) in &self.monthly_firm_procurement_purchases {
            let Some(firm) = self.firms.get(&buyer) else {
                continue;
            };
            let entry = pressures.entry((firm.region(), good)).or_default();
            entry.sold = entry.sold.saturating_add(quantity.get());
        }
        // An input nobody could deliver is a queue at this market's counter,
        // exactly like a household leaving empty-handed. Wholesale shortfalls
        // are summed rather than maximised because each buyer queued for its
        // own tonnage, whereas a household market-wide figure is repeated by
        // every seller reporting it.
        for (&(region, good), &quantity) in &self.monthly_firm_effective_input_shortfalls {
            let entry = pressures.entry((region, good)).or_default();
            entry.unmet = entry.unmet.saturating_add(quantity.get());
        }
        pressures
    }

    /// Reports what one whole unit of its own output costs this firm to make,
    /// counting the inputs it must replace at local prices and the wage it has
    /// actually agreed to pay.
    ///
    /// Returns `None` when an input has no local price, because a cost nobody
    /// can price is a cost nobody can defend a refusal to sell with.
    ///
    /// # Errors
    /// Returns [`WorldError::ArithmeticOverflow`] when a cost computation overflows.
    pub fn firm_variable_unit_cost(&self, firm: FirmId) -> Result<Option<Money>, WorldError> {
        let Some(definition) = self.firms.get(&firm) else {
            return Ok(None);
        };
        let Some(recipe) = self.production_recipes.get(&definition.recipe()) else {
            return Ok(None);
        };
        let output_per_batch = i128::from(recipe.output_per_batch().get());
        if output_per_batch <= 0 {
            return Ok(None);
        }
        let mut batch_cost = 0_i128;
        for input in recipe.inputs() {
            let Some(price) = self
                .regional_prices
                .get(&(definition.region(), input.good()))
                .copied()
            else {
                return Ok(None);
            };
            let cost = i128::from(price.minor_units())
                .checked_mul(i128::from(input.quantity_per_batch().get()))
                .ok_or(WorldError::ArithmeticOverflow("variable input cost"))?
                / i128::from(QuantityMilli::SCALE);
            batch_cost = batch_cost
                .checked_add(cost)
                .ok_or(WorldError::ArithmeticOverflow("variable input cost"))?;
        }
        let labor_cost = self
            .observed_firm_wage(firm)
            .checked_mul(i128::from(recipe.labor_milli_worker_months()))
            .ok_or(WorldError::ArithmeticOverflow("variable labor cost"))?
            / LABOR_MILLI_SCALE;
        batch_cost = batch_cost
            .checked_add(labor_cost)
            .ok_or(WorldError::ArithmeticOverflow("variable labor cost"))?;
        let unit_cost = batch_cost
            .checked_mul(i128::from(QuantityMilli::SCALE))
            .ok_or(WorldError::ArithmeticOverflow("variable cost floor"))?
            / output_per_batch;
        Ok(Some(Money::from_minor_units(
            i64::try_from(unit_cost)
                .map_err(|_| WorldError::ArithmeticOverflow("variable cost floor"))?,
        )))
    }

    /// Reports the lowest price at which a unit still leaves this firm whole once
    /// it has handed the state the sales tax that unit attracts.
    ///
    /// A wage is not the whole of what a sale must earn back. The tax is levied on
    /// final household sales only, so a firm trading into wholesale owes nothing on
    /// that turnover and has nothing to add to its price, while a shop that ignores
    /// the levy pays it out of capital and empties its till at a steady rate no
    /// amount of selling can arrest. The taxed share is read from what this firm
    /// actually sold. A firm that has yet to sell anything adds nothing, because a
    /// seller guessing the other way would charge a wholesale buyer a tax nobody
    /// will ever collect and pocket the difference as invented revenue. The price
    /// of learning is real and the seller pays it: the first month of retail trade
    /// is taxed out of the till before the ask rises to cover the levy.
    ///
    /// # Errors
    /// Returns [`WorldError::ArithmeticOverflow`] when a cost computation overflows.
    pub fn firm_break_even_unit_price(&self, firm: FirmId) -> Result<Option<Money>, WorldError> {
        let Some(cost) = self.firm_variable_unit_cost(firm)? else {
            return Ok(None);
        };
        let levy_bps = crate::simulation::SALES_TAX_BPS
            .checked_mul(self.observed_taxed_sales_share_bps(firm))
            .ok_or(WorldError::ArithmeticOverflow("observed sales tax levy"))?
            / 10_000;
        let retained_bps = 10_000 - levy_bps;
        if retained_bps <= 0 {
            return Ok(Some(cost));
        }
        let gross = i128::from(cost.minor_units())
            .checked_mul(10_000)
            .ok_or(WorldError::ArithmeticOverflow("break-even unit price"))?;
        // Rounded up: a price that recovers all but one minor unit of the levy is
        // still a price that shrinks the till every month it is charged.
        let price = (gross + retained_bps - 1) / retained_bps;
        Ok(Some(Money::from_minor_units(
            i64::try_from(price)
                .map_err(|_| WorldError::ArithmeticOverflow("break-even unit price"))?,
        )))
    }

    /// Reports the share of this firm's observed turnover that carried sales tax.
    fn observed_taxed_sales_share_bps(&self, firm: FirmId) -> i128 {
        let Some(history) = self.firm_operating_history.get(&firm) else {
            return 0;
        };
        let mut turnover = 0_i128;
        let mut taxed = 0_i128;
        for observation in history {
            turnover =
                turnover.saturating_add(i128::from(observation.sales_revenue().minor_units()));
            taxed =
                taxed.saturating_add(i128::from(observation.final_sales_revenue().minor_units()));
        }
        if turnover <= 0 {
            return 0;
        }
        (taxed.saturating_mul(10_000) / turnover).clamp(0, 10_000)
    }

    /// Reports the worker-weighted monthly wage the firm has actually agreed to pay.
    fn observed_firm_wage(&self, firm: FirmId) -> i128 {
        let mut workers = 0_u64;
        let mut total = 0_i128;
        for ((employer, _), agreement) in &self.employment_agreements {
            if *employer != firm || !agreement.active() {
                continue;
            }
            workers = workers.saturating_add(agreement.workers());
            total = total.saturating_add(
                i128::from(agreement.wage().minor_units())
                    .saturating_mul(i128::from(agreement.workers())),
            );
        }
        if workers == 0 {
            0
        } else {
            total / i128::from(workers)
        }
    }
}

/// Computes the revised price and the fractional pressure carried into the next month.
///
/// Returns `None` for the revision when the admissible move does not yet amount to a
/// whole minor unit, which is the normal case for a cheap good under mild pressure.
fn revised_price(
    previous: Money,
    pressure: ObservedMarketPressure,
    carry: i64,
) -> Result<(Option<Money>, i64), WorldError> {
    let previous_units = i128::from(previous.minor_units());
    let traded = pressure.sold.saturating_add(pressure.unmet);
    // Unfilled demand raises a price only when the market actually sold out.
    // Demand left unfilled while goods remained on the shelf is unpaid need
    // rather than effective demand: households without money cannot bid a price
    // up, and the unsold stock beside them argues for a cut instead.
    let shortage_share = if pressure.unmet > 0 && traded > 0 && pressure.unsold == 0 {
        share_bps(pressure.unmet, traded)
    } else {
        0
    };
    let glut_share = if pressure.unsold > 0 && pressure.offered > 0 {
        share_bps(pressure.unsold, pressure.offered)
    } else {
        0
    };
    // No monthly bound is imposed on the move. A ceiling would mean somebody chose
    // to hold a price down, and in this model nobody is paying for that choice, so
    // the market reprices by as much as the observed scarcity or glut implies.
    let delta_bps = if shortage_share > 0 {
        SHORTAGE_SENSITIVITY_BPS * shortage_share / 10_000
    } else if glut_share > 0 {
        -(SURPLUS_SENSITIVITY_BPS * glut_share / 10_000)
    } else {
        0
    };

    // Costs do not enter this reading at all. A seller that cannot cover its variable
    // cost withdraws its offer instead of pricing under it, and it pays for that
    // refusal with unsold stock and a payroll drawn from the same till. What arrives
    // here is the consequence: goods that were never offered. A floor written into the
    // observation itself would be a limit nobody holds and nobody funds.
    let desired = milli_move(previous_units, delta_bps)?;

    let accumulated = i128::from(carry)
        .checked_add(desired)
        .ok_or(WorldError::ArithmeticOverflow("carried price pressure"))?;
    // Truncating division keeps the carry signed: pressure accumulates until it is
    // worth a whole minor unit instead of rounding a cheap good every month.
    let price = previous_units
        .saturating_add(accumulated / CARRY_SCALE)
        .max(1);

    let applied = price
        .checked_sub(previous_units)
        .and_then(|applied| applied.checked_mul(CARRY_SCALE))
        .ok_or(WorldError::ArithmeticOverflow("applied price move"))?;
    // Pressure that the minimum price of one minor unit refused to express is not
    // banked forever, otherwise a market would jump the moment the bound lifts.
    let remainder = (accumulated - applied).clamp(-(CARRY_SCALE - 1), CARRY_SCALE - 1);
    let next_carry =
        i64::try_from(remainder).map_err(|_| WorldError::ArithmeticOverflow("price carry"))?;

    if price == previous_units {
        return Ok((None, next_carry));
    }
    let price = i64::try_from(price)
        .map_err(|_| WorldError::ArithmeticOverflow("revised reference price"))?;
    Ok((Some(Money::from_minor_units(price)), next_carry))
}

/// Expresses a part of a whole as basis points, saturating at the whole.
fn share_bps(part: u64, whole: u64) -> i128 {
    if whole == 0 {
        return 0;
    }
    (i128::from(part) * 10_000 / i128::from(whole)).clamp(0, 10_000)
}

/// Expresses a basis-point move of a price in thousandths of a minor unit.
fn milli_move(value: i128, delta_bps: i128) -> Result<i128, WorldError> {
    Ok(value
        .checked_mul(delta_bps)
        .and_then(|scaled| scaled.checked_mul(CARRY_SCALE))
        .ok_or(WorldError::ArithmeticOverflow("revised reference price"))?
        / 10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn pressure(offered: u64, sold: u64, unsold: u64, unmet: u64) -> ObservedMarketPressure {
        ObservedMarketPressure {
            offered,
            sold,
            unsold,
            unmet,
        }
    }

    /// Settles the same market pressure for several months and reports the price.
    fn settle(previous: i64, pressure: ObservedMarketPressure, months: u32) -> i64 {
        let mut price = previous;
        let mut carry = 0_i64;
        for _ in 0..months {
            let (revision, next_carry) =
                revised_price(Money::from_minor_units(price), pressure, carry).expect("revision");
            if let Some(revised) = revision {
                price = revised.minor_units();
            }
            carry = next_carry;
        }
        price
    }

    #[test]
    fn a_sold_out_market_reprices_at_once_by_the_observed_scarcity() {
        // Half of the demand walked away empty, so the price moves by half the
        // scarcity elasticity in the very first month. Nothing holds it back.
        assert_eq!(settle(10, pressure(1_000, 1_000, 0, 1_000), 1), 11);
    }

    #[test]
    fn a_severe_shortage_produces_a_severe_price_shock() {
        // Nine tenths of demand went unfilled. A jump of that size is what the
        // scarcity implies, and nobody in this model is paying to prevent it.
        assert_eq!(settle(100, pressure(1_000, 1_000, 0, 9_000), 1), 122);
    }

    #[test]
    fn a_fractional_move_of_a_cheap_price_is_carried_rather_than_rounded() {
        // Currency is discrete: a move worth a fraction of a minor unit waits
        // instead of being rounded up into a large jump.
        assert_eq!(settle(10, pressure(1_000, 1_000, 0, 100), 1), 10);
        assert_eq!(settle(10, pressure(1_000, 1_000, 0, 100), 6), 11);
    }

    #[test]
    fn sustained_unsold_offers_cut_the_price() {
        assert_eq!(settle(10, pressure(1_000, 0, 1_000, 0), 2), 9);
    }

    #[test]
    fn unpaid_need_beside_unsold_stock_cuts_the_price_instead_of_raising_it() {
        assert_eq!(settle(10, pressure(1_000, 0, 1_000, 1_000), 2), 9);
    }

    /// A reference price is a reading of the counter, not a promise to anybody. Goods
    /// left unsold keep cutting it, however far the cut goes past what the good costs
    /// to make: the seller answers a price it cannot live with by withdrawing the
    /// offer, and it pays for that refusal with its own unsold stock and payroll.
    /// Holding the reading up instead would be a floor nobody funds.
    #[test]
    fn unsold_stock_keeps_cutting_a_price_with_no_regard_for_costs() {
        let after_two = settle(10, pressure(1_000, 0, 1_000, 0), 2);
        let after_a_year = settle(10, pressure(1_000, 0, 1_000, 0), 12);
        assert!(after_two < 10, "a glut cuts at once: {after_two}");
        assert!(
            after_a_year < after_two,
            "a year of unsold stock keeps cutting: {after_a_year} vs {after_two}"
        );
    }

    #[test]
    fn a_market_that_cleared_exactly_leaves_the_price_alone() {
        assert_eq!(settle(10, pressure(1_000, 1_000, 0, 0), 12), 10);
    }

    /// A region with a priced good, one farm that could produce it, and no
    /// household trade at all. Every price move here comes from wholesale
    /// evidence, so nothing else can be credited for it.
    fn grain_market_world() -> World {
        let mut world = World::new(
            crate::WorldSeed::new(4_242),
            crate::SimDate::new(2025, 1).expect("date"),
        );
        world
            .register_country(crate::Country::new(crate::CountryId::new(1), "A").expect("country"))
            .expect("register country");
        world
            .register_good(crate::Good::new(GoodId::new(1), "Grain").expect("good"))
            .expect("register grain");
        world
            .register_region(
                crate::Region::new(
                    RegionId::new(1),
                    crate::CountryId::new(1),
                    "R",
                    crate::Population::new(2),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("register region");
        world
            .register_production_recipe(
                crate::ProductionRecipe::new(
                    crate::RecipeId::new(1),
                    "Grain recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("register recipe");
        world
            .register_firm(
                crate::Firm::new(
                    FirmId::new(1),
                    "Farm",
                    RegionId::new(1),
                    crate::RecipeId::new(1),
                    1,
                    1,
                    Money::default(),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("register firm");
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(100),
            )
            .expect("grain price");
        world
    }

    #[test]
    fn a_queue_of_firms_at_the_counter_lifts_the_price_they_queued_for() {
        // Nobody delivered a single unit of the grain the mills asked for. That
        // is a market that sold out with demand still standing there, and an
        // intermediate good has no household need to speak for it, so if this
        // queue does not move the price then nothing ever will.
        let mut world = grain_market_world();
        world.monthly_firm_effective_input_shortfalls.insert(
            (RegionId::new(1), GoodId::new(1)),
            QuantityMilli::new(1_000),
        );

        world
            .execute_monthly_price_formation()
            .expect("price formation");

        assert_eq!(
            world.regional_prices[&(RegionId::new(1), GoodId::new(1))],
            Money::from_minor_units(125),
            "an input nobody could deliver must become dearer, or its producers can never justify the wage that would grow it"
        );
    }

    #[test]
    fn a_channel_that_almost_filled_moves_its_price_far_less() {
        // The same thousand missing units, this time beside nine thousand that
        // arrived. A mill that got nine tenths of its grain is short, but it is
        // not the same kind of short as one that got none.
        let mut world = grain_market_world();
        world.monthly_firm_procurement_purchases.insert(
            (FirmId::new(1), GoodId::new(1)),
            (QuantityMilli::new(9_000), Money::from_minor_units(900)),
        );
        world.monthly_firm_effective_input_shortfalls.insert(
            (RegionId::new(1), GoodId::new(1)),
            QuantityMilli::new(1_000),
        );

        world
            .execute_monthly_price_formation()
            .expect("price formation");

        assert_eq!(
            world.regional_prices[&(RegionId::new(1), GoodId::new(1))],
            Money::from_minor_units(102),
            "what the buyers did manage to purchase belongs in the denominator, otherwise every trickle of a shortfall reads as an emptied market"
        );
    }
}
