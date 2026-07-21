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
    /// Returns a policy with a different marketing allocation.
    /// # Errors
    /// Returns [`WorldError::InvalidBusinessPolicy`] when total allocations exceed 100%.
    pub fn with_marketing_budget(self, value: BasisPoints) -> Result<Self, WorldError> {
        Self::new(
            self.inventory_buffer_days,
            self.price_markup,
            value,
            self.reinvestment,
            self.dividend,
        )
    }
    #[must_use]
    pub const fn with_inventory_buffer_days(mut self, value: u16) -> Self {
        self.inventory_buffer_days = value;
        self
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum CorporateRole {
    BoardDirector,
    ChiefExecutive,
    OperationsManager,
    MarketingManager,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CorporateAction {
    SetOverallPolicy,
    SetOperationsPolicy,
    SetMarketingPolicy,
    ProposeMajorInvestment,
    ApproveMajorInvestment,
    DeclareDividend,
    AppointExecutive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmAppointment {
    firm: FirmId,
    actor: ActorId,
    role: CorporateRole,
}
impl FirmAppointment {
    #[must_use]
    pub const fn new(firm: FirmId, actor: ActorId, role: CorporateRole) -> Self {
        Self { firm, actor, role }
    }
    #[must_use]
    pub const fn firm(self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn actor(self) -> ActorId {
        self.actor
    }
    #[must_use]
    pub const fn role(self) -> CorporateRole {
        self.role
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
    /// Registers a corporate appointment.
    /// # Errors
    /// Returns [`WorldError`] for unknown references or a duplicate actor/role appointment.
    pub fn register_firm_appointment(
        &mut self,
        appointment: FirmAppointment,
    ) -> Result<(), WorldError> {
        if !self.firms.contains_key(&appointment.firm()) {
            return Err(WorldError::UnknownFirm(appointment.firm()));
        }
        if !self.actors().contains_key(&appointment.actor()) {
            return Err(WorldError::UnknownActor(appointment.actor()));
        }
        let key = (appointment.firm(), appointment.actor(), appointment.role());
        if self.firm_appointments.contains_key(&key) {
            return Err(WorldError::DuplicateFirmAppointment);
        }
        self.firm_appointments.insert(key, appointment);
        Ok(())
    }
    #[must_use]
    pub fn firm_appointments(
        &self,
    ) -> &BTreeMap<(FirmId, ActorId, CorporateRole), FirmAppointment> {
        &self.firm_appointments
    }
    #[must_use]
    pub fn can_perform_corporate_action(
        &self,
        actor: ActorId,
        firm: FirmId,
        action: CorporateAction,
    ) -> bool {
        let majority = self
            .ownership_stakes
            .get(&(firm, actor))
            .is_some_and(|stake| stake.voting_rights().get() > 5_000);
        let has = |role| self.firm_appointments.contains_key(&(firm, actor, role));
        majority
            || match action {
                CorporateAction::SetOverallPolicy | CorporateAction::ProposeMajorInvestment => {
                    has(CorporateRole::ChiefExecutive)
                }
                CorporateAction::SetOperationsPolicy => {
                    has(CorporateRole::ChiefExecutive) || has(CorporateRole::OperationsManager)
                }
                CorporateAction::SetMarketingPolicy => {
                    has(CorporateRole::ChiefExecutive) || has(CorporateRole::MarketingManager)
                }
                CorporateAction::ApproveMajorInvestment
                | CorporateAction::DeclareDividend
                | CorporateAction::AppointExecutive => has(CorporateRole::BoardDirector),
            }
    }
    #[must_use]
    pub fn can_control_firm(&self, actor: ActorId, firm: FirmId) -> bool {
        self.can_perform_corporate_action(actor, firm, CorporateAction::SetOverallPolicy)
    }
    /// Changes only marketing allocation under marketing-scoped authority.
    /// # Errors
    /// Returns an authority, missing-policy, or allocation error without mutation.
    pub fn set_marketing_budget(
        &mut self,
        actor: ActorId,
        firm: FirmId,
        value: BasisPoints,
    ) -> Result<(), WorldError> {
        if !self.can_perform_corporate_action(actor, firm, CorporateAction::SetMarketingPolicy) {
            return Err(WorldError::UnauthorizedFirmControl { actor, firm });
        }
        let current = *self
            .firm_policies
            .get(&firm)
            .ok_or(WorldError::MissingFirmPolicy(firm))?;
        let updated = current.with_marketing_budget(value)?;
        self.firm_policies.insert(firm, updated);
        Ok(())
    }
    /// Changes inventory buffer under operations-scoped authority.
    /// # Errors
    /// Returns an authority or missing-policy error without mutation.
    pub fn set_inventory_buffer(
        &mut self,
        actor: ActorId,
        firm: FirmId,
        days: u16,
    ) -> Result<(), WorldError> {
        if !self.can_perform_corporate_action(actor, firm, CorporateAction::SetOperationsPolicy) {
            return Err(WorldError::UnauthorizedFirmControl { actor, firm });
        }
        let current = *self
            .firm_policies
            .get(&firm)
            .ok_or(WorldError::MissingFirmPolicy(firm))?;
        self.firm_policies
            .insert(firm, current.with_inventory_buffer_days(days));
        Ok(())
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
    fn managed_world(voting: u16) -> (World, FirmPolicy) {
        use crate::{
            Actor, Country, CountryId, Firm, Good, GoodId, Money, Population, ProductionRecipe,
            QuantityMilli, RecipeId, Region, RegionId, SimDate, WorldSeed,
        };
        let mut world = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
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
            .register_actor(
                Actor::new(ActorId::new(1), "Owner", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_good(Good::new(GoodId::new(1), "Service").expect("good"))
            .expect("good");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Recipe",
                    GoodId::new(1),
                    QuantityMilli::new(1000),
                    1000,
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
                    1,
                    1,
                    Money::from_minor_units(1),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .register_ownership_stake(OwnershipStake::new(
                FirmId::new(1),
                ActorId::new(1),
                BasisPoints::new(voting).expect("rights"),
                BasisPoints::new(voting).expect("rights"),
            ))
            .expect("stake");
        let policy = FirmPolicy::new(
            30,
            BasisPoints::new(1000).expect("rate"),
            BasisPoints::new(1000).expect("rate"),
            BasisPoints::new(4000).expect("rate"),
            BasisPoints::new(3000).expect("rate"),
        )
        .expect("policy");
        (world, policy)
    }
    #[test]
    fn strict_majority_can_apply_replayable_policy_command() {
        use crate::WorldCommand;
        let (mut world, policy) = managed_world(6000);
        WorldCommand::SetFirmPolicy {
            actor: ActorId::new(1),
            firm: FirmId::new(1),
            policy,
        }
        .apply(&mut world)
        .expect("authorized");
        assert_eq!(world.firm_policies()[&FirmId::new(1)], policy);
    }
    #[test]
    fn half_ownership_cannot_unilaterally_change_policy() {
        use crate::WorldCommand;
        let (mut world, policy) = managed_world(5000);
        let before = world.stable_fingerprint();
        assert!(matches!(
            WorldCommand::SetFirmPolicy {
                actor: ActorId::new(1),
                firm: FirmId::new(1),
                policy
            }
            .apply(&mut world),
            Err(WorldError::UnauthorizedFirmControl { .. })
        ));
        assert_eq!(world.stable_fingerprint(), before);
    }
}
