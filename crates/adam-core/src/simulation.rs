use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BasisPoints, CohortId, CountryId, DomainEvent, FirmId, GoodId, Money, Population,
    QuantityMilli, RatePpm, RegionId, World, WorldError,
};

const DEMOGRAPHY_DOMAIN: u64 = 0x4445_4d4f_4752_4150;
const ECONOMY_DOMAIN: u64 = 0x4543_4f4e_4f4d_5900;
const POLITICS_DOMAIN: u64 = 0x504f_4c49_5449_4353;
const RATE_SCALE: i128 = 1_000_000;
const SALES_TAX_BPS: i128 = 2_000;

#[derive(Clone, Debug)]
struct RegionUpdate {
    id: RegionId,
    population: Population,
    population_rate: RatePpm,
    annual_output: Money,
    output_rate: RatePpm,
    material_components: Option<(Money, Money)>,
    cohort_populations: Vec<(CohortId, Population)>,
}

#[derive(Clone, Copy, Debug)]
struct FirmTaxUpdate {
    firm: FirmId,
    country: CountryId,
    taxable_sales: Money,
    liability: Money,
    paid: Money,
}

#[derive(Clone, Copy, Debug)]
struct CountryUpdate {
    id: CountryId,
    revenue: Money,
    spending: Money,
    treasury: Money,
    debt: Money,
    legitimacy: BasisPoints,
    cohesion: BasisPoints,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EconomicYearResult {
    pub closed_year: i32,
    pub months: Vec<crate::MonthlyEconomicCycleResult>,
}

impl World {
    /// Simulates one complete causal year as an atomic state transition.
    ///
    /// System order is demographics, regional production, fiscal closure, then politics.
    /// Randomness is isolated by subsystem, entity, and year.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] without changing the world if date or fixed-point arithmetic
    /// cannot be represented.
    pub fn advance_one_year(&mut self) -> Result<(), WorldError> {
        let simulated_year = self.date.year();
        if self.last_annual_closure_year == Some(simulated_year) {
            return Err(WorldError::AnnualClosureAlreadyExecuted(simulated_year));
        }
        let mut close_date = self.date;
        close_date.advance_years(1)?;
        let region_updates = self.plan_regions(simulated_year)?;
        let country_updates = self.plan_countries(simulated_year, &region_updates)?;
        self.apply_year(
            simulated_year,
            close_date,
            &[],
            &region_updates,
            &country_updates,
        );
        Ok(())
    }

    /// Simulates several complete years in order.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError`] if a year cannot be represented. Every previously completed year
    /// remains committed; the failing year is atomic and leaves no partial changes.
    pub fn advance_years(&mut self, years: u32) -> Result<(), WorldError> {
        for _ in 0..years {
            self.advance_one_year()?;
        }
        Ok(())
    }

    /// Runs twelve atomic economic months followed by one annual demographic/fiscal closure.
    /// # Errors
    /// Returns the first monthly or annual error without changing the failing year.
    pub fn advance_economic_year(&mut self) -> Result<EconomicYearResult, WorldError> {
        let closed_year = self.date.year();
        if self.last_annual_closure_year == Some(closed_year) {
            return Err(WorldError::AnnualClosureAlreadyExecuted(closed_year));
        }
        let mut next = self.clone();
        let opening_inventories: BTreeMap<FirmId, BTreeMap<GoodId, QuantityMilli>> = next
            .firms
            .iter()
            .map(|(id, firm)| (*id, firm.inventories().clone()))
            .collect();
        let mut months = Vec::with_capacity(12);
        for _ in 0..12 {
            months.push(next.execute_monthly_economic_cycle()?);
        }
        let region_updates =
            next.plan_material_regions(closed_year, &opening_inventories, &months)?;
        let tax_updates = next.plan_firm_sales_taxes()?;
        let country_updates =
            next.plan_material_countries(closed_year, &region_updates, &tax_updates)?;
        let close_date = next.date;
        next.apply_year(
            closed_year,
            close_date,
            &tax_updates,
            &region_updates,
            &country_updates,
        );
        next.events.append(
            close_date,
            DomainEvent::EconomicYearCompleted {
                closed_year,
                monthly_cycles: 12,
            },
        );
        let result = EconomicYearResult {
            closed_year,
            months,
        };
        *self = next;
        Ok(result)
    }

    /// Runs several monthly economic years, committing each completed year in order.
    /// # Errors
    /// Returns the first failing year while preserving earlier completed years.
    pub fn advance_economic_years(&mut self, years: u32) -> Result<(), WorldError> {
        for _ in 0..years {
            self.advance_economic_year()?;
        }
        Ok(())
    }

