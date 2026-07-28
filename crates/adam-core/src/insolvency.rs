use std::collections::BTreeMap;

use crate::{
    ActorId, DomainEvent, FirmId, GoodId, Money, QuantityMilli, SimDate, World, WorldError,
};

/// Immutable asset-and-worker-claim snapshot taken when ordinary firm operations
/// enter insolvency administration.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FirmInsolvency {
    firm: FirmId,
    administrator: ActorId,
    declared_on: SimDate,
    cash_at_declaration: Money,
    wage_arrears: Money,
    inventories_at_declaration: BTreeMap<GoodId, QuantityMilli>,
}

impl FirmInsolvency {
    #[must_use]
    pub const fn firm(&self) -> FirmId {
        self.firm
    }
    #[must_use]
    pub const fn administrator(&self) -> ActorId {
        self.administrator
    }
    #[must_use]
    pub const fn declared_on(&self) -> SimDate {
        self.declared_on
    }
    #[must_use]
    pub const fn cash_at_declaration(&self) -> Money {
        self.cash_at_declaration
    }
    #[must_use]
    pub const fn wage_arrears(&self) -> Money {
        self.wage_arrears
    }
    #[must_use]
    pub fn inventories_at_declaration(&self) -> &BTreeMap<GoodId, QuantityMilli> {
        &self.inventories_at_declaration
    }
}

impl World {
    pub(crate) fn declare_firm_insolvent(
        &mut self,
        firm: FirmId,
        administrator: ActorId,
        wage_arrears: Money,
    ) -> Result<(), WorldError> {
        if self.firm_insolvencies.contains_key(&firm) {
            return Ok(());
        }
        let definition = self.firms.get(&firm).ok_or(WorldError::UnknownFirm(firm))?;
        let snapshot = FirmInsolvency {
            firm,
            administrator,
            declared_on: self.date,
            cash_at_declaration: definition.cash(),
            wage_arrears,
            inventories_at_declaration: definition.inventories().clone(),
        };
        let inventory = snapshot
            .inventories_at_declaration
            .iter()
            .map(|(good, quantity)| (*good, *quantity))
            .collect();
        self.firm_insolvencies.insert(firm, snapshot.clone());
        self.firm_distress_months.remove(&firm);
        self.events.append(
            self.date,
            DomainEvent::FirmInsolvencyDeclared {
                firm,
                administrator,
                cash: snapshot.cash_at_declaration,
                wage_arrears,
                inventory,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn is_firm_insolvent(&self, firm: FirmId) -> bool {
        self.firm_insolvencies.contains_key(&firm)
    }

    #[must_use]
    pub fn firm_insolvencies(&self) -> &BTreeMap<FirmId, FirmInsolvency> {
        &self.firm_insolvencies
    }
}
