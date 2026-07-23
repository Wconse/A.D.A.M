use crate::{CohortId, DomainEvent, Money, World, WorldError};

const SURVIVAL_DEBT_CEILING_ANNUAL_INCOME_MULTIPLIER: i64 = 2;

/// One household cohort's debt-funded attempt to preserve monthly survival consumption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HouseholdSurvivalBorrowing {
    pub cohort: CohortId,
    pub amount: Money,
    pub ending_wealth: Money,
    pub ending_debt: Money,
}

impl World {
    /// Borrows against aggregate annual income when current liquid wealth cannot fund survival needs.
    ///
    /// Debt is limited to twice annual income. The borrowed cash enters the ordinary household budget,
    /// is serviced by the existing debt cashflow, and does not guarantee that physical supply exists.
    ///
    /// # Errors
    /// Returns an error on duplicate monthly execution or arithmetic overflow without partially applying
    /// later cohorts.
    pub fn execute_monthly_household_coping(
        &mut self,
    ) -> Result<Vec<HouseholdSurvivalBorrowing>, WorldError> {
        if self.last_household_coping_date == Some(self.date) {
            return Err(WorldError::MonthlyStageAlreadyExecuted {
                stage: "household survival coping",
                date: self.date,
            });
        }
        let mut cohorts = self.cohorts.clone();
        let mut rows = Vec::new();
        for cohort in cohorts.values_mut() {
            let required = self.monthly_survival_cost(cohort.id())?.minor_units();
            let shortfall = required
                .checked_sub(cohort.liquid_wealth().minor_units())
                .unwrap_or(0)
                .max(0);
            let ceiling = cohort
                .annual_income()
                .minor_units()
                .checked_mul(SURVIVAL_DEBT_CEILING_ANNUAL_INCOME_MULTIPLIER)
                .ok_or(WorldError::ArithmeticOverflow("survival debt ceiling"))?;
            let headroom = ceiling
                .checked_sub(cohort.debt().minor_units())
                .unwrap_or(0)
                .max(0);
            let amount = shortfall.min(headroom);
            if amount == 0 {
                continue;
            }
            let amount = Money::from_minor_units(amount);
            cohort.borrow_for_survival(amount)?;
            rows.push(HouseholdSurvivalBorrowing {
                cohort: cohort.id(),
                amount,
                ending_wealth: cohort.liquid_wealth(),
                ending_debt: cohort.debt(),
            });
        }
        self.cohorts = cohorts;
        self.last_household_coping_date = Some(self.date);
        for row in &rows {
            self.events.append(
                self.date,
                DomainEvent::HouseholdSurvivalBorrowed {
                    cohort: row.cohort,
                    amount: row.amount,
                    ending_wealth: row.ending_wealth,
                    ending_debt: row.ending_debt,
                },
            );
        }
        Ok(rows)
    }
}