    fn plan_regions(&self, simulated_year: i32) -> Result<Vec<RegionUpdate>, WorldError> {
        self.regions
            .values()
            .map(|region| {
                let indicators = self
                    .countries
                    .get(&region.country())
                    .expect("registered region country exists")
                    .indicators();
                let population_rate = population_rate(
                    self.seed,
                    simulated_year,
                    region.id(),
                    region.population(),
                    region.annual_output(),
                    indicators.legitimacy(),
                );
                let population = apply_population_rate(region.population(), population_rate)?;
                let output_rate = output_rate(
                    self.seed,
                    simulated_year,
                    region.id(),
                    population_rate,
                    indicators.legitimacy(),
                    indicators.elite_cohesion(),
                );
                let annual_output = apply_money_rate(region.annual_output(), output_rate)?;
                let cohort_populations =
                    self.plan_region_cohort_rescale(region.id(), population)?;
                Ok(RegionUpdate {
                    id: region.id(),
                    population,
                    population_rate,
                    annual_output,
                    output_rate,
                    material_components: None,
                    cohort_populations,
                })
            })
            .collect()
    }

    fn plan_material_regions(
        &self,
        simulated_year: i32,
        opening_inventories: &BTreeMap<FirmId, BTreeMap<GoodId, QuantityMilli>>,
        months: &[crate::MonthlyEconomicCycleResult],
    ) -> Result<Vec<RegionUpdate>, WorldError> {
        let mut final_consumption = BTreeMap::<RegionId, i128>::new();
        for month in months {
            for fill in &month.commercial.clearing.fills {
                let region = self
                    .firms
                    .get(&fill.seller)
                    .expect("settled market seller exists")
                    .region();
                let total = final_consumption.entry(region).or_default();
                *total = total
                    .checked_add(i128::from(fill.spend.minor_units()))
                    .ok_or(WorldError::ArithmeticOverflow("annual final consumption"))?;
            }
        }
        let inventory_change = self.plan_regional_inventory_change(opening_inventories)?;
        self.regions
            .values()
            .map(|region| {
                let indicators = self
                    .countries
                    .get(&region.country())
                    .expect("registered region country exists")
                    .indicators();
                let population_rate = population_rate(
                    self.seed,
                    simulated_year,
                    region.id(),
                    region.population(),
                    region.annual_output(),
                    indicators.legitimacy(),
                );
                let population = apply_population_rate(region.population(), population_rate)?;
                let consumption = *final_consumption.get(&region.id()).unwrap_or(&0);
                let inventories = *inventory_change.get(&region.id()).unwrap_or(&0);
                let measured =
                    consumption
                        .checked_add(inventories)
                        .ok_or(WorldError::ArithmeticOverflow(
                            "annual measured regional output",
                        ))?;
                let annual_output =
                    money_from_i128(measured.max(0), "annual measured regional output")?;
                let output_rate = realized_output_rate(region.annual_output(), annual_output);
                let cohort_populations =
                    self.plan_region_cohort_rescale(region.id(), population)?;
                Ok(RegionUpdate {
                    id: region.id(),
                    population,
                    population_rate,
                    annual_output,
                    output_rate,
                    material_components: Some((
                        money_from_i128(consumption, "annual final consumption")?,
                        money_from_i128(inventories, "annual inventory change")?,
                    )),
                    cohort_populations,
                })
            })
            .collect()
    }

    fn plan_regional_inventory_change(
        &self,
        opening: &BTreeMap<FirmId, BTreeMap<GoodId, QuantityMilli>>,
    ) -> Result<BTreeMap<RegionId, i128>, WorldError> {
        let mut changes = BTreeMap::<RegionId, i128>::new();
        for firm in self.firms.values() {
            let empty = BTreeMap::new();
            let before = opening.get(&firm.id()).unwrap_or(&empty);
            let goods: BTreeSet<_> = before
                .keys()
                .chain(firm.inventories().keys())
                .copied()
                .collect();
            for good in goods {
                let previous = before.get(&good).copied().unwrap_or_default().get();
                let current = firm
                    .inventories()
                    .get(&good)
                    .copied()
                    .unwrap_or_default()
                    .get();
                let delta = i128::from(current) - i128::from(previous);
                let price = self.regional_prices.get(&(firm.region(), good)).ok_or(
                    WorldError::MissingRegionalPrice {
                        region: firm.region(),
                        good,
                    },
                )?;
                let value = delta.checked_mul(i128::from(price.minor_units())).ok_or(
                    WorldError::ArithmeticOverflow("regional inventory valuation"),
                )? / i128::from(QuantityMilli::SCALE);
                let total = changes.entry(firm.region()).or_default();
                *total = total
                    .checked_add(value)
                    .ok_or(WorldError::ArithmeticOverflow(
                        "annual regional inventory change",
                    ))?;
            }
        }
        Ok(changes)
    }

