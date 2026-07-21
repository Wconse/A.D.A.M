use crate::{ActorId, BasisPoints, FirmId, World, WorldError};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
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

impl World {
    /// Registers ownership after checking actor, firm, duplicate, and aggregate rights.
    /// # Errors
    /// Returns [`WorldError`] when references or ownership totals are invalid.
    pub fn register_ownership_stake(&mut self, stake: OwnershipStake) -> Result<(), WorldError> {
        if !self.firms.contains_key(&stake.firm()) {
            return Err(WorldError::UnknownFirm(stake.firm()));
        }
        if !self.actors().contains_key(&stake.owner()) {
            return Err(WorldError::UnknownActor(stake.owner()));
        }
        let key = (stake.firm(), stake.owner());
        if self.ownership_stakes.contains_key(&key) {
            return Err(WorldError::DuplicateOwnershipStake {
                firm: stake.firm(),
                owner: stake.owner(),
            });
        }
        let economic: u32 = self
            .ownership_stakes
            .values()
            .filter(|s| s.firm() == stake.firm())
            .map(|s| u32::from(s.economic_rights().get()))
            .sum::<u32>()
            + u32::from(stake.economic_rights().get());
        let voting: u32 = self
            .ownership_stakes
            .values()
            .filter(|s| s.firm() == stake.firm())
            .map(|s| u32::from(s.voting_rights().get()))
            .sum::<u32>()
            + u32::from(stake.voting_rights().get());
        if economic > 10_000 || voting > 10_000 {
            return Err(WorldError::OwnershipExceedsFull(stake.firm()));
        }
        self.ownership_stakes.insert(key, stake);
        Ok(())
    }
    #[must_use]
    pub fn ownership_stakes(&self) -> &BTreeMap<(FirmId, ActorId), OwnershipStake> {
        &self.ownership_stakes
    }
    #[must_use]
    pub fn firm_policies(&self) -> &BTreeMap<FirmId, FirmPolicy> {
        &self.firm_policies
    }
    #[must_use]
    pub fn can_control_firm(&self, actor: ActorId, firm: FirmId) -> bool {
        self.ownership_stakes
            .get(&(firm, actor))
            .is_some_and(|stake| stake.voting_rights().get() > 5_000)
    }
    /// Changes policy only for an actor with majority voting control.
    /// # Errors
    /// Returns [`WorldError::UnauthorizedFirmControl`] without changing state when authority is absent.
    pub fn set_firm_policy(
        &mut self,
        actor: ActorId,
        firm: FirmId,
        policy: FirmPolicy,
    ) -> Result<(), WorldError> {
        if !self.can_control_firm(actor, firm) {
            return Err(WorldError::UnauthorizedFirmControl { actor, firm });
        }
        self.firm_policies.insert(firm, policy);
        Ok(())
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
