use crate::{BasisPoints, ContractId, FirmId, Money, QuantityMilli, RouteId, WorldError};
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
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
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