    fn plan_firm_sales_taxes(&self) -> Result<Vec<FirmTaxUpdate>, WorldError> {
        self.firms
            .values()
            .filter_map(|firm| {
                let history = self.firm_operating_history.get(&firm.id())?;
                let taxable_sales = history.iter().try_fold(0_i128, |total, observation| {
                    total
                        .checked_add(i128::from(observation.sales_revenue().minor_units()))
                        .ok_or(WorldError::ArithmeticOverflow("annual taxable firm sales"))
                });
                Some((firm, taxable_sales))
            })
            .map(|(firm, taxable_sales)| {
                let taxable_sales = taxable_sales?;
                let liability = taxable_sales
                    .checked_mul(SALES_TAX_BPS)
                    .ok_or(WorldError::ArithmeticOverflow("annual firm sales tax"))?
                    / 10_000;
                let liability = money_from_i128(liability, "annual firm sales tax")?;
                let paid = Money::from_minor_units(
                    firm.cash()
                        .minor_units()
                        .max(0)
                        .min(liability.minor_units()),
                );
                let country = self
                    .regions
                    .get(&firm.region())
                    .expect("registered firm region exists")
                    .country();
                Ok(FirmTaxUpdate {
                    firm: firm.id(),
                    country,
                    taxable_sales: money_from_i128(taxable_sales, "annual taxable firm sales")?,
                    liability,
                    paid,
                })
            })
            .collect()
    }

    fn plan_material_countries(
        &self,
        simulated_year: i32,
        region_updates: &[RegionUpdate],
        tax_updates: &[FirmTaxUpdate],
    ) -> Result<Vec<CountryUpdate>, WorldError> {
        let mut old_output = BTreeMap::<CountryId, i128>::new();
        let mut new_output = BTreeMap::<CountryId, i128>::new();
        let mut revenue = BTreeMap::<CountryId, i128>::new();
        for region in self.regions.values() {
            *old_output.entry(region.country()).or_default() +=
                i128::from(region.annual_output().minor_units());
        }
        for update in region_updates {
            let country = self
                .regions
                .get(&update.id)
                .expect("planned region exists")
                .country();
            *new_output.entry(country).or_default() +=
                i128::from(update.annual_output.minor_units());
        }
        for update in tax_updates {
            let total = revenue.entry(update.country).or_default();
            *total = total
                .checked_add(i128::from(update.paid.minor_units()))
                .ok_or(WorldError::ArithmeticOverflow("annual country tax revenue"))?;
        }
        self.countries
            .values()
            .map(|country| {
                plan_material_country(
                    self.seed,
                    simulated_year,
                    country.id(),
                    country.indicators(),
                    *old_output.get(&country.id()).unwrap_or(&0),
                    *new_output.get(&country.id()).unwrap_or(&0),
                    *revenue.get(&country.id()).unwrap_or(&0),
                )
            })
            .collect()
    }

    fn plan_countries(
        &self,
        simulated_year: i32,
        region_updates: &[RegionUpdate],
    ) -> Result<Vec<CountryUpdate>, WorldError> {
        let mut old_output = BTreeMap::<CountryId, i128>::new();
        let mut new_output = BTreeMap::<CountryId, i128>::new();
        for region in self.regions.values() {
            *old_output.entry(region.country()).or_default() +=
                i128::from(region.annual_output().minor_units());
        }
        for update in region_updates {
            let country = self
                .regions
                .get(&update.id)
                .expect("planned region exists")
                .country();
            *new_output.entry(country).or_default() +=
                i128::from(update.annual_output.minor_units());
        }

        self.countries
            .values()
            .map(|country| {
                plan_country(
                    self.seed,
                    simulated_year,
                    country.id(),
                    country.indicators(),
                    *old_output.get(&country.id()).unwrap_or(&0),
                    *new_output.get(&country.id()).unwrap_or(&0),
                )
            })
            .collect()
    }

