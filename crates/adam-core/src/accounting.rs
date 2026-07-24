use crate::{FirmId, Money, World, WorldError};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmMonthlyAccounts {
    sales_revenue: Money,
    final_sales_revenue: Money,
    wages_owed: Money,
    wages_paid: Money,
    wage_arrears: Money,
    produced_batches: u64,
}
impl FirmMonthlyAccounts {
    #[must_use]
    pub const fn sales_revenue(self) -> Money {
        self.sales_revenue
    }
    #[must_use]
    pub const fn final_sales_revenue(self) -> Money {
        self.final_sales_revenue
    }
    #[must_use]
    pub const fn wages_owed(self) -> Money {
        self.wages_owed
    }
    #[must_use]
    pub const fn wages_paid(self) -> Money {
        self.wages_paid
    }
    #[must_use]
    pub const fn wage_arrears(self) -> Money {
        self.wage_arrears
    }
    #[must_use]
    pub const fn produced_batches(self) -> u64 {
        self.produced_batches
    }
    fn add(field: &mut Money, value: Money, label: &'static str) -> Result<(), WorldError> {
        *field = Money::from_minor_units(
            field
                .minor_units()
                .checked_add(value.minor_units())
                .ok_or(WorldError::ArithmeticOverflow(label))?,
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FirmExpectationSource {
    Management,
    ObservedOperations,
    ObservedHistory,
}
impl FirmExpectationSource {
    pub(crate) const fn fingerprint_tag(self) -> u8 {
        match self {
            Self::Management => 1,
            Self::ObservedOperations => 2,
            Self::ObservedHistory => 3,
        }
    }
}

/// A manager's explicit forecast of firm cash flows over a fixed horizon.
/// Forecasts inform decisions but never mutate cash, inventories, or employment directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmExpectations {
    expected_sales_revenue: Money,
    expected_input_costs: Money,
    expected_financing: Money,
    horizon_months: u16,
    source: FirmExpectationSource,
}
impl FirmExpectations {
    /// Creates a non-negative cash-flow forecast for one or more future months.
    /// # Errors
    /// Returns [`WorldError::InvalidFirmExpectations`] for negative flows or a zero horizon.
    pub fn new(
        expected_sales_revenue: Money,
        expected_input_costs: Money,
        expected_financing: Money,
        horizon_months: u16,
        source: FirmExpectationSource,
    ) -> Result<Self, WorldError> {
        let value = Self {
            expected_sales_revenue,
            expected_input_costs,
            expected_financing,
            horizon_months,
            source,
        };
        value.validate()?;
        Ok(value)
    }
    pub(crate) fn validate(self) -> Result<(), WorldError> {
        if self.horizon_months == 0 {
            return Err(WorldError::InvalidFirmExpectations(
                "forecast horizon must be positive",
            ));
        }
        if self.expected_sales_revenue.minor_units() < 0
            || self.expected_input_costs.minor_units() < 0
            || self.expected_financing.minor_units() < 0
        {
            return Err(WorldError::InvalidFirmExpectations(
                "forecast cash flows must be non-negative",
            ));
        }
        Ok(())
    }
    #[must_use]
    pub const fn expected_sales_revenue(self) -> Money {
        self.expected_sales_revenue
    }
    #[must_use]
    pub const fn expected_input_costs(self) -> Money {
        self.expected_input_costs
    }
    #[must_use]
    pub const fn expected_financing(self) -> Money {
        self.expected_financing
    }
    #[must_use]
    pub const fn horizon_months(self) -> u16 {
        self.horizon_months
    }
    #[must_use]
    pub const fn source(self) -> FirmExpectationSource {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmploymentAdjustmentProposal {
    pub firm: FirmId,
    pub cohort: crate::CohortId,
    pub current_workers: u64,
    pub affordable_workers: u64,
    /// Coverage used for this proposal: forecast coverage when expectations exist,
    /// otherwise realized payroll coverage.
    pub wage_coverage_bps: u16,
    pub realized_wage_coverage_bps: Option<u16>,
    pub forecast_wage_coverage_bps: Option<u16>,
}

fn coverage_bps(available: i128, obligation: i128) -> u16 {
    if obligation <= 0 {
        return 10_000;
    }
    u16::try_from((available.max(0).saturating_mul(10_000) / obligation).clamp(0, 10_000))
        .unwrap_or(10_000)
}

impl World {
    pub(crate) fn record_firm_sale(
        &mut self,
        firm: FirmId,
        revenue: Money,
    ) -> Result<(), WorldError> {
        FirmMonthlyAccounts::add(
            &mut self
                .firm_monthly_accounts
                .entry(firm)
                .or_default()
                .sales_revenue,
            revenue,
            "firm monthly sales",
        )
    }
    /// Records a final (household) sale: counted in total revenue and in the taxable base.
    pub(crate) fn record_firm_final_sale(
        &mut self,
        firm: FirmId,
        revenue: Money,
    ) -> Result<(), WorldError> {
        self.record_firm_sale(firm, revenue)?;
        FirmMonthlyAccounts::add(
            &mut self
                .firm_monthly_accounts
                .entry(firm)
                .or_default()
                .final_sales_revenue,
            revenue,
            "firm monthly final sales",
        )
    }
    pub(crate) fn record_firm_payroll(
        &mut self,
        firm: FirmId,
        owed: Money,
        paid: Money,
        arrears: Money,
    ) -> Result<(), WorldError> {
        let row = self.firm_monthly_accounts.entry(firm).or_default();
        FirmMonthlyAccounts::add(&mut row.wages_owed, owed, "firm wages owed")?;
        FirmMonthlyAccounts::add(&mut row.wages_paid, paid, "firm wages paid")?;
        row.wage_arrears = arrears;
        Ok(())
    }
    pub(crate) fn record_firm_production(
        &mut self,
        firm: FirmId,
        batches: u64,
    ) -> Result<(), WorldError> {
        let row = self.firm_monthly_accounts.entry(firm).or_default();
        row.produced_batches = row
            .produced_batches
            .checked_add(batches)
            .ok_or(WorldError::ArithmeticOverflow("firm produced batches"))?;
        Ok(())
    }
    pub fn reset_monthly_firm_accounts(&mut self) {
        self.firm_monthly_accounts.clear();
        self.monthly_firm_market_outcomes.clear();
        self.monthly_firm_procurement_purchases.clear();
    }
    /// Replaces the firm's explicit management forecast without changing physical or monetary state.
    /// # Errors
    /// Returns an error for an unknown firm or invalid forecast values.
    pub fn update_firm_expectations(
        &mut self,
        firm: FirmId,
        expectations: FirmExpectations,
    ) -> Result<(), WorldError> {
        if !self.firms.contains_key(&firm) {
            return Err(WorldError::UnknownFirm(firm));
        }
        expectations.validate()?;
        self.firm_expectations.insert(firm, expectations);
        self.events.append(
            self.date,
            crate::DomainEvent::FirmExpectationsUpdated {
                firm,
                expected_sales_revenue: expectations.expected_sales_revenue(),
                expected_input_costs: expectations.expected_input_costs(),
                expected_financing: expectations.expected_financing(),
                horizon_months: expectations.horizon_months(),
                source: expectations.source(),
            },
        );
        Ok(())
    }
    /// Derives a forecast from realized sales, current input prices, and the current physical production plan.
    /// No financing is assumed until concrete financing offers exist in the world model.
    /// # Errors
    /// Returns an error for an unknown firm, invalid horizon, missing input price, or arithmetic overflow.
    pub fn derive_firm_expectations_from_observations(
        &mut self,
        firm: FirmId,
        horizon_months: u16,
    ) -> Result<FirmExpectations, WorldError> {
        if horizon_months == 0 {
            return Err(WorldError::InvalidFirmExpectations(
                "forecast horizon must be positive",
            ));
        }
        let baseline = self.observed_operating_baseline(firm)?;
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let region = definition.region();
        let recipe = self
            .production_recipes
            .get(&definition.recipe())
            .ok_or(WorldError::UnknownRecipe(definition.recipe()))?;
        let (monthly_sales, planned_batches, source) = if let Some(observed) = &baseline {
            (
                observed.monthly_sales.minor_units(),
                observed.produced_batches,
                FirmExpectationSource::ObservedHistory,
            )
        } else {
            let planned_batches = self
                .plan_monthly_production()?
                .into_iter()
                .find(|plan| plan.firm() == firm)
                .ok_or(WorldError::UnknownFirm(firm))?
                .batches();
            let monthly_sales = self
                .firm_monthly_accounts
                .get(&firm)
                .map_or(0_i64, |accounts| accounts.sales_revenue().minor_units());
            (
                monthly_sales,
                planned_batches,
                FirmExpectationSource::ObservedOperations,
            )
        };
        let expected_sales = i128::from(monthly_sales)
            .checked_mul(i128::from(horizon_months))
            .ok_or(WorldError::ArithmeticOverflow("expected sales revenue"))?;
        let mut expected_inputs = 0_i128;
        for input in recipe.inputs() {
            let price = if let Some(observed) = &baseline {
                observed
                    .input_prices
                    .get(&input.good())
                    .ok_or(WorldError::MissingRegionalPrice {
                        region,
                        good: input.good(),
                    })?
                    .minor_units()
            } else {
                self.regional_prices
                    .get(&(region, input.good()))
                    .ok_or(WorldError::MissingRegionalPrice {
                        region,
                        good: input.good(),
                    })?
                    .minor_units()
            };
            let quantity = i128::from(input.quantity_per_batch().get())
                .checked_mul(i128::from(planned_batches))
                .and_then(|value| value.checked_mul(i128::from(horizon_months)))
                .ok_or(WorldError::ArithmeticOverflow("expected input quantity"))?;
            let cost = i128::from(price)
                .checked_mul(quantity)
                .ok_or(WorldError::ArithmeticOverflow("expected input cost"))?
                / i128::from(crate::QuantityMilli::SCALE);
            expected_inputs = expected_inputs
                .checked_add(cost)
                .ok_or(WorldError::ArithmeticOverflow("expected input costs"))?;
        }
        let expectations = FirmExpectations::new(
            Money::from_minor_units(
                i64::try_from(expected_sales)
                    .map_err(|_| WorldError::ArithmeticOverflow("expected sales revenue"))?,
            ),
            Money::from_minor_units(
                i64::try_from(expected_inputs)
                    .map_err(|_| WorldError::ArithmeticOverflow("expected input costs"))?,
            ),
            Money::default(),
            horizon_months,
            source,
        )?;
        self.update_firm_expectations(firm, expectations)?;
        Ok(expectations)
    }
    /// Produces advisory staffing proposals from realized payroll and, when present,
    /// management forecasts. It never fires workers or moves money automatically.
    #[must_use]
    pub fn plan_cash_constrained_staffing(&self) -> Vec<EmploymentAdjustmentProposal> {
        let mut proposals = Vec::new();
        for ((firm, cohort), agreement) in &self.employment_agreements {
            if !agreement.active() {
                continue;
            }
            let realized = self.firm_monthly_accounts.get(firm).and_then(|accounts| {
                let owed = i128::from(accounts.wages_owed().minor_units());
                (owed > 0)
                    .then(|| coverage_bps(i128::from(accounts.wages_paid().minor_units()), owed))
            });
            let forecast = self.firm_expectations.get(firm).map(|expectations| {
                let monthly_wages = self
                    .employment_agreements
                    .values()
                    .filter(|row| row.active() && row.firm() == *firm)
                    .fold(0_i128, |total, row| {
                        total.saturating_add(
                            i128::from(row.wage().minor_units())
                                .saturating_mul(i128::from(row.workers())),
                        )
                    });
                let arrears = self
                    .employment_agreements
                    .values()
                    .filter(|row| row.firm() == *firm)
                    .fold(0_i128, |total, row| {
                        total.saturating_add(i128::from(row.arrears().minor_units()))
                    });
                let obligation = monthly_wages
                    .saturating_mul(i128::from(expectations.horizon_months()))
                    .saturating_add(arrears);
                let firm_cash = self
                    .firms
                    .get(firm)
                    .map_or(0, |row| i128::from(row.cash().minor_units()));
                let available = firm_cash
                    .saturating_add(i128::from(
                        expectations.expected_sales_revenue().minor_units(),
                    ))
                    .saturating_add(i128::from(expectations.expected_financing().minor_units()))
                    .saturating_sub(i128::from(
                        expectations.expected_input_costs().minor_units(),
                    ));
                coverage_bps(available, obligation)
            });
            let Some(coverage) = forecast.or(realized) else {
                continue;
            };
            if coverage < 10_000 {
                let affordable =
                    u64::try_from(i128::from(agreement.workers()) * i128::from(coverage) / 10_000)
                        .unwrap_or(0);
                proposals.push(EmploymentAdjustmentProposal {
                    firm: *firm,
                    cohort: *cohort,
                    current_workers: agreement.workers(),
                    affordable_workers: affordable,
                    wage_coverage_bps: coverage,
                    realized_wage_coverage_bps: realized,
                    forecast_wage_coverage_bps: forecast,
                });
            }
        }
        proposals
    }
    #[must_use]
    pub fn firm_monthly_accounts(&self) -> &BTreeMap<FirmId, FirmMonthlyAccounts> {
        &self.firm_monthly_accounts
    }
    #[must_use]
    pub fn firm_expectations(&self) -> &BTreeMap<FirmId, FirmExpectations> {
        &self.firm_expectations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, ConsumptionProfile, ConsumptionTarget, Country, CountryId, DemandBasis,
        EducationLevel, EmploymentAgreement, EmploymentStatus, Firm, Good, GoodId, HouseholdCohort,
        HouseholdType, NeedProfileId, NeedTier, Population, ProductionInput, ProductionRecipe,
        QuantityMilli, RecipeId, Region, RegionId, SimDate, WorldCommand, WorldSeed,
    };

    #[allow(clippy::too_many_lines)]
    fn staffed_world(firm_cash: i64) -> World {
        let mut world = World::new(WorldSeed::new(7), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country registration");
        world
            .register_good(Good::new(GoodId::new(1), "Food").expect("good"))
            .expect("good registration");
        world
            .register_good(Good::new(GoodId::new(2), "Energy").expect("good"))
            .expect("good registration");
        world
            .register_consumption_profile(
                ConsumptionProfile::new(
                    NeedProfileId::new(1),
                    "Workers",
                    vec![ConsumptionTarget::new(
                        GoodId::new(1),
                        NeedTier::Survival,
                        DemandBasis::PerPerson,
                        QuantityMilli::new(1_000),
                    )],
                )
                .expect("profile"),
            )
            .expect("profile registration");
        world
            .register_region(
                Region::new(
                    RegionId::new(1),
                    CountryId::new(1),
                    "R",
                    Population::new(100),
                    Money::from_minor_units(1_000_000),
                )
                .expect("region"),
            )
            .expect("region registration");
        world
            .set_regional_price(RegionId::new(1), GoodId::new(2), Money::from_minor_units(3))
            .expect("input price");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Food",
                    GoodId::new(1),
                    QuantityMilli::new(1_000),
                    1_000,
                    vec![ProductionInput::new(
                        GoodId::new(2),
                        QuantityMilli::new(2_000),
                    )],
                )
                .expect("recipe"),
            )
            .expect("recipe registration");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(1),
                    "Factory",
                    RegionId::new(1),
                    RecipeId::new(1),
                    100,
                    100,
                    Money::from_minor_units(firm_cash),
                    BTreeMap::from([(GoodId::new(2), QuantityMilli::new(100_000))]),
                )
                .expect("firm"),
            )
            .expect("firm registration");
        world
            .register_household_cohort(
                HouseholdCohort::new(
                    crate::CohortId::new(1),
                    RegionId::new(1),
                    NeedProfileId::new(1),
                    Population::new(100),
                    50,
                    AgeBand::Adult,
                    HouseholdType::WorkingAge,
                    EducationLevel::Secondary,
                    EmploymentStatus::Employed,
                    Money::default(),
                    Money::default(),
                    Money::default(),
                )
                .expect("cohort"),
            )
            .expect("cohort registration");
        world
            .register_employment_agreement(
                EmploymentAgreement::new(
                    FirmId::new(1),
                    crate::CohortId::new(1),
                    100,
                    Money::from_minor_units(100),
                )
                .expect("agreement"),
            )
            .expect("agreement registration");
        world
    }

    fn expectations(sales: i64, inputs: i64, financing: i64) -> FirmExpectations {
        FirmExpectations::new(
            Money::from_minor_units(sales),
            Money::from_minor_units(inputs),
            Money::from_minor_units(financing),
            1,
            FirmExpectationSource::Management,
        )
        .expect("expectations")
    }

    #[test]
    fn recovery_forecast_can_prevent_a_mechanical_layoff_proposal() {
        let mut world = staffed_world(6_000);
        world.execute_monthly_payroll().expect("payroll");
        let realized = world.plan_cash_constrained_staffing();
        assert_eq!(realized[0].wage_coverage_bps, 6_000);
        assert_eq!(realized[0].affordable_workers, 60);

        world
            .update_firm_expectations(FirmId::new(1), expectations(14_000, 0, 0))
            .expect("forecast");
        assert!(world.plan_cash_constrained_staffing().is_empty());
    }

    #[test]
    fn deterioration_forecast_can_warn_before_payroll_fails() {
        let mut world = staffed_world(10_000);
        world.execute_monthly_payroll().expect("payroll");
        assert!(world.plan_cash_constrained_staffing().is_empty());

        world
            .update_firm_expectations(FirmId::new(1), expectations(5_000, 0, 0))
            .expect("forecast");
        let proposals = world.plan_cash_constrained_staffing();
        assert_eq!(proposals.len(), 1);
        assert_eq!(proposals[0].realized_wage_coverage_bps, Some(10_000));
        assert_eq!(proposals[0].forecast_wage_coverage_bps, Some(5_000));
        assert_eq!(proposals[0].affordable_workers, 50);
    }

    #[test]
    fn expectation_update_is_replayable_and_does_not_move_resources() {
        let direct = staffed_world(20_000);
        let mut replayed = direct.clone();
        let cash_before = direct.firms()[&FirmId::new(1)].cash();
        let fingerprint_before = direct.stable_fingerprint();
        let workers_before =
            direct.employment_agreements()[&(FirmId::new(1), crate::CohortId::new(1))].workers();
        let command = WorldCommand::UpdateFirmExpectations {
            firm: FirmId::new(1),
            expectations: expectations(12_000, 3_000, 2_000),
        };
        let mut direct = direct;
        direct
            .update_firm_expectations(FirmId::new(1), expectations(12_000, 3_000, 2_000))
            .expect("direct update");
        command.apply(&mut replayed).expect("replay update");

        assert_eq!(direct, replayed);
        assert_ne!(direct.stable_fingerprint(), fingerprint_before);
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        assert!(matches!(
            direct.events().events().last().map(crate::EventEnvelope::event),
            Some(crate::DomainEvent::FirmExpectationsUpdated { firm, .. })
                if *firm == FirmId::new(1)
        ));
        assert_eq!(direct.firms()[&FirmId::new(1)].cash(), cash_before);
        assert_eq!(
            direct.employment_agreements()[&(FirmId::new(1), crate::CohortId::new(1))].workers(),
            workers_before
        );
    }

    #[test]
    fn observed_history_derives_replayable_average_sales_and_input_costs() {
        let mut direct = staffed_world(20_000);
        direct
            .record_firm_sale(FirmId::new(1), Money::from_minor_units(1_000))
            .expect("sale observation");
        direct
            .record_firm_production(FirmId::new(1), 40)
            .expect("production observation");
        direct
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("first observation");
        direct.reset_monthly_firm_accounts();
        direct
            .set_regional_price(RegionId::new(1), GoodId::new(2), Money::from_minor_units(5))
            .expect("second input price");
        direct
            .record_firm_sale(FirmId::new(1), Money::from_minor_units(3_000))
            .expect("sale observation");
        direct
            .record_firm_production(FirmId::new(1), 20)
            .expect("production observation");
        direct
            .capture_monthly_firm_observation(FirmId::new(1))
            .expect("second observation");
        let mut replayed = direct.clone();
        let cash_before = direct.firms()[&FirmId::new(1)].cash();
        let inventory_before = direct.firms()[&FirmId::new(1)].inventories().clone();

        let derived = direct
            .derive_firm_expectations_from_observations(FirmId::new(1), 2)
            .expect("derived expectations");
        WorldCommand::DeriveFirmExpectationsFromObservations {
            firm: FirmId::new(1),
            horizon_months: 2,
        }
        .apply(&mut replayed)
        .expect("replayed derivation");

        assert_eq!(derived.expected_sales_revenue().minor_units(), 4_000);
        assert_eq!(derived.expected_input_costs().minor_units(), 480);
        assert_eq!(derived.expected_financing(), Money::default());
        assert_eq!(derived.source(), FirmExpectationSource::ObservedHistory);
        assert_eq!(direct, replayed);
        assert_eq!(direct.firms()[&FirmId::new(1)].cash(), cash_before);
        assert_eq!(
            direct.firms()[&FirmId::new(1)].inventories(),
            &inventory_before
        );
    }

    #[test]
    fn invalid_expectation_values_are_rejected() {
        assert!(matches!(
            FirmExpectations::new(
                Money::from_minor_units(-1),
                Money::default(),
                Money::default(),
                1,
                FirmExpectationSource::Management,
            ),
            Err(WorldError::InvalidFirmExpectations(_))
        ));
        assert!(matches!(
            FirmExpectations::new(
                Money::default(),
                Money::default(),
                Money::default(),
                0,
                FirmExpectationSource::Management,
            ),
            Err(WorldError::InvalidFirmExpectations(_))
        ));
    }
}
