use crate::{
    BasisPoints, ContractId, DomainEvent, FirmId, Money, QuantityMilli, RouteId, World, WorldError,
};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContractStatus {
    Proposed,
    Active,
    Expired,
    Terminated,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FreightContract {
    id: ContractId,
    shipper: FirmId,
    carrier: FirmId,
    route: RouteId,
    reserved_capacity: QuantityMilli,
    discount: BasisPoints,
    start_month: u32,
    end_month: u32,
    status: ContractStatus,
}
impl FreightContract {
    /// Creates a long-term freight capacity agreement.
    /// # Errors
    /// Returns [`WorldError::InvalidFreightContract`] for self-dealing, empty capacity, or invalid term.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ContractId,
        shipper: FirmId,
        carrier: FirmId,
        route: RouteId,
        reserved_capacity: QuantityMilli,
        discount: BasisPoints,
        start_month: u32,
        end_month: u32,
    ) -> Result<Self, WorldError> {
        if shipper == carrier || reserved_capacity.get() == 0 || end_month <= start_month {
            return Err(WorldError::InvalidFreightContract(
                "invalid parties, capacity, or term",
            ));
        }
        Ok(Self {
            id,
            shipper,
            carrier,
            route,
            reserved_capacity,
            discount,
            start_month,
            end_month,
            status: ContractStatus::Proposed,
        })
    }
    #[must_use]
    pub const fn id(&self) -> ContractId {
        self.id
    }
    #[must_use]
    pub const fn shipper(&self) -> FirmId {
        self.shipper
    }
    #[must_use]
    pub const fn carrier(&self) -> FirmId {
        self.carrier
    }
    #[must_use]
    pub const fn route(&self) -> RouteId {
        self.route
    }
    #[must_use]
    pub const fn reserved_capacity(&self) -> QuantityMilli {
        self.reserved_capacity
    }
    #[must_use]
    pub const fn discount(&self) -> BasisPoints {
        self.discount
    }
    #[must_use]
    pub const fn status(&self) -> ContractStatus {
        self.status
    }
    #[must_use]
    pub const fn start_month(&self) -> u32 {
        self.start_month
    }
    #[must_use]
    pub const fn end_month(&self) -> u32 {
        self.end_month
    }
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
    }
    pub fn expire_if_due(&mut self, month: u32) -> bool {
        if self.status == ContractStatus::Active && month >= self.end_month {
            self.status = ContractStatus::Expired;
            true
        } else {
            false
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteOperatingCost {
    fuel_per_unit: Money,
    labor_per_unit: Money,
    maintenance_per_unit: Money,
}
impl RouteOperatingCost {
    /// Creates positive or zero cost components.
    /// # Errors
    /// Returns [`WorldError::InvalidFreightContract`] for negative costs.
    pub fn new(fuel: Money, labor: Money, maintenance: Money) -> Result<Self, WorldError> {
        if [fuel, labor, maintenance]
            .iter()
            .any(|v| v.minor_units() < 0)
        {
            return Err(WorldError::InvalidFreightContract(
                "operating costs cannot be negative",
            ));
        }
        Ok(Self {
            fuel_per_unit: fuel,
            labor_per_unit: labor,
            maintenance_per_unit: maintenance,
        })
    }
    #[must_use]
    pub const fn total_per_unit(self) -> Money {
        Money::from_minor_units(
            self.fuel_per_unit.minor_units()
                + self.labor_per_unit.minor_units()
                + self.maintenance_per_unit.minor_units(),
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FreightEconomics {
    pub revenue: Money,
    pub operating_cost: Money,
    pub margin: Money,
}
/// Calculates fixed-point shipment revenue and carrier operating margin.
/// # Errors
/// Returns [`WorldError`] on arithmetic overflow.
pub fn evaluate_freight_economics(
    base_tariff: Money,
    quantity: QuantityMilli,
    cost: RouteOperatingCost,
    contract: Option<&FreightContract>,
) -> Result<FreightEconomics, WorldError> {
    let discount = contract
        .filter(|c| c.status() == ContractStatus::Active)
        .map_or(0, |c| c.discount().get());
    let tariff = i128::from(base_tariff.minor_units()) * i128::from(10_000 - discount) / 10_000;
    let revenue = tariff * i128::from(quantity.get()) / i128::from(QuantityMilli::SCALE);
    let operating = i128::from(cost.total_per_unit().minor_units()) * i128::from(quantity.get())
        / i128::from(QuantityMilli::SCALE);
    let revenue =
        i64::try_from(revenue).map_err(|_| WorldError::ArithmeticOverflow("freight revenue"))?;
    let operating = i64::try_from(operating)
        .map_err(|_| WorldError::ArithmeticOverflow("freight operating cost"))?;
    Ok(FreightEconomics {
        revenue: Money::from_minor_units(revenue),
        operating_cost: Money::from_minor_units(operating),
        margin: Money::from_minor_units(revenue - operating),
    })
}
impl World {
    /// Registers a proposed freight contract after validating parties, route ownership, and capacity.
    /// # Errors
    /// Returns an error for duplicate IDs, unknown references, carrier mismatch, or excess capacity.
    pub fn register_freight_contract(
        &mut self,
        contract: FreightContract,
    ) -> Result<(), WorldError> {
        if self.freight_contracts.contains_key(&contract.id()) {
            return Err(WorldError::DuplicateFreightContract(contract.id()));
        }
        if !self.firms().contains_key(&contract.shipper()) {
            return Err(WorldError::UnknownFirm(contract.shipper()));
        }
        if !self.firms().contains_key(&contract.carrier()) {
            return Err(WorldError::UnknownFirm(contract.carrier()));
        }
        let route = self
            .logistics_routes()
            .get(&contract.route())
            .ok_or(WorldError::UnknownLogisticsRoute(contract.route()))?;
        if route.carrier() != Some(contract.carrier()) {
            return Err(WorldError::InvalidFreightContract(
                "contract carrier does not own route",
            ));
        }
        if contract.reserved_capacity().get() > route.capacity().get() {
            return Err(WorldError::InvalidFreightContract(
                "reserved capacity exceeds route capacity",
            ));
        }
        self.events.append(
            self.date,
            DomainEvent::FreightContractRegistered {
                contract: contract.id(),
                shipper: contract.shipper(),
                carrier: contract.carrier(),
                route: contract.route(),
            },
        );
        self.freight_contracts.insert(contract.id(), contract);
        Ok(())
    }
    /// Activates a proposed freight contract.
    /// # Errors
    /// Returns an error for an unknown or non-proposed contract.
    pub fn activate_freight_contract(&mut self, id: ContractId) -> Result<(), WorldError> {
        let contract = self
            .freight_contracts
            .get_mut(&id)
            .ok_or(WorldError::UnknownFreightContract(id))?;
        if contract.status() != ContractStatus::Proposed {
            return Err(WorldError::InvalidFreightContract(
                "contract is not proposed",
            ));
        }
        contract.activate();
        self.events.append(
            self.date,
            DomainEvent::FreightContractActivated { contract: id },
        );
        Ok(())
    }
    /// Stores a route's operating-cost profile.
    /// # Errors
    /// Returns an error for an unknown route.
    pub fn set_route_operating_cost(
        &mut self,
        route: RouteId,
        cost: RouteOperatingCost,
    ) -> Result<(), WorldError> {
        if !self.logistics_routes().contains_key(&route) {
            return Err(WorldError::UnknownLogisticsRoute(route));
        }
        self.route_operating_costs.insert(route, cost);
        Ok(())
    }
    pub fn expire_freight_contracts(&mut self, month: u32) {
        for contract in self.freight_contracts.values_mut() {
            if contract.expire_if_due(month) {
                self.events.append(
                    self.date,
                    DomainEvent::FreightContractExpired {
                        contract: contract.id(),
                    },
                );
            }
        }
    }
    #[must_use]
    pub fn freight_contracts(&self) -> &BTreeMap<ContractId, FreightContract> {
        &self.freight_contracts
    }
    #[must_use]
    pub fn route_operating_costs(&self) -> &BTreeMap<RouteId, RouteOperatingCost> {
        &self.route_operating_costs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contract_discount_can_turn_margin_negative() {
        let mut contract = FreightContract::new(
            ContractId::new(1),
            FirmId::new(1),
            FirmId::new(2),
            RouteId::new(1),
            QuantityMilli::new(1000),
            BasisPoints::new(5000).expect("discount"),
            1,
            12,
        )
        .expect("contract");
        contract.activate();
        let cost = RouteOperatingCost::new(
            Money::from_minor_units(4),
            Money::from_minor_units(3),
            Money::from_minor_units(1),
        )
        .expect("cost");
        let result = evaluate_freight_economics(
            Money::from_minor_units(10),
            QuantityMilli::new(1000),
            cost,
            Some(&contract),
        )
        .expect("economics");
        assert_eq!(result.revenue, Money::from_minor_units(5));
        assert_eq!(result.operating_cost, Money::from_minor_units(8));
        assert_eq!(result.margin, Money::from_minor_units(-3));
    }
}