    fn record_region_output_events(&mut self, close_date: crate::SimDate, update: &RegionUpdate) {
        if let Some((final_consumption, inventory_change)) = update.material_components {
            self.events.append(
                close_date,
                DomainEvent::RegionalOutputMeasured {
                    region: update.id,
                    final_consumption,
                    inventory_change,
                    annual_output: update.annual_output,
                },
            );
        }
        self.events.append(
            close_date,
            DomainEvent::RegionOutputChanged {
                region: update.id,
                annual_output: update.annual_output,
                rate: update.output_rate,
            },
        );
    }

    fn apply_year(
        &mut self,
        simulated_year: i32,
        close_date: crate::SimDate,
        tax_updates: &[FirmTaxUpdate],
        region_updates: &[RegionUpdate],
        country_updates: &[CountryUpdate],
    ) {
        for update in tax_updates {
            self.firms
                .get_mut(&update.firm)
                .expect("planned tax firm exists")
                .debit_cash(update.paid)
                .expect("planned tax payment is liquid");
            self.events.append(
                close_date,
                DomainEvent::FirmSalesTaxPaid {
                    firm: update.firm,
                    country: update.country,
                    taxable_sales: update.taxable_sales,
                    liability: update.liability,
                    paid: update.paid,
                },
            );
        }
        for update in region_updates {
            let region = self
                .regions
                .get_mut(&update.id)
                .expect("planned region exists");
            region.set_population(update.population);
            region.set_annual_output(update.annual_output);
            self.events.append(
                close_date,
                DomainEvent::RegionPopulationChanged {
                    region: update.id,
                    population: update.population,
                    rate: update.population_rate,
                },
            );
            self.record_region_output_events(close_date, update);
            for (cohort_id, people) in &update.cohort_populations {
                self.cohorts
                    .get_mut(cohort_id)
                    .expect("planned cohort exists")
                    .set_people(*people);
                self.events.append(
                    close_date,
                    DomainEvent::HouseholdCohortPopulationChanged {
                        cohort: *cohort_id,
                        people: *people,
                    },
                );
            }
        }
        for update in country_updates {
            let country = self
                .countries
                .get_mut(&update.id)
                .expect("planned country exists");
            let indicators = country.indicators_mut();
            indicators.set_treasury(update.treasury);
            indicators.set_public_debt(update.debt);
            indicators.set_legitimacy(update.legitimacy);
            indicators.set_elite_cohesion(update.cohesion);
            self.events.append(
                close_date,
                DomainEvent::CountryFiscalYearClosed {
                    country: update.id,
                    revenue: update.revenue,
                    spending: update.spending,
                    treasury: update.treasury,
                    debt: update.debt,
                },
            );
            self.events.append(
                close_date,
                DomainEvent::CountryPoliticsChanged {
                    country: update.id,
                    legitimacy: update.legitimacy,
                    elite_cohesion: update.cohesion,
                },
            );
        }
        self.date = close_date;
        self.last_annual_closure_year = Some(simulated_year);
        self.events.append(
            close_date,
            DomainEvent::YearAdvanced {
                year: close_date.year(),
            },
        );
    }
}

fn population_rate(
    seed: crate::WorldSeed,
    year: i32,
    region: RegionId,
    population: Population,
    output: Money,
    legitimacy: BasisPoints,
) -> RatePpm {
    let output_per_person = if population.people() == 0 {
        0
    } else {
        i128::from(output.minor_units()) / i128::from(population.people())
    };
    let prosperity = ((output_per_person - 3_000_000) / 2_000).clamp(-3_000, 3_000);
    let legitimacy_effect = (i32::from(legitimacy.get()) - 5_000) / 2;
    let mut random = seed.stream_for(DEMOGRAPHY_DOMAIN, region.get(), year);
    let shock = centered(&mut random, 1_000);
    let raw =
        4_000 + i32::try_from(prosperity).expect("clamped to i32") + legitimacy_effect + shock;
    RatePpm::new(raw.clamp(-30_000, 30_000)).expect("clamped population rate")
}

fn realized_output_rate(previous: Money, current: Money) -> RatePpm {
    let previous = i128::from(previous.minor_units());
    let current = i128::from(current.minor_units());
    let rate = if previous == 0 {
        if current == 0 { 0 } else { 1_000_000 }
    } else {
        ((current - previous) * RATE_SCALE / previous.abs()).clamp(-1_000_000, 1_000_000)
    };
    RatePpm::new(i32::try_from(rate).expect("clamped realized output rate"))
        .expect("valid realized output rate")
}

fn output_rate(
    seed: crate::WorldSeed,
    year: i32,
    region: RegionId,
    population_rate: RatePpm,
    legitimacy: BasisPoints,
    cohesion: BasisPoints,
) -> RatePpm {
    let legitimacy_effect = i32::from(legitimacy.get()) - 5_000;
    let cohesion_effect = i32::from(cohesion.get()) - 5_000;
    let mut random = seed.stream_for(ECONOMY_DOMAIN, region.get(), year);
    let shock = centered(&mut random, 12_000);
    let raw = 15_000 + population_rate.get() / 2 + legitimacy_effect + cohesion_effect + shock;
    RatePpm::new(raw.clamp(-150_000, 200_000)).expect("clamped output rate")
}

fn plan_material_country(
    seed: crate::WorldSeed,
    year: i32,
    id: CountryId,
    indicators: crate::CountryIndicators,
    old_output: i128,
    new_output: i128,
    tax_revenue: i128,
) -> Result<CountryUpdate, WorldError> {
    let spending_bps = 2_200 + i128::from(10_000 - indicators.legitimacy().get()) / 10;
    let revenue = money_from_i128(tax_revenue, "collected sales tax revenue")?;
    let spending = money_from_i128(new_output * spending_bps / 10_000, "fiscal spending")?;
    let balance = i128::from(revenue.minor_units()) - i128::from(spending.minor_units());
    let (treasury, debt) = close_budget(indicators.treasury(), indicators.public_debt(), balance)?;
    let growth_ppm = if old_output == 0 {
        0
    } else {
        ((new_output - old_output) * RATE_SCALE / old_output).clamp(-1_000_000, 1_000_000)
    };
    let fiscal_signal = if balance >= 0 { 35 } else { -70 };
    let mut random = seed.stream_for(POLITICS_DOMAIN, id.get(), year);
    let political_shock = centered(&mut random, 90);
    let growth_signal = i32::try_from(growth_ppm / 250).expect("growth signal fits i32");
    let legitimacy = indicators
        .legitimacy()
        .shifted(growth_signal + fiscal_signal + political_shock);
    let cohesion = indicators
        .elite_cohesion()
        .shifted(fiscal_signal / 2 + political_shock / 3);
    Ok(CountryUpdate {
        id,
        revenue,
        spending,
        treasury,
        debt,
        legitimacy,
        cohesion,
    })
}

fn plan_country(
    seed: crate::WorldSeed,
    year: i32,
    id: CountryId,
    indicators: crate::CountryIndicators,
    old_output: i128,
    new_output: i128,
) -> Result<CountryUpdate, WorldError> {
    let revenue_bps = 1_600 + i128::from(indicators.elite_cohesion().get()) / 20;
    let spending_bps = 2_200 + i128::from(10_000 - indicators.legitimacy().get()) / 10;
    let revenue = money_from_i128(new_output * revenue_bps / 10_000, "fiscal revenue")?;
    let spending = money_from_i128(new_output * spending_bps / 10_000, "fiscal spending")?;
    let balance = i128::from(revenue.minor_units()) - i128::from(spending.minor_units());
    let (treasury, debt) = close_budget(indicators.treasury(), indicators.public_debt(), balance)?;
    let growth_ppm = if old_output == 0 {
        0
    } else {
        ((new_output - old_output) * RATE_SCALE / old_output).clamp(-1_000_000, 1_000_000)
    };
    let fiscal_signal = if balance >= 0 { 35 } else { -70 };
    let mut random = seed.stream_for(POLITICS_DOMAIN, id.get(), year);
    let political_shock = centered(&mut random, 90);
    let growth_signal = i32::try_from(growth_ppm / 250).expect("growth signal fits i32");
    let legitimacy = indicators
        .legitimacy()
        .shifted(growth_signal + fiscal_signal + political_shock);
    let cohesion = indicators
        .elite_cohesion()
        .shifted(fiscal_signal / 2 + political_shock / 3);
    Ok(CountryUpdate {
        id,
        revenue,
        spending,
        treasury,
        debt,
        legitimacy,
        cohesion,
    })
}

fn close_budget(treasury: Money, debt: Money, balance: i128) -> Result<(Money, Money), WorldError> {
    let mut treasury_value = i128::from(treasury.minor_units());
    let mut debt_value = i128::from(debt.minor_units());
    if balance >= 0 {
        let repayment = (balance / 2).min(debt_value.max(0));
        debt_value -= repayment;
        treasury_value += balance - repayment;
    } else {
        let deficit = -balance;
        let draw = deficit.min(treasury_value.max(0));
        treasury_value -= draw;
        debt_value += deficit - draw;
    }
    Ok((
        money_from_i128(treasury_value, "treasury closure")?,
        money_from_i128(debt_value, "debt closure")?,
    ))
}

fn apply_population_rate(value: Population, rate: RatePpm) -> Result<Population, WorldError> {
    let base = i128::from(value.people());
    let result = base + base * i128::from(rate.get()) / RATE_SCALE;
    u64::try_from(result)
        .map(Population::new)
        .map_err(|_| WorldError::ArithmeticOverflow("population growth"))
}

fn apply_money_rate(value: Money, rate: RatePpm) -> Result<Money, WorldError> {
    let base = i128::from(value.minor_units());
    money_from_i128(
        base + base * i128::from(rate.get()) / RATE_SCALE,
        "regional output growth",
    )
}

fn money_from_i128(value: i128, operation: &'static str) -> Result<Money, WorldError> {
    i64::try_from(value)
        .map(Money::from_minor_units)
        .map_err(|_| WorldError::ArithmeticOverflow(operation))
}

fn centered(random: &mut crate::RandomStream, amplitude: u64) -> i32 {
    let width = amplitude * 2 + 1;
    let draw = random.next_bounded(width).expect("non-zero random width");
    i32::try_from(draw).expect("small random draw")
        - i32::try_from(amplitude).expect("small amplitude")
}

#[cfg(test)]
mod tests {
    use crate::{
        Country, CountryId, CountryIndicators, Money, Population, Region, RegionId, SimDate,
        WorldSeed,
    };

