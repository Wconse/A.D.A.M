use crate::{ActorId, BasisPoints, FirmId, WorldError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnershipStake {
    firm: FirmId,
    owner: ActorId,
    economic_rights: BasisPoints,
    voting_rights: BasisPoints,
}
impl OwnershipStake {
    #[must_use]
    pub const fn new(
        firm: FirmId,
        owner: ActorId,
        economic_rights: BasisPoints,
        voting_rights: BasisPoints,
    ) -> Self {
        Self {
            firm,
            owner,
            economic_rights,
            voting_rights,
        }
    }
    #[must_use]
    pub const fn firm(self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn owner(self) -> ActorId {
        self.owner
    }
    #[must_use]
    pub const fn economic_rights(self) -> BasisPoints {
        self.economic_rights
    }
    #[must_use]
    pub const fn voting_rights(self) -> BasisPoints {
        self.voting_rights
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirmPolicy {
    inventory_buffer_days: u16,
    price_markup: BasisPoints,
    marketing_budget: BasisPoints,
    reinvestment: BasisPoints,
    dividend: BasisPoints,
}
impl FirmPolicy {
    /// Creates a management policy without implying that an owner has authority to enact it.
    /// # Errors
    /// Returns [`WorldError::InvalidBusinessPolicy`] when profit allocations exceed 100%.
    pub fn new(
        inventory_buffer_days: u16,
        price_markup: BasisPoints,
        marketing_budget: BasisPoints,
        reinvestment: BasisPoints,
        dividend: BasisPoints,
    ) -> Result<Self, WorldError> {
        let allocation = u32::from(marketing_budget.get())
            + u32::from(reinvestment.get())
            + u32::from(dividend.get());
        if allocation > 10_000 {
            return Err(WorldError::InvalidBusinessPolicy(
                "marketing, reinvestment, and dividend allocations exceed 100%",
            ));
        }
        Ok(Self {
            inventory_buffer_days,
            price_markup,
            marketing_budget,
            reinvestment,
            dividend,
        })
    }
    #[must_use]
    pub const fn inventory_buffer_days(self) -> u16 {
        self.inventory_buffer_days
    }
    #[must_use]
    pub const fn price_markup(self) -> BasisPoints {
        self.price_markup
    }
    #[must_use]
    pub const fn marketing_budget(self) -> BasisPoints {
        self.marketing_budget
    }
    #[must_use]
    pub const fn reinvestment(self) -> BasisPoints {
        self.reinvestment
    }
    #[must_use]
    pub const fn dividend(self) -> BasisPoints {
        self.dividend
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_impossible_profit_allocation() {
        let result = FirmPolicy::new(
            30,
            BasisPoints::new(1000).expect("rate"),
            BasisPoints::new(4000).expect("rate"),
            BasisPoints::new(4000).expect("rate"),
            BasisPoints::new(3000).expect("rate"),
        );
        assert!(matches!(result, Err(WorldError::InvalidBusinessPolicy(_))));
    }
}
