use crate::{GoodId, Money, QuantityMilli, RegionId, RouteId, ShipmentId, WorldError};
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TransportMode {
    Road,
    Rail,
    Sea,
    Air,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LogisticsRoute {
    id: RouteId,
    origin: RegionId,
    destination: RegionId,
    mode: TransportMode,
    monthly_capacity: QuantityMilli,
    cost_per_unit: Money,
    transit_days: u16,
    reliability_bps: u16,
}
impl LogisticsRoute {
    /// Creates a directed transport service route.
    /// # Errors
    /// Returns [`WorldError::InvalidLogistics`] for loops or non-positive physical/economic values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RouteId,
        origin: RegionId,
        destination: RegionId,
        mode: TransportMode,
        monthly_capacity: QuantityMilli,
        cost_per_unit: Money,
        transit_days: u16,
        reliability_bps: u16,
    ) -> Result<Self, WorldError> {
        if origin == destination
            || monthly_capacity.get() == 0
            || cost_per_unit.minor_units() <= 0
            || transit_days == 0
            || reliability_bps > 10_000
        {
            return Err(WorldError::InvalidLogistics("invalid route values"));
        }
        Ok(Self {
            id,
            origin,
            destination,
            mode,
            monthly_capacity,
            cost_per_unit,
            transit_days,
            reliability_bps,
        })
    }
    #[must_use]
    pub const fn id(&self) -> RouteId {
        self.id
    }
    #[must_use]
    pub const fn origin(&self) -> RegionId {
        self.origin
    }
    #[must_use]
    pub const fn destination(&self) -> RegionId {
        self.destination
    }
    #[must_use]
    pub const fn capacity(&self) -> QuantityMilli {
        self.monthly_capacity
    }
    #[must_use]
    pub const fn cost_per_unit(&self) -> Money {
        self.cost_per_unit
    }
    #[must_use]
    pub const fn transit_days(&self) -> u16 {
        self.transit_days
    }
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShipmentOrder {
    id: ShipmentId,
    good: GoodId,
    origin: RegionId,
    destination: RegionId,
    quantity: QuantityMilli,
    max_total_cost: Money,
}
impl ShipmentOrder {
    #[must_use]
    pub const fn new(
        id: ShipmentId,
        good: GoodId,
        origin: RegionId,
        destination: RegionId,
        quantity: QuantityMilli,
        max_total_cost: Money,
    ) -> Self {
        Self {
            id,
            good,
            origin,
            destination,
            quantity,
            max_total_cost,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentPlan {
    pub shipment: ShipmentId,
    pub route: RouteId,
    pub total_cost: Money,
    pub arrival_days: u16,
}
/// Chooses the cheapest feasible direct route, breaking equal costs by route ID.
/// # Errors
/// Returns [`WorldError::NoFeasibleLogisticsRoute`] when capacity, direction, or budget cannot satisfy the order.
pub fn plan_direct_shipment(
    order: &ShipmentOrder,
    routes: &[LogisticsRoute],
) -> Result<ShipmentPlan, WorldError> {
    let mut candidates = Vec::new();
    for route in routes {
        if route.origin() != order.origin
            || route.destination() != order.destination
            || route.capacity().get() < order.quantity.get()
        {
            continue;
        }
        let cost = i128::from(route.cost_per_unit().minor_units())
            * i128::from(order.quantity.get())
            / i128::from(QuantityMilli::SCALE);
        let cost =
            i64::try_from(cost).map_err(|_| WorldError::ArithmeticOverflow("shipment cost"))?;
        if cost <= order.max_total_cost.minor_units() {
            candidates.push((cost, route.id(), route.transit_days()));
        }
    }
    candidates.sort_by_key(|(cost, id, _)| (*cost, *id));
    let (cost, route, days) = candidates
        .first()
        .copied()
        .ok_or(WorldError::NoFeasibleLogisticsRoute(order.id))?;
    Ok(ShipmentPlan {
        shipment: order.id,
        route,
        total_cost: Money::from_minor_units(cost),
        arrival_days: days,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cheapest_feasible_route_wins() {
        let order = ShipmentOrder::new(
            ShipmentId::new(1),
            GoodId::new(1),
            RegionId::new(1),
            RegionId::new(2),
            QuantityMilli::new(10_000),
            Money::from_minor_units(1000),
        );
        let routes = vec![
            LogisticsRoute::new(
                RouteId::new(2),
                RegionId::new(1),
                RegionId::new(2),
                TransportMode::Road,
                QuantityMilli::new(20_000),
                Money::from_minor_units(20),
                2,
                9000,
            )
            .expect("route"),
            LogisticsRoute::new(
                RouteId::new(1),
                RegionId::new(1),
                RegionId::new(2),
                TransportMode::Rail,
                QuantityMilli::new(20_000),
                Money::from_minor_units(10),
                4,
                9500,
            )
            .expect("route"),
        ];
        assert_eq!(
            plan_direct_shipment(&order, &routes).expect("plan").route,
            RouteId::new(1)
        );
    }
}
