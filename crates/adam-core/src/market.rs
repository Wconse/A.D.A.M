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
pub struct FirmMarketOfferPlan {
    pub seller: FirmId,
    pub region: RegionId,
    pub good: GoodId,
    pub inventory: QuantityMilli,
    pub average_monthly_sales: QuantityMilli,
    pub retained_inventory: QuantityMilli,
    pub offered: QuantityMilli,
    pub unit_price: Money,
}
impl FirmMarketOfferPlan {
    #[must_use]
    pub const fn market_offer(self) -> Option<MarketOffer> {
        if self.offered.get() == 0 {
            None
        } else {
            Some(MarketOffer {
                seller: self.seller,
                region: self.region,
                good: self.good,
                quantity: self.offered,
                unit_price: self.unit_price,
            })
        }
    }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarketOfferOutcome {
    pub seller: FirmId,
    pub region: RegionId,
    pub good: GoodId,
    pub unit_price: Money,
    pub offered: QuantityMilli,
    pub sold: QuantityMilli,
    pub unsold: QuantityMilli,
    pub unmet_market_demand: QuantityMilli,
}
impl MarketOfferOutcome {
    #[must_use]
    pub const fn sold_out_while_demand_remained(self) -> bool {
        self.offered.get() > 0 && self.unsold.get() == 0 && self.unmet_market_demand.get() > 0
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarketClearing {
    pub fills: Vec<MarketFill>,
    pub unmet: BTreeUnmet,
    pub offer_outcomes: Vec<MarketOfferOutcome>,
}
pub type BTreeUnmet = std::collections::BTreeMap<(CohortId, GoodId, NeedTier), QuantityMilli>;
impl crate::World {
    /// Derives concrete seller offers from output inventory, observed sales, and firm policy.
    /// # Errors
    /// Returns an error for missing reference prices or fixed-point overflow.
    pub fn plan_firm_market_offers(&self) -> Result<Vec<FirmMarketOfferPlan>, WorldError> {
        self.firms
            .values()
            .filter_map(|firm| {
                self.firm_policies
                    .get(&firm.id())
                    .map(|policy| (firm, policy))
            })
            .map(|(firm, policy)| {
                let recipe = self
                    .production_recipes
                    .get(&firm.recipe())
                    .ok_or(WorldError::UnknownRecipe(firm.recipe()))?;
                let good = recipe.output_good();
                let inventory = firm.inventories().get(&good).copied().unwrap_or_default();
                let average_sales = average_observed_firm_sales(self, firm.id(), good)?;
                let retained = u128::from(average_sales.get())
                    .checked_mul(u128::from(policy.inventory_buffer_days()))
                    .ok_or(WorldError::ArithmeticOverflow("inventory buffer target"))?
                    .div_ceil(30);
                let retained = u64::try_from(retained)
                    .map_err(|_| WorldError::ArithmeticOverflow("inventory buffer target"))?
                    .min(inventory.get());
                let reference = self
                    .regional_prices
                    .get(&(firm.region(), good))
                    .copied()
                    .ok_or(WorldError::MissingRegionalPrice {
                        region: firm.region(),
                        good,
                    })?;
                let multiplier = 10_000_i128 + i128::from(policy.price_markup().get());
                let unit_price = i64::try_from(
                    i128::from(reference.minor_units())
                        .checked_mul(multiplier)
                        .ok_or(WorldError::ArithmeticOverflow("seller offer price"))?
                        / 10_000,
                )
                .map_err(|_| WorldError::ArithmeticOverflow("seller offer price"))?;
                Ok(FirmMarketOfferPlan {
                    seller: firm.id(),
                    region: firm.region(),
                    good,
                    inventory,
                    average_monthly_sales: average_sales,
                    retained_inventory: QuantityMilli::new(retained),
                    offered: QuantityMilli::new(inventory.get() - retained),
                    unit_price: Money::from_minor_units(unit_price),
                })
            })
            .collect()
    }
}

fn average_observed_firm_sales(
    world: &crate::World,
    firm: FirmId,
    good: GoodId,
) -> Result<QuantityMilli, WorldError> {
    let Some(history) = world.firm_operating_history.get(&firm) else {
        return Ok(QuantityMilli::default());
    };
    let mut periods = 0_u128;
    let mut total = 0_u128;
    for observation in history {
        let outcomes: Vec<_> = observation
            .market_outcomes()
            .iter()
            .filter(|outcome| outcome.good == good)
            .collect();
        if outcomes.is_empty() {
            continue;
        }
        periods += 1;
        for outcome in outcomes {
            total = total
                .checked_add(u128::from(outcome.sold.get()))
                .ok_or(WorldError::ArithmeticOverflow("observed seller sales"))?;
        }
    }
    if periods == 0 {
        return Ok(QuantityMilli::default());
    }
    Ok(QuantityMilli::new(u64::try_from(total / periods).map_err(
        |_| WorldError::ArithmeticOverflow("average seller sales"),
    )?))
}

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
    if supply
        .iter()
        .any(|offer| offer.unit_price.minor_units() <= 0)
    {
        return Err(WorldError::InvalidPrice);
    }
    let mut remaining: Vec<u64> = supply.iter().map(|o| o.quantity.get()).collect();
    let mut fills = Vec::new();
    let mut unmet = BTreeUnmet::new();
    let mut unmet_by_market: std::collections::BTreeMap<(RegionId, GoodId), u64> =
        std::collections::BTreeMap::new();
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
            let market = unmet_by_market
                .entry((order.region, order.good))
                .or_default();
            *market = market
                .checked_add(need)
                .ok_or(WorldError::ArithmeticOverflow("market unmet demand"))?;
        }
    }
    let offer_outcomes = supply
        .iter()
        .zip(remaining)
        .map(|(offer, unsold)| MarketOfferOutcome {
            seller: offer.seller,
            region: offer.region,
            good: offer.good,
            unit_price: offer.unit_price,
            offered: offer.quantity,
            sold: QuantityMilli::new(offer.quantity.get() - unsold),
            unsold: QuantityMilli::new(unsold),
            unmet_market_demand: QuantityMilli::new(
                unmet_by_market
                    .get(&(offer.region, offer.good))
                    .copied()
                    .unwrap_or_default(),
            ),
        })
        .collect();
    Ok(MarketClearing {
        fills,
        unmet,
        offer_outcomes,
    })
}
fn record_market_settlement_evidence(
    world: &mut crate::World,
    clearing: &MarketClearing,
) -> Result<(), WorldError> {
    for outcome in &clearing.offer_outcomes {
        world
            .monthly_firm_market_outcomes
            .entry(outcome.seller)
            .or_default()
            .push(*outcome);
    }
    let mut revenues: std::collections::BTreeMap<FirmId, Money> = std::collections::BTreeMap::new();
    for fill in &clearing.fills {
        let current = revenues.get(&fill.seller).copied().unwrap_or_default();
        revenues.insert(
            fill.seller,
            Money::from_minor_units(
                current
                    .minor_units()
                    .checked_add(fill.spend.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow("seller revenue aggregation"))?,
            ),
        );
    }
    for (firm, revenue) in revenues {
        world.record_firm_final_sale(firm, revenue)?;
    }
    for fill in &clearing.fills {
        world.events.append(
            world.date,
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

fn validate_market_clearing(
    world: &crate::World,
    clearing: &MarketClearing,
) -> Result<(), WorldError> {
    let mut outcome_sales: std::collections::BTreeMap<(FirmId, GoodId), u64> =
        std::collections::BTreeMap::new();
    for outcome in &clearing.offer_outcomes {
        let definition = world
            .firms
            .get(&outcome.seller)
            .ok_or(WorldError::UnknownFirm(outcome.seller))?;
        if definition.region() != outcome.region
            || outcome.offered.get()
                != outcome
                    .sold
                    .get()
                    .checked_add(outcome.unsold.get())
                    .ok_or(WorldError::ArithmeticOverflow("market offer outcome"))?
        {
            return Err(WorldError::InvalidMarketClearing(
                "offer outcomes must conserve quantity in the seller region",
            ));
        }
        if outcome.sold.get() > 0 {
            let sold = outcome_sales
                .entry((outcome.seller, outcome.good))
                .or_default();
            *sold = sold
                .checked_add(outcome.sold.get())
                .ok_or(WorldError::ArithmeticOverflow("market outcome sales"))?;
        }
    }
    let mut fill_sales: std::collections::BTreeMap<(FirmId, GoodId), u64> =
        std::collections::BTreeMap::new();
    for fill in &clearing.fills {
        let sold = fill_sales.entry((fill.seller, fill.good)).or_default();
        *sold = sold
            .checked_add(fill.quantity.get())
            .ok_or(WorldError::ArithmeticOverflow("market fill sales"))?;
    }
    if outcome_sales != fill_sales {
        return Err(WorldError::InvalidMarketClearing(
            "offer outcomes must match settled fills",
        ));
    }
    Ok(())
}

impl crate::World {
    /// Atomically settles pre-cleared market fills against household wealth and firm inventories.
    /// # Errors
    /// Returns an error without mutation for insufficient cash, stock, or arithmetic overflow.
    pub fn settle_local_market(&mut self, clearing: &MarketClearing) -> Result<(), WorldError> {
        validate_market_clearing(self, clearing)?;
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
        record_market_settlement_evidence(self, clearing)?;
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
    use crate::{
        AgeBand, BasisPoints, ConsumptionProfile, ConsumptionTarget, Country, CountryId,
        DemandBasis, EducationLevel, EmploymentStatus, Firm, FirmPolicy, Good, HouseholdCohort,
        HouseholdType, NeedProfileId, Population, ProductionRecipe, RecipeId, Region, SimDate,
        World, WorldSeed,
    };
    use std::collections::BTreeMap;

    fn settlement_world() -> World {
        let mut world = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Households",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(1),
                    Money::from_minor_units(1),
                )
                .expect("region"),
            )
            .expect("region");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![],
                )
                .expect("recipe"),
            )
            .expect("recipe");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Farm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::default(),
                    BTreeMap::from([(GoodId::new(1), QuantityMilli::new(1_000))]),
                )
                .expect("firm"),
            )
            .expect("firm");
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
                    Money::default(),
                    Money::from_minor_units(100),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort");
        world
    }

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
        assert_eq!(result.offer_outcomes.len(), 1);
        assert_eq!(result.offer_outcomes[0].sold.get(), 1_000);
        assert_eq!(result.offer_outcomes[0].unsold.get(), 0);
        assert_eq!(result.offer_outcomes[0].unmet_market_demand.get(), 400);
        assert!(result.offer_outcomes[0].sold_out_while_demand_remained());
    }
    #[test]
    fn inventory_policy_and_markup_form_a_concrete_offer() {
        let mut world = settlement_world();
        world
            .set_regional_price(
                RegionId::new(1),
                GoodId::new(1),
                Money::from_minor_units(10),
            )
            .expect("price");
        world.firm_policies.insert(
            FirmId::new(1),
            FirmPolicy::new(
                30,
                BasisPoints::new(1_000).expect("markup"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
                BasisPoints::new(0).expect("allocation"),
            )
            .expect("policy"),
        );
        world.monthly_firm_market_outcomes.insert(
            FirmId::new(1),
            vec![MarketOfferOutcome {
                seller: FirmId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                unit_price: Money::from_minor_units(10),
                offered: QuantityMilli::new(400),
                sold: QuantityMilli::new(400),
                unsold: QuantityMilli::default(),
                unmet_market_demand: QuantityMilli::default(),
            }],
        );
        world
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("capture");

        let plans = world.plan_firm_market_offers().expect("offer plans");
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].average_monthly_sales, QuantityMilli::new(400));
        assert_eq!(plans[0].retained_inventory, QuantityMilli::new(400));
        assert_eq!(plans[0].offered, QuantityMilli::new(600));
        assert_eq!(plans[0].unit_price, Money::from_minor_units(11));
        assert_eq!(plans[0].market_offer().expect("offer").quantity.get(), 600);
    }

    #[test]
    fn settlement_carries_offer_outcome_into_firm_history() {
        let mut world = settlement_world();
        let clearing = clear_local_market(
            &[MarketOrder {
                buyer: CohortId::new(1),
                tier: NeedTier::Survival,
                region: RegionId::new(1),
                good: GoodId::new(1),
                quantity: QuantityMilli::new(1_400),
                max_spend: Money::from_minor_units(100),
            }],
            &[MarketOffer {
                seller: FirmId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                quantity: QuantityMilli::new(1_000),
                unit_price: Money::from_minor_units(10),
            }],
        )
        .expect("clearing");
        world.settle_local_market(&clearing).expect("settlement");
        assert!(
            world.monthly_firm_market_outcomes()[&FirmId::new(1)][0]
                .sold_out_while_demand_remained()
        );
        world
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("capture");
        assert!(
            world.firm_operating_history()[&FirmId::new(1)][0].market_outcomes()[0]
                .sold_out_while_demand_remained()
        );
    }
}
