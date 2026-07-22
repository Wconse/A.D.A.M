use crate::{FirmId, Money, World, WorldError};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmMonthlyAccounts {
    sales_revenue: Money,
    wages_owed: Money,
    wages_paid: Money,
    wage_arrears: Money,
}
impl FirmMonthlyAccounts {
    #[must_use]
    pub const fn sales_revenue(self) -> Money {
        self.sales_revenue
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmploymentAdjustmentProposal {
    pub firm: FirmId,
    pub cohort: crate::CohortId,
    pub current_workers: u64,
    pub affordable_workers: u64,
    pub wage_coverage_bps: u16,
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
    pub fn reset_monthly_firm_accounts(&mut self) {
        self.firm_monthly_accounts.clear();
    }
    /// Produces advisory staffing proposals from realized payroll coverage; it does not fire workers automatically.
    #[must_use]
    pub fn plan_cash_constrained_staffing(&self) -> Vec<EmploymentAdjustmentProposal> {
        let mut proposals = Vec::new();
        for ((firm, cohort), agreement) in &self.employment_agreements {
            if !agreement.active() {
                continue;
            }
            let Some(accounts) = self.firm_monthly_accounts.get(firm) else {
                continue;
            };
            let owed = accounts.wages_owed().minor_units();
            if owed <= 0 {
                continue;
            }
            let paid = accounts.wages_paid().minor_units().max(0);
            let coverage =
                u16::try_from((i128::from(paid) * 10_000 / i128::from(owed)).clamp(0, 10_000))
                    .unwrap_or(10_000);
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
                });
            }
        }
        proposals
    }
    #[must_use]
    pub fn firm_monthly_accounts(&self) -> &BTreeMap<FirmId, FirmMonthlyAccounts> {
        &self.firm_monthly_accounts
    }
}
