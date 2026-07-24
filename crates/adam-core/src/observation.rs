use std::collections::BTreeMap;

use crate::{FirmId, GoodId, MarketOfferOutcome, Money, SimDate, World, WorldError};

pub const FIRM_OBSERVATION_HISTORY_LIMIT: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmOperatingObservation {
    date: SimDate,
    sales_revenue: Money,
    final_sales_revenue: Money,
    produced_batches: u64,
    input_prices: BTreeMap<GoodId, Money>,
    market_outcomes: Vec<MarketOfferOutcome>,
}
impl FirmOperatingObservation {
    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }
    #[must_use]
    pub const fn sales_revenue(&self) -> Money {
        self.sales_revenue
    }
    #[must_use]
    pub const fn final_sales_revenue(&self) -> Money {
        self.final_sales_revenue
    }
    #[must_use]
    pub const fn produced_batches(&self) -> u64 {
        self.produced_batches
    }
    #[must_use]
    pub const fn input_prices(&self) -> &BTreeMap<GoodId, Money> {
        &self.input_prices
    }
    #[must_use]
    pub fn market_outcomes(&self) -> &[MarketOfferOutcome] {
        &self.market_outcomes
    }
}

pub(crate) struct ObservedOperatingBaseline {
    pub(crate) monthly_sales: Money,
    pub(crate) produced_batches: u64,
    pub(crate) input_prices: BTreeMap<GoodId, Money>,
}

impl World {
    /// Captures one bounded monthly observation before the monthly accounts are reset.
    /// # Errors
    /// Returns an error for an unknown firm or a missing observed input price.
    pub fn capture_monthly_firm_observation(
        &mut self,
        firm: FirmId,
    ) -> Result<FirmOperatingObservation, WorldError> {
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let recipe = self
            .production_recipes
            .get(&definition.recipe())
            .ok_or(WorldError::UnknownRecipe(definition.recipe()))?;
        let mut input_prices = BTreeMap::new();
        for input in recipe.inputs() {
            let purchase = self
                .monthly_firm_procurement_purchases
                .get(&(firm, input.good()))
                .copied()
                .filter(|(quantity, _)| quantity.get() > 0);
            let price = if let Some((quantity, spend)) = purchase {
                let unit_price = i128::from(spend.minor_units())
                    .checked_mul(i128::from(crate::QuantityMilli::SCALE))
                    .ok_or(WorldError::ArithmeticOverflow(
                        "observed procurement unit price",
                    ))?
                    / i128::from(quantity.get());
                Money::from_minor_units(i64::try_from(unit_price).map_err(|_| {
                    WorldError::ArithmeticOverflow("observed procurement unit price")
                })?)
            } else {
                self.regional_prices
                    .get(&(definition.region(), input.good()))
                    .copied()
                    .ok_or(WorldError::MissingRegionalPrice {
                        region: definition.region(),
                        good: input.good(),
                    })?
            };
            input_prices.insert(input.good(), price);
        }
        let accounts = self
            .firm_monthly_accounts
            .get(&firm)
            .copied()
            .unwrap_or_default();
        let mut market_outcomes = self
            .monthly_firm_market_outcomes
            .get(&firm)
            .cloned()
            .unwrap_or_default();
        market_outcomes.sort_by_key(|outcome| {
            (
                outcome.region,
                outcome.good,
                outcome.unit_price,
                outcome.seller,
                outcome.offered,
            )
        });
        let observation = FirmOperatingObservation {
            date: self.date,
            sales_revenue: accounts.sales_revenue(),
            final_sales_revenue: accounts.final_sales_revenue(),
            produced_batches: accounts.produced_batches(),
            input_prices,
            market_outcomes,
        };
        let history = self.firm_operating_history.entry(firm).or_default();
        if history.len() == FIRM_OBSERVATION_HISTORY_LIMIT {
            history.remove(0);
        }
        history.push(observation.clone());
        self.events.append(
            self.date,
            crate::DomainEvent::FirmOperatingObservationCaptured {
                firm,
                sales_revenue: observation.sales_revenue(),
                final_sales_revenue: observation.final_sales_revenue(),
                produced_batches: observation.produced_batches(),
                input_prices: observation
                    .input_prices()
                    .iter()
                    .map(|(good, price)| (*good, *price))
                    .collect(),
                market_outcomes: observation.market_outcomes().to_vec(),
            },
        );
        Ok(observation)
    }

