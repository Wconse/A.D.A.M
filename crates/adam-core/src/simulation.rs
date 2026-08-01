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
const DEBT_BRAKE_RATIO_DIVISOR: i128 = 10;
const DEBT_BRAKE_MAX_BPS: i128 = 600;
const SPENDING_FLOOR_BPS: i128 = 1_200;
const DEBT_INTEREST_BPS: i128 = 300;
const DEBT_RESTRUCTURING_OUTPUT_MULTIPLE: i128 = 2;
const DEBT_RESTRUCTURING_INTEREST_REVENUE_MULTIPLE: i128 = 3;
const DEBT_RESTRUCTURING_HAIRCUT_BPS: i128 = 4_000;
const DEBT_RESTRUCTURING_LEGITIMACY_SHOCK: i32 = -800;
const DEBT_RESTRUCTURING_COHESION_SHOCK: i32 = -500;

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
    /// Share of the executed spending that is actually paid out to households.
    /// The legacy non-material scaffold pays nothing, because its revenue is a
    /// coefficient of output rather than tax collected from real firms.
    household_outlay: Money,
    treasury: Money,
    debt: Money,
    opening_debt: Money,
    interest: Money,
    debt_before_restructuring: Money,
    principal_written_off: Money,
    social_pressure: BasisPoints,
    legitimacy_effect: i32,
    regional_confidence: BasisPoints,
    regional_confidence_effect: i32,
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
        let social_pressure = self.plan_annual_regional_social_pressure()?;
        let country_updates =
            self.plan_countries(simulated_year, &region_updates, &social_pressure)?;
        self.apply_year(
            simulated_year,
            close_date,
            &[],
            &region_updates,
            &social_pressure,
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
        let social_pressure = next.plan_annual_regional_social_pressure()?;
        let country_updates = next.plan_material_countries(
            closed_year,
            &region_updates,
            &tax_updates,
            &social_pressure,
        )?;
        let close_date = next.date;
        next.apply_year(
            closed_year,
            close_date,
            &tax_updates,
            &region_updates,
            &social_pressure,
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

    /// Latest observed transaction price for one firm and good in the closed year.
    ///
    /// Prefers the buyer-side price the firm actually paid for the good, then the
    /// firm's latest settled seller-side outcome price. `None` means the bounded
    /// operating history holds no transaction evidence for this good, and the
    /// caller falls back to the regional reference table.
    fn observed_inventory_valuation_price(&self, firm: FirmId, good: GoodId) -> Option<Money> {
        let history = self.firm_operating_history.get(&firm)?;
        for observation in history.iter().rev() {
            if let Some(price) = observation.input_prices().get(&good) {
                return Some(*price);
            }
            if let Some(outcome) = observation
                .market_outcomes()
                .iter()
                .rev()
                .find(|outcome| outcome.good == good && outcome.sold.get() > 0)
            {
                return Some(outcome.unit_price);
            }
        }
        None
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
                let price = match self.observed_inventory_valuation_price(firm.id(), good) {
                    Some(observed) => observed,
                    None => self
                        .regional_prices
                        .get(&(firm.region(), good))
                        .copied()
                        .ok_or(WorldError::MissingRegionalPrice {
                            region: firm.region(),
                            good,
                        })?,
                };
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

    /// Sales tax is levied on final household sales only; intermediate
    /// firm-to-firm turnover is untaxed. Total revenue remains untaxed
    /// decision evidence for firm management.
    fn plan_firm_sales_taxes(&self) -> Result<Vec<FirmTaxUpdate>, WorldError> {
        self.firms
            .values()
            .filter_map(|firm| {
                let history = self.firm_operating_history.get(&firm.id())?;
                let taxable_sales = history.iter().try_fold(0_i128, |total, observation| {
                    total
                        .checked_add(i128::from(observation.final_sales_revenue().minor_units()))
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
        social_pressure: &BTreeMap<RegionId, crate::RegionalSocialPressure>,
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
                    self.country_social_pressure(country.id(), social_pressure),
                    self.country_regional_confidence(country.id()),
                )
            })
            .collect()
    }

    fn plan_countries(
        &self,
        simulated_year: i32,
        region_updates: &[RegionUpdate],
        social_pressure: &BTreeMap<RegionId, crate::RegionalSocialPressure>,
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
                    self.country_social_pressure(country.id(), social_pressure),
                    self.country_regional_confidence(country.id()),
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

    fn apply_country_update(&mut self, close_date: crate::SimDate, update: &CountryUpdate) {
        let country = self
            .countries
            .get_mut(&update.id)
            .expect("planned country exists");
        let indicators = country.indicators_mut();
        indicators.set_treasury(update.treasury);
        indicators.set_public_debt(update.debt);
        indicators.set_legitimacy(update.legitimacy);
        indicators.set_elite_cohesion(update.cohesion);
        if update.interest.minor_units() > 0 {
            self.events.append(
                close_date,
                DomainEvent::PublicDebtInterestCharged {
                    country: update.id,
                    opening_debt: update.opening_debt,
                    interest: update.interest,
                },
            );
        }
        if update.principal_written_off.minor_units() > 0 {
            self.events.append(
                close_date,
                DomainEvent::PublicDebtRestructured {
                    country: update.id,
                    debt_before: update.debt_before_restructuring,
                    debt_after: update.debt,
                    principal_written_off: update.principal_written_off,
                },
            );
        }
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
            DomainEvent::CountryLegitimacyPressureApplied {
                country: update.id,
                population_weighted_pressure: update.social_pressure,
                legitimacy_effect: update.legitimacy_effect,
            },
        );
        self.events.append(
            close_date,
            DomainEvent::CountryRegionalConfidenceApplied {
                country: update.id,
                population_weighted_confidence: update.regional_confidence,
                legitimacy_effect: update.regional_confidence_effect,
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

    #[allow(clippy::too_many_lines)]
    fn apply_year(
        &mut self,
        simulated_year: i32,
        close_date: crate::SimDate,
        tax_updates: &[FirmTaxUpdate],
        region_updates: &[RegionUpdate],
        social_pressure: &BTreeMap<RegionId, crate::RegionalSocialPressure>,
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
        let cohort_ids: Vec<_> = self.cohorts.keys().copied().collect();
        for cohort_id in cohort_ids {
            let transition = self
                .cohorts
                .get_mut(&cohort_id)
                .expect("registered cohort exists")
                .advance_lifecycle_year();
            let Some((previous_age_band, age_band)) = transition else {
                continue;
            };
            let retired_workers = if age_band == crate::AgeBand::Senior {
                self.employment_agreements
                    .values_mut()
                    .filter(|agreement| agreement.cohort() == cohort_id && agreement.active())
                    .map(|agreement| {
                        let workers = agreement.workers();
                        agreement.set_workers(0);
                        workers
                    })
                    .sum()
            } else {
                0
            };
            self.events.append(
                close_date,
                DomainEvent::HouseholdCohortAged {
                    cohort: cohort_id,
                    previous_age_band,
                    age_band,
                    retired_workers,
                },
            );
        }
        self.regional_social_pressure = social_pressure.clone();
        for (region, pressure) in social_pressure {
            self.events.append(
                close_date,
                DomainEvent::RegionalSocialPressureUpdated {
                    region: *region,
                    chronic_unemployment: pressure.chronic_unemployment(),
                    livelihood_stress: pressure.livelihood_stress(),
                    public_service_shortfall: pressure.public_service_shortfall(),
                    combined: pressure.combined(),
                },
            );
        }
        for country_update in country_updates {
            let service_budget =
                i128::from(country_update.spending.minor_units()).max(0) * 3_000 / 10_000;
            let (allocation_source, shares, regional_budgets, allocation_influences) =
                self.plan_regional_service_budgets(country_update.id, service_budget);
            for influence in allocation_influences {
                self.events.append(
                    close_date,
                    DomainEvent::RegionalServiceAllocationInfluenceApplied {
                        country: country_update.id,
                        actor: influence.actor,
                        office: influence.office,
                        region: influence.region,
                        kind: influence.kind,
                        weight: influence.weight,
                        score_bonus: influence.score_bonus,
                    },
                );
            }
            let regions: Vec<_> = self
                .regions
                .values()
                .filter(|region| region.country() == country_update.id)
                .map(crate::Region::id)
                .collect();
            for region in regions {
                let regional_budget = *regional_budgets.get(&region).unwrap_or(&0);
                let regional_output = region_updates
                    .iter()
                    .find(|update| update.id == region)
                    .map_or(0, |update| i128::from(update.annual_output.minor_units()));
                let target_value = if regional_output <= 0 {
                    0
                } else {
                    (regional_budget * 100_000 / regional_output).clamp(0, 10_000)
                };
                let funding_target =
                    BasisPoints::new(u16::try_from(target_value).unwrap_or(10_000))
                        .expect("regional service target is bounded");
                let share = shares.get(&region).copied().unwrap_or(BasisPoints::ZERO);
                let budget =
                    Money::from_minor_units(i64::try_from(regional_budget).unwrap_or(i64::MAX));
                self.events.append(
                    close_date,
                    DomainEvent::RegionalServiceBudgetAllocated {
                        country: country_update.id,
                        region,
                        source: allocation_source,
                        share,
                        service_budget: budget,
                    },
                );
                let services =
                    self.regional_public_services[&region].adjusted_toward_funding(funding_target);
                self.regional_public_services.insert(region, services);
                self.events.append(
                    close_date,
                    DomainEvent::RegionalPublicServicesUpdated {
                        region,
                        service_budget: budget,
                        funding_target,
                        healthcare: services.healthcare(),
                        infrastructure: services.infrastructure(),
                        administration: services.administration(),
                    },
                );
            }
        }
        for update in country_updates {
            self.apply_country_update(close_date, update);
            self.distribute_public_outlay(close_date, update);
        }
        self.update_annual_regional_interests(close_date)
            .expect("bounded annual regional policy evidence fits");
        self.execute_annual_internal_migration(close_date);
        self.execute_annual_housing_investment(close_date);
        self.date = close_date;
        self.last_annual_closure_year = Some(simulated_year);
        self.events.append(
            close_date,
            DomainEvent::YearAdvanced {
                year: close_date.year(),
            },
        );
    }

    /// Pays one country's executed public spending to its households.
    ///
    /// Public employment and public procurement are not modeled as separate
    /// counterparties yet, so every executed outlay reaches households directly
    /// as public wages, transfers, and domestic debt coupons. This is what keeps
    /// the fiscal circuit closed: tax leaves firm cash, becomes treasury cash,
    /// and returns to the economy as household purchasing power instead of
    /// disappearing from the world.
    fn distribute_public_outlay(&mut self, close_date: crate::SimDate, update: &CountryUpdate) {
        for (cohort, amount) in self.plan_public_outlay_shares(update.id, update.household_outlay) {
            self.cohorts
                .get_mut(&cohort)
                .expect("planned outlay cohort exists")
                .credit_wealth(amount)
                .expect("planned outlay share is non-negative");
            self.events.append(
                close_date,
                DomainEvent::PublicOutlayDistributed {
                    country: update.id,
                    cohort,
                    amount,
                },
            );
        }
    }

    /// Splits an outlay across a country's cohorts by population with the
    /// largest-remainder method, so the paid total equals the spent total to the
    /// minor unit and no money is created or destroyed in the split.
    fn plan_public_outlay_shares(
        &self,
        country: CountryId,
        outlay: Money,
    ) -> Vec<(CohortId, Money)> {
        let total = u128::try_from(outlay.minor_units().max(0)).expect("non-negative outlay");
        if total == 0 {
            return Vec::new();
        }
        let mut rows: Vec<(CohortId, u128, u128)> = Vec::new();
        let mut population = 0_u128;
        for cohort in self.cohorts.values() {
            let in_country = self
                .regions
                .get(&cohort.region())
                .is_some_and(|region| region.country() == country);
            let people = u128::from(cohort.people().people());
            if in_country && people > 0 {
                population += people;
                rows.push((cohort.id(), people, 0));
            }
        }
        if population == 0 {
            return Vec::new();
        }
        let mut assigned = 0_u128;
        for row in &mut rows {
            let numerator = total * row.1;
            row.1 = numerator / population;
            row.2 = numerator % population;
            assigned += row.1;
        }
        let mut order: Vec<usize> = (0..rows.len()).collect();
        order.sort_by(|&first, &second| {
            rows[second]
                .2
                .cmp(&rows[first].2)
                .then_with(|| rows[first].0.cmp(&rows[second].0))
        });
        let leftover = usize::try_from(total - assigned).expect("largest remainder fits");
        for index in order.into_iter().take(leftover) {
            rows[index].1 += 1;
        }
        rows.into_iter()
            .filter(|row| row.1 > 0)
            .map(|(id, amount, _)| {
                (
                    id,
                    Money::from_minor_units(
                        i64::try_from(amount).expect("outlay share fits currency"),
                    ),
                )
            })
            .collect()
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

#[allow(clippy::too_many_arguments)]
fn plan_material_country(
    seed: crate::WorldSeed,
    year: i32,
    id: CountryId,
    indicators: crate::CountryIndicators,
    old_output: i128,
    new_output: i128,
    tax_revenue: i128,
    social_pressure: BasisPoints,
    regional_confidence: BasisPoints,
) -> Result<CountryUpdate, WorldError> {
    let debt_for_brake = i128::from(indicators.public_debt().minor_units()).max(0);
    let debt_ratio_bps = if debt_for_brake == 0 {
        0
    } else if new_output <= 0 {
        DEBT_BRAKE_MAX_BPS * DEBT_BRAKE_RATIO_DIVISOR
    } else {
        debt_for_brake.saturating_mul(10_000) / new_output
    };
    let debt_brake_bps = (debt_ratio_bps / DEBT_BRAKE_RATIO_DIVISOR).min(DEBT_BRAKE_MAX_BPS);
    let spending_bps = (2_200 + i128::from(10_000 - indicators.legitimacy().get()) / 10
        - debt_brake_bps)
        .max(SPENDING_FLOOR_BPS);
    let revenue = money_from_i128(tax_revenue, "collected sales tax revenue")?;
    let interest =
        i128::from(indicators.public_debt().minor_units()).max(0) * DEBT_INTEREST_BPS / 10_000;
    let spending = money_from_i128(
        new_output * spending_bps / 10_000 + interest,
        "fiscal spending",
    )?;
    let balance = i128::from(revenue.minor_units()) - i128::from(spending.minor_units());
    let (treasury, debt_before_restructuring) =
        close_budget(indicators.treasury(), indicators.public_debt(), balance)?;
    let interest_money = money_from_i128(interest, "public debt interest")?;
    let (debt, principal_written_off) = plan_public_debt_restructuring(
        debt_before_restructuring,
        new_output,
        revenue,
        interest_money,
    )?;
    let growth_ppm = if old_output == 0 {
        0
    } else {
        ((new_output - old_output) * RATE_SCALE / old_output).clamp(-1_000_000, 1_000_000)
    };
    let fiscal_signal = if balance >= 0 { 35 } else { -70 };
    let mut random = seed.stream_for(POLITICS_DOMAIN, id.get(), year);
    let political_shock = centered(&mut random, 90);
    let growth_signal = i32::try_from(growth_ppm / 250).expect("growth signal fits i32");
    let legitimacy_effect = crate::political_economy::legitimacy_effect(social_pressure);
    let regional_confidence_effect =
        crate::regional_interests::regional_confidence_effect(regional_confidence);
    let mut legitimacy = indicators.legitimacy().shifted(
        growth_signal
            + fiscal_signal
            + political_shock
            + legitimacy_effect
            + regional_confidence_effect,
    );
    let mut cohesion = indicators
        .elite_cohesion()
        .shifted(fiscal_signal / 2 + political_shock / 3);
    if principal_written_off.minor_units() > 0 {
        legitimacy = legitimacy.shifted(DEBT_RESTRUCTURING_LEGITIMACY_SHOCK);
        cohesion = cohesion.shifted(DEBT_RESTRUCTURING_COHESION_SHOCK);
    }
    Ok(CountryUpdate {
        id,
        revenue,
        spending,
        household_outlay: spending,
        treasury,
        debt,
        opening_debt: indicators.public_debt(),
        interest: interest_money,
        debt_before_restructuring,
        principal_written_off,
        social_pressure,
        legitimacy_effect,
        regional_confidence,
        regional_confidence_effect,
        legitimacy,
        cohesion,
    })
}

#[allow(clippy::too_many_arguments)]
fn plan_country(
    seed: crate::WorldSeed,
    year: i32,
    id: CountryId,
    indicators: crate::CountryIndicators,
    old_output: i128,
    new_output: i128,
    social_pressure: BasisPoints,
    regional_confidence: BasisPoints,
) -> Result<CountryUpdate, WorldError> {
    let revenue_bps = 1_600 + i128::from(indicators.elite_cohesion().get()) / 20;
    let debt_for_brake = i128::from(indicators.public_debt().minor_units()).max(0);
    let debt_ratio_bps = if debt_for_brake == 0 {
        0
    } else if new_output <= 0 {
        DEBT_BRAKE_MAX_BPS * DEBT_BRAKE_RATIO_DIVISOR
    } else {
        debt_for_brake.saturating_mul(10_000) / new_output
    };
    let debt_brake_bps = (debt_ratio_bps / DEBT_BRAKE_RATIO_DIVISOR).min(DEBT_BRAKE_MAX_BPS);
    let spending_bps = (2_200 + i128::from(10_000 - indicators.legitimacy().get()) / 10
        - debt_brake_bps)
        .max(SPENDING_FLOOR_BPS);
    let revenue = money_from_i128(new_output * revenue_bps / 10_000, "fiscal revenue")?;
    let interest =
        i128::from(indicators.public_debt().minor_units()).max(0) * DEBT_INTEREST_BPS / 10_000;
    let spending = money_from_i128(
        new_output * spending_bps / 10_000 + interest,
        "fiscal spending",
    )?;
    let balance = i128::from(revenue.minor_units()) - i128::from(spending.minor_units());
    let (treasury, debt_before_restructuring) =
        close_budget(indicators.treasury(), indicators.public_debt(), balance)?;
    let interest_money = money_from_i128(interest, "public debt interest")?;
    let (debt, principal_written_off) = plan_public_debt_restructuring(
        debt_before_restructuring,
        new_output,
        revenue,
        interest_money,
    )?;
    let growth_ppm = if old_output == 0 {
        0
    } else {
        ((new_output - old_output) * RATE_SCALE / old_output).clamp(-1_000_000, 1_000_000)
    };
    let fiscal_signal = if balance >= 0 { 35 } else { -70 };
    let mut random = seed.stream_for(POLITICS_DOMAIN, id.get(), year);
    let political_shock = centered(&mut random, 90);
    let growth_signal = i32::try_from(growth_ppm / 250).expect("growth signal fits i32");
    let legitimacy_effect = crate::political_economy::legitimacy_effect(social_pressure);
    let regional_confidence_effect =
        crate::regional_interests::regional_confidence_effect(regional_confidence);
    let mut legitimacy = indicators.legitimacy().shifted(
        growth_signal
            + fiscal_signal
            + political_shock
            + legitimacy_effect
            + regional_confidence_effect,
    );
    let mut cohesion = indicators
        .elite_cohesion()
        .shifted(fiscal_signal / 2 + political_shock / 3);
    if principal_written_off.minor_units() > 0 {
        legitimacy = legitimacy.shifted(DEBT_RESTRUCTURING_LEGITIMACY_SHOCK);
        cohesion = cohesion.shifted(DEBT_RESTRUCTURING_COHESION_SHOCK);
    }
    Ok(CountryUpdate {
        id,
        revenue,
        spending,
        household_outlay: Money::default(),
        treasury,
        debt,
        opening_debt: indicators.public_debt(),
        interest: interest_money,
        debt_before_restructuring,
        principal_written_off,
        social_pressure,
        legitimacy_effect,
        regional_confidence,
        regional_confidence_effect,
        legitimacy,
        cohesion,
    })
}

fn plan_public_debt_restructuring(
    debt: Money,
    annual_output: i128,
    revenue: Money,
    interest: Money,
) -> Result<(Money, Money), WorldError> {
    let debt_value = i128::from(debt.minor_units()).max(0);
    let output_value = annual_output.max(0);
    let revenue_value = i128::from(revenue.minor_units()).max(0);
    let interest_value = i128::from(interest.minor_units()).max(0);
    let debt_exceeds_capacity =
        debt_value > output_value.saturating_mul(DEBT_RESTRUCTURING_OUTPUT_MULTIPLE);
    let service_exceeds_revenue = interest_value
        .saturating_mul(DEBT_RESTRUCTURING_INTEREST_REVENUE_MULTIPLE)
        >= revenue_value.max(1);
    if !debt_exceeds_capacity || !service_exceeds_revenue {
        return Ok((debt, Money::default()));
    }
    let written_off = debt_value
        .checked_mul(DEBT_RESTRUCTURING_HAIRCUT_BPS)
        .ok_or(WorldError::ArithmeticOverflow("public debt restructuring"))?
        / 10_000;
    let remaining = debt_value
        .checked_sub(written_off)
        .ok_or(WorldError::ArithmeticOverflow("public debt restructuring"))?;
    Ok((
        money_from_i128(remaining, "restructured public debt")?,
        money_from_i128(written_off, "public debt principal written off")?,
    ))
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

    #[test]
    fn public_debt_interest_deepens_the_deficit_and_compounds_debt() {
        fn world_with_debt(debt: i64) -> World {
            let indicators = CountryIndicators::new(
                Money::from_minor_units(0),
                Money::from_minor_units(debt),
                BasisPoints::new(5_000).expect("valid legitimacy"),
                BasisPoints::new(5_000).expect("valid cohesion"),
            );
            let mut world = World::new(
                WorldSeed::new(47),
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

        let mut indebted = world_with_debt(1_000_000);
        let mut debt_free = world_with_debt(0);
        indebted.advance_one_year().expect("simulation succeeds");
        debt_free.advance_one_year().expect("simulation succeeds");

        let debt_of = |world: &World| {
            i128::from(
                world.countries()[&CountryId::new(1)]
                    .indicators()
                    .public_debt()
                    .minor_units(),
            )
        };
        // Same seed, same year, identical worlds except the seed principal:
        // the indebted world must end exactly principal + 300 bps interest deeper,
        // because the interest widens the deficit and close_budget capitalizes it.
        let interest = 1_000_000_i128 * DEBT_INTEREST_BPS / 10_000;
        assert_eq!(
            debt_of(&indebted) - debt_of(&debt_free),
            1_000_000 + interest
        );
    }

    #[test]
    fn debt_service_is_recorded_as_a_typed_domain_event() {
        let indicators = CountryIndicators::new(
            Money::from_minor_units(0),
            Money::from_minor_units(1_000_000),
            BasisPoints::new(5_000).expect("valid legitimacy"),
            BasisPoints::new(5_000).expect("valid cohesion"),
        );
        let mut world = World::new(
            WorldSeed::new(47),
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
        world.advance_one_year().expect("simulation succeeds");

        let charged = world
            .events()
            .events()
            .iter()
            .find_map(|envelope| match envelope.event() {
                DomainEvent::PublicDebtInterestCharged {
                    country,
                    opening_debt,
                    interest,
                } => Some((*country, *opening_debt, *interest)),
                _ => None,
            })
            .expect("debt service event");
        assert_eq!(charged.0, CountryId::new(1));
        assert_eq!(charged.1, Money::from_minor_units(1_000_000));
        assert_eq!(charged.2, Money::from_minor_units(30_000));

        let mut debt_free = economy(47, 5_000);
        debt_free.advance_one_year().expect("simulation succeeds");
        assert!(!debt_free.events().events().iter().any(|envelope| matches!(
            envelope.event(),
            DomainEvent::PublicDebtInterestCharged { .. }
        )));
    }

    #[test]
    fn public_debt_restrains_discretionary_spending() {
        let build = |debt: i64| {
            let indicators = CountryIndicators::new(
                Money::from_minor_units(0),
                Money::from_minor_units(debt),
                BasisPoints::new(5_000).expect("valid legitimacy"),
                BasisPoints::new(5_000).expect("valid cohesion"),
            );
            let mut world = World::new(
                WorldSeed::new(61),
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
        };
        let mut lean = build(0);
        let mut indebted = build(800_000_000_000);
        lean.advance_one_year().expect("simulation succeeds");
        indebted.advance_one_year().expect("simulation succeeds");
        let fiscal = |world: &World| {
            world
                .events()
                .events()
                .iter()
                .find_map(|envelope| match envelope.event() {
                    DomainEvent::CountryFiscalYearClosed {
                        revenue, spending, ..
                    } => Some((*revenue, *spending)),
                    _ => None,
                })
                .expect("fiscal closure event")
        };
        let (lean_revenue, lean_spending) = fiscal(&lean);
        let (indebted_revenue, indebted_spending) = fiscal(&indebted);
        assert_eq!(lean_revenue.minor_units(), indebted_revenue.minor_units());
        assert!(indebted_spending.minor_units() < lean_spending.minor_units());
    }

    #[test]
    fn unsustainable_public_debt_is_restructured_with_a_political_cost() {
        let build = |debt: i64| {
            let indicators = CountryIndicators::new(
                Money::default(),
                Money::from_minor_units(debt),
                BasisPoints::new(7_000).expect("valid legitimacy"),
                BasisPoints::new(7_000).expect("valid cohesion"),
            );
            let mut world = World::new(
                WorldSeed::new(47),
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
        };
        let mut crisis = build(15_000_000_000_000);
        let mut sustainable = build(100_000_000_000);

        crisis.advance_one_year().expect("crisis year closes");
        sustainable
            .advance_one_year()
            .expect("sustainable year closes");

        let restructuring = crisis
            .events()
            .events()
            .iter()
            .find_map(|envelope| match envelope.event() {
                DomainEvent::PublicDebtRestructured {
                    debt_before,
                    debt_after,
                    principal_written_off,
                    ..
                } => Some((*debt_before, *debt_after, *principal_written_off)),
                _ => None,
            })
            .expect("restructuring event");
        assert_eq!(
            restructuring.0.minor_units() - restructuring.2.minor_units(),
            restructuring.1.minor_units()
        );
        assert_eq!(
            i128::from(restructuring.2.minor_units()) * 10_000,
            i128::from(restructuring.0.minor_units()) * DEBT_RESTRUCTURING_HAIRCUT_BPS
        );
        assert_eq!(
            crisis.countries()[&CountryId::new(1)]
                .indicators()
                .public_debt(),
            restructuring.1
        );
        assert!(
            crisis.countries()[&CountryId::new(1)]
                .indicators()
                .legitimacy()
                < sustainable.countries()[&CountryId::new(1)]
                    .indicators()
                    .legitimacy()
        );
        assert!(
            !sustainable
                .events()
                .events()
                .iter()
                .any(|event| matches!(event.event(), DomainEvent::PublicDebtRestructured { .. }))
        );
    }
}
