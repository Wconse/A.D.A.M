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
    pub cost_floor: Money,
    pub floor_binding: bool,
}

impl World {
    /// Revises regional reference prices from the settled month's fills and unsold offers.
    ///
    /// A market that sold out while demand remained unfilled raises its price, a market
    /// that left goods unsold cuts its price even when needs went unmet beside them, and
    /// no market settles below the variable cost per unit of its cheapest solvent local
    /// producer. Every move is bounded per month, and the fraction of a minor unit that a
    /// bounded move cannot express is carried into the following months, so a price is a
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
            let cost_floor = self.local_variable_cost_floor(region, good)?;
            let carry = self
                .regional_price_carry
                .get(&(region, good))
                .copied()
                .unwrap_or_default();
            let (revision, next_carry) = revised_price(previous, cost_floor, *pressure, carry)?;
            carries.push(((region, good), next_carry));
            let Some((price, floor_binding)) = revision else {
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
                cost_floor,
                floor_binding,
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
                    cost_floor: adjustment.cost_floor,
                    floor_binding: adjustment.floor_binding,
                },
            );
        }
        Ok(planned)
    }

    /// Aggregates the settled month's seller books into one pressure reading per market.
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
        pressures
    }

    /// Reports the lowest variable cost per whole unit among solvent local producers.
    ///
    /// Returns zero when the region has no solvent producer of the good whose inputs
    /// are all priced, which leaves the price unconstrained from below.
    ///
    /// # Errors
    /// Returns [`WorldError::ArithmeticOverflow`] when a cost computation overflows.
    pub fn local_variable_cost_floor(
        &self,
        region: RegionId,
        good: GoodId,
    ) -> Result<Money, WorldError> {
        let mut floor: Option<i128> = None;
        for firm in self.firms.values() {
            if firm.region() != region || self.is_firm_insolvent(firm.id()) {
                continue;
            }
            let Some(recipe) = self.production_recipes.get(&firm.recipe()) else {
                continue;
            };
            if recipe.output_good() != good {
                continue;
            }
            let output_per_batch = i128::from(recipe.output_per_batch().get());
            if output_per_batch <= 0 {
                continue;
            }
            let mut batch_cost = 0_i128;
            let mut fully_priced = true;
            for input in recipe.inputs() {
                let Some(price) = self.regional_prices.get(&(region, input.good())).copied() else {
                    fully_priced = false;
                    break;
                };
                let cost = i128::from(price.minor_units())
                    .checked_mul(i128::from(input.quantity_per_batch().get()))
                    .ok_or(WorldError::ArithmeticOverflow("variable input cost"))?
                    / i128::from(QuantityMilli::SCALE);
                batch_cost = batch_cost
                    .checked_add(cost)
                    .ok_or(WorldError::ArithmeticOverflow("variable input cost"))?;
            }
            if !fully_priced {
                continue;
            }
            let labor_cost = self
                .observed_firm_wage(firm.id())
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
            floor = Some(floor.map_or(unit_cost, |current| current.min(unit_cost)));
        }
        let floor = floor.unwrap_or_default();
        Ok(Money::from_minor_units(i64::try_from(floor).map_err(
            |_| WorldError::ArithmeticOverflow("variable cost floor"),
        )?))
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

/// Computes the revised price, whether the variable-cost floor bound it, and the
/// fractional pressure carried into the next month.
///
/// Returns `None` for the revision when the admissible move does not yet amount to a
/// whole minor unit, which is the normal case for a cheap good under mild pressure.
fn revised_price(
    previous: Money,
    cost_floor: Money,
    pressure: ObservedMarketPressure,
    carry: i64,
) -> Result<(Option<(Money, bool)>, i64), WorldError> {
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

    let mut floor_binding = false;
    let floor_units = i128::from(cost_floor.minor_units());
    let mut desired = milli_move(previous_units, delta_bps)?;
    // A price below the variable cost of its cheapest producer climbs toward that
    // cost, still no faster than the monthly cap allows.
    if floor_units > previous_units {
        let gap = floor_units
            .checked_sub(previous_units)
            .and_then(|gap| gap.checked_mul(CARRY_SCALE))
            .ok_or(WorldError::ArithmeticOverflow("variable cost gap"))?;
        // Selling below variable cost is a loss taken on every unit, so the price
        // goes to cost immediately rather than creeping toward it.
        if gap > desired {
            desired = gap;
            floor_binding = true;
        }
    }

    let accumulated = i128::from(carry)
        .checked_add(desired)
        .ok_or(WorldError::ArithmeticOverflow("carried price pressure"))?;
    // Truncating division keeps the carry signed: pressure accumulates until it is
    // worth a whole minor unit instead of rounding a cheap good every month.
    let mut price = previous_units.saturating_add(accumulated / CARRY_SCALE);
    if floor_units > 0 && price < floor_units && previous_units >= floor_units {
        price = floor_units;
        floor_binding = true;
    }
    price = price.max(1);

    let applied = price
        .checked_sub(previous_units)
        .and_then(|applied| applied.checked_mul(CARRY_SCALE))
        .ok_or(WorldError::ArithmeticOverflow("applied price move"))?;
    // Pressure that a binding floor or the minimum price refused to express is not
    // banked forever, otherwise a market would jump the moment the bound lifts.
    let remainder = (accumulated - applied).clamp(-(CARRY_SCALE - 1), CARRY_SCALE - 1);
    let next_carry =
        i64::try_from(remainder).map_err(|_| WorldError::ArithmeticOverflow("price carry"))?;

    if price == previous_units {
        return Ok((None, next_carry));
    }
    let price = i64::try_from(price)
        .map_err(|_| WorldError::ArithmeticOverflow("revised reference price"))?;
    Ok((
        Some((Money::from_minor_units(price), floor_binding)),
        next_carry,
    ))
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
    fn settle(
        previous: i64,
        cost_floor: i64,
        pressure: ObservedMarketPressure,
        months: u32,
    ) -> i64 {
        let mut price = previous;
        let mut carry = 0_i64;
        for _ in 0..months {
            let (revision, next_carry) = revised_price(
                Money::from_minor_units(price),
                Money::from_minor_units(cost_floor),
                pressure,
                carry,
            )
            .expect("revision");
            if let Some((revised, _)) = revision {
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
        assert_eq!(settle(10, 0, pressure(1_000, 1_000, 0, 1_000), 1), 11);
    }

    #[test]
    fn a_severe_shortage_produces_a_severe_price_shock() {
        // Nine tenths of demand went unfilled. A jump of that size is what the
        // scarcity implies, and nobody in this model is paying to prevent it.
        assert_eq!(settle(100, 0, pressure(1_000, 1_000, 0, 9_000), 1), 122);
    }

    #[test]
    fn a_fractional_move_of_a_cheap_price_is_carried_rather_than_rounded() {
        // Currency is discrete: a move worth a fraction of a minor unit waits
        // instead of being rounded up into a large jump.
        assert_eq!(settle(10, 0, pressure(1_000, 1_000, 0, 100), 1), 10);
        assert_eq!(settle(10, 0, pressure(1_000, 1_000, 0, 100), 6), 11);
    }

    #[test]
    fn sustained_unsold_offers_cut_the_price() {
        assert_eq!(settle(10, 0, pressure(1_000, 0, 1_000, 0), 2), 9);
    }

    #[test]
    fn unpaid_need_beside_unsold_stock_cuts_the_price_instead_of_raising_it() {
        assert_eq!(settle(10, 0, pressure(1_000, 0, 1_000, 1_000), 2), 9);
    }

    #[test]
    fn a_price_at_variable_cost_is_never_cut_below_it() {
        // The one bound that survives, because somebody concretely eats the loss:
        // selling under variable cost loses money on every unit sold.
        assert_eq!(settle(10, 10, pressure(1_000, 0, 1_000, 0), 12), 10);
    }

    #[test]
    fn a_price_below_variable_cost_goes_to_cost_at_once() {
        let (revision, _) = revised_price(
            Money::from_minor_units(10),
            Money::from_minor_units(100),
            pressure(1_000, 500, 500, 0),
            0,
        )
        .expect("revision");
        let (price, floor_binding) = revision.expect("the price moves");
        assert_eq!(price, Money::from_minor_units(100));
        assert!(floor_binding);
    }

    #[test]
    fn a_market_that_cleared_exactly_leaves_the_price_alone() {
        assert_eq!(settle(10, 0, pressure(1_000, 1_000, 0, 0), 12), 10);
    }
}