    pub(crate) fn observed_operating_baseline(
        &self,
        firm: FirmId,
    ) -> Result<Option<ObservedOperatingBaseline>, WorldError> {
        let Some(history) = self
            .firm_operating_history
            .get(&firm)
            .filter(|history| !history.is_empty())
        else {
            return Ok(None);
        };
        let count = i128::try_from(history.len())
            .map_err(|_| WorldError::ArithmeticOverflow("firm observation count"))?;
        let sales = history.iter().try_fold(0_i128, |sum, observation| {
            sum.checked_add(i128::from(observation.sales_revenue().minor_units()))
                .ok_or(WorldError::ArithmeticOverflow("observed sales history"))
        })? / count;
        let batches = history.iter().try_fold(0_u128, |sum, observation| {
            sum.checked_add(u128::from(observation.produced_batches()))
                .ok_or(WorldError::ArithmeticOverflow(
                    "observed production history",
                ))
        })? / u128::try_from(history.len())
            .map_err(|_| WorldError::ArithmeticOverflow("firm observation count"))?;
        let mut price_totals: BTreeMap<GoodId, (i128, u64)> = BTreeMap::new();
        for observation in history {
            for (good, price) in observation.input_prices() {
                let row = price_totals.entry(*good).or_default();
                row.0 = row
                    .0
                    .checked_add(i128::from(price.minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow("observed input prices"))?;
                row.1 = row
                    .1
                    .checked_add(1)
                    .ok_or(WorldError::ArithmeticOverflow("observed price count"))?;
            }
        }
        let input_prices = price_totals
            .into_iter()
            .map(|(good, (sum, samples))| {
                let average = sum / i128::from(samples);
                Ok((
                    good,
                    Money::from_minor_units(
                        i64::try_from(average)
                            .map_err(|_| WorldError::ArithmeticOverflow("average input price"))?,
                    ),
                ))
            })
            .collect::<Result<_, WorldError>>()?;
        Ok(Some(ObservedOperatingBaseline {
            monthly_sales: Money::from_minor_units(
                i64::try_from(sales)
                    .map_err(|_| WorldError::ArithmeticOverflow("average observed sales"))?,
            ),
            produced_batches: u64::try_from(batches)
                .map_err(|_| WorldError::ArithmeticOverflow("average produced batches"))?,
            input_prices,
        }))
    }

    #[must_use]
    pub fn firm_operating_history(&self) -> &BTreeMap<FirmId, Vec<FirmOperatingObservation>> {
        &self.firm_operating_history
    }
    #[must_use]
    pub fn monthly_firm_market_outcomes(&self) -> &BTreeMap<FirmId, Vec<MarketOfferOutcome>> {
        &self.monthly_firm_market_outcomes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Country, CountryId, Firm, FirmExpectationSource, FirmExpectations, Good, GoodId,
        MarketOfferOutcome, Population, ProductionRecipe, QuantityMilli, RecipeId, Region,
        RegionId, SimDate, WorldCommand, WorldSeed,
    };

    fn world() -> World {
        let mut world = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        world
            .register_good(Good::new(GoodId::new(1), "Output").expect("good"))
            .expect("good");
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
                    "Output recipe",
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
                    "Firm",
                    RegionId::new(1),
                    RecipeId::new(1),
                    3,
                    3,
                    Money::from_minor_units(1_000),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
    }

    #[test]
    fn stockout_history_produces_bounded_expansion_advice() {
        let mut world = world();
        world
            .record_firm_production(FirmId::new(1), 1)
            .expect("production record");
        world.monthly_firm_market_outcomes.insert(
            FirmId::new(1),
            vec![MarketOfferOutcome {
                seller: FirmId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                unit_price: Money::from_minor_units(10),
                offered: QuantityMilli::new(1_000),
                sold: QuantityMilli::new(1_000),
                unsold: QuantityMilli::default(),
                unmet_market_demand: QuantityMilli::new(2_000),
            }],
        );
        world
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("capture");
        world
            .update_firm_expectations(
                FirmId::new(1),
                FirmExpectations::new(
                    Money::from_minor_units(100),
                    Money::default(),
                    Money::default(),
                    1,
                    FirmExpectationSource::ObservedHistory,
                )
                .expect("expectations"),
            )
            .expect("expectation update");

        world.firm_production_targets.insert(FirmId::new(1), 1);
        let proposals = world
            .plan_observed_production_adjustments()
            .expect("production advice");
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].average_produced_batches, 1);
        assert_eq!(proposals[0].sales_supported_batches, 1);
        assert_eq!(proposals[0].market_demand_ceiling_batches, 3);
        assert_eq!(proposals[0].physically_feasible_batches, 3);
        assert_eq!(proposals[0].advisory_batches, 3);
        assert_eq!(proposals[0].stockout_observations, 1);
        assert_eq!(
            proposals[0].expected_operating_cash_margin,
            Some(Money::from_minor_units(100))
        );
    }

    #[test]
    fn history_is_bounded_and_capture_is_replayable() {
        let mut direct = world();
        direct.monthly_firm_market_outcomes.insert(
            FirmId::new(1),
            vec![MarketOfferOutcome {
                seller: FirmId::new(1),
                region: RegionId::new(1),
                good: GoodId::new(1),
                unit_price: Money::from_minor_units(10),
                offered: QuantityMilli::new(1_000),
                sold: QuantityMilli::new(1_000),
                unsold: QuantityMilli::default(),
                unmet_market_demand: QuantityMilli::new(400),
            }],
        );
        for _ in 0..FIRM_OBSERVATION_HISTORY_LIMIT {
            direct
                .capture_monthly_firm_observation(FirmId::new(1))
                .expect("capture");
        }
        let mut replayed = direct.clone();
        direct
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("capture");
        WorldCommand::CaptureMonthlyFirmObservation {
            firm: FirmId::new(1),
        }
        .apply(&mut replayed)
        .expect("replay capture");
        assert_eq!(
            direct.firm_operating_history()[&FirmId::new(1)].len(),
            FIRM_OBSERVATION_HISTORY_LIMIT
        );
        assert!(
            direct.firm_operating_history()[&FirmId::new(1)][0].market_outcomes()[0]
                .sold_out_while_demand_remained()
        );
        assert_eq!(direct, replayed);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
    }
}
