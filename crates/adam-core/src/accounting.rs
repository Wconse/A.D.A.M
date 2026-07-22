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
    #[must_use]
    pub fn firm_monthly_accounts(&self) -> &BTreeMap<FirmId, FirmMonthlyAccounts> {
        &self.firm_monthly_accounts
    }
}