    use super::*;

    fn economy(seed: u64, legitimacy: u16) -> World {
        let indicators = CountryIndicators::new(
            Money::from_minor_units(0),
            Money::from_minor_units(0),
            BasisPoints::new(legitimacy).expect("valid legitimacy"),
            BasisPoints::new(5_000).expect("valid cohesion"),
        );
        let mut world = World::new(
            WorldSeed::new(seed),
            SimDate::new(2025, 1).expect("valid date"),
        );
        world
            .register_country(
                Country::new(CountryId::new(1), "A")
                    .expect("country")
                    .with_indicators(indicators),
            )
            .expect("country");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "Capital",
                    Population::new(1_000_000),
                    Money::from_minor_units(4_000_000_000_000),
                )
                .expect("region"),
            )
            .expect("region");
        world
    }

    #[test]
    fn twin_years_are_identical() {
        let mut first = economy(47, 5_000);
        let mut second = first.clone();
        first.advance_years(20).expect("simulation succeeds");
        second.advance_years(20).expect("simulation succeeds");
        assert_eq!(first, second);
    }

    #[test]
    fn seed_changes_history_not_only_fingerprint_header() {
        let mut first = economy(47, 5_000);
        let mut second = economy(48, 5_000);
        first.advance_one_year().expect("simulation succeeds");
        second.advance_one_year().expect("simulation succeeds");
        assert_ne!(first.regions(), second.regions());
    }

    #[test]
    fn legitimacy_has_causal_demographic_effect() {
        let mut stable = economy(47, 8_000);
        let mut fragile = economy(47, 2_000);
        stable.advance_one_year().expect("simulation succeeds");
        fragile.advance_one_year().expect("simulation succeeds");
        let stable_population = stable.regions()[&RegionId::new(1)].population();
        let fragile_population = fragile.regions()[&RegionId::new(1)].population();
        assert!(stable_population > fragile_population);
    }

    #[test]
    fn economic_year_without_trade_eliminates_phantom_output() {
        let mut world = economy(47, 5_000);
        world
            .regions
            .get_mut(&RegionId::new(1))
            .expect("region")
            .set_population(Population::default());
        world
            .advance_economic_year()
            .expect("material economic year");

        assert_eq!(
            world.regions()[&RegionId::new(1)].annual_output(),
            Money::default()
        );
        assert!(world.events().events().iter().any(|event| matches!(
            event.event(),
            DomainEvent::CountryFiscalYearClosed {
                revenue,
                spending,
                ..
            } if *revenue == Money::default() && *spending == Money::default()
        )));
    }

    #[test]
    fn persistent_deficit_becomes_public_debt() {
        let mut world = economy(47, 5_000);
        world.advance_one_year().expect("simulation succeeds");
        assert!(
            world.countries()[&CountryId::new(1)]
                .indicators()
                .public_debt()
                .minor_units()
                > 0
        );
    }
}
