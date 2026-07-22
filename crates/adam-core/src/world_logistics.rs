use crate::{
    FirmId, GoodId, LogisticsRoute, MultiLegShipmentPlan, QuantityMilli, RouteCapacityLedger,
    RouteId, ShipmentId, ShipmentLifecycle, ShipmentOrder, ShipmentStatus, World, WorldError,
    plan_multileg_shipment,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InventoryShipment {
    id: ShipmentId,
    good: GoodId,
    source: FirmId,
    destination: FirmId,
    quantity: QuantityMilli,
    total_cost: crate::Money,
    lifecycle: ShipmentLifecycle,
}
impl InventoryShipment {
    #[must_use]
    pub const fn id(&self) -> ShipmentId {
        self.id
    }
    #[must_use]
    pub const fn good(&self) -> GoodId {
        self.good
    }
    #[must_use]
    pub const fn source(&self) -> FirmId {
        self.source
    }
    #[must_use]
    pub const fn destination(&self) -> FirmId {
        self.destination
    }
    #[must_use]
    pub const fn quantity(&self) -> QuantityMilli {
        self.quantity
    }
    #[must_use]
    pub const fn total_cost(&self) -> crate::Money {
        self.total_cost
    }
    #[must_use]
    pub const fn status(&self) -> ShipmentStatus {
        self.lifecycle.status()
    }
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.lifecycle.remaining_days()
    }
    #[must_use]
    pub fn routes(&self) -> &[RouteId] {
        self.lifecycle.routes()
    }
}
impl World {
    /// Registers a directed route after validating both regions.
    /// # Errors
    /// Returns an error for duplicates or unknown regions.
    pub fn register_logistics_route(&mut self, route: LogisticsRoute) -> Result<(), WorldError> {
        if self.logistics_routes.contains_key(&route.id()) {
            return Err(WorldError::DuplicateLogisticsRoute(route.id()));
        }
        if !self.regions().contains_key(&route.origin()) {
            return Err(WorldError::UnknownRegion(route.origin()));
        }
        if !self.regions().contains_key(&route.destination()) {
            return Err(WorldError::UnknownRegion(route.destination()));
        }
        let carrier = route.carrier().ok_or(WorldError::InvalidLogistics(
            "route requires a carrier firm",
        ))?;
        if !self.firms().contains_key(&carrier) {
            return Err(WorldError::UnknownFirm(carrier));
        }
        self.logistics_routes.insert(route.id(), route);
        Ok(())
    }
    /// Plans, reserves, and starts a firm-to-firm inventory shipment atomically.
    /// # Errors
    /// Returns an error for invalid references, inventory, routing, capacity, or duplicate IDs.
    pub fn start_inventory_shipment(
        &mut self,
        source: FirmId,
        destination: FirmId,
        order: &ShipmentOrder,
        max_legs: usize,
    ) -> Result<(), WorldError> {
        if self.inventory_shipments.contains_key(&order.id()) {
            return Err(WorldError::DuplicateShipment(order.id()));
        }
        let source_region = self
            .firms()
            .get(&source)
            .ok_or(WorldError::UnknownFirm(source))?
            .region();
        let destination_region = self
            .firms()
            .get(&destination)
            .ok_or(WorldError::UnknownFirm(destination))?
            .region();
        if source_region != order.origin() || destination_region != order.destination() {
            return Err(WorldError::InvalidLogistics(
                "shipment regions do not match firms",
            ));
        }
        if !self.goods().contains_key(&order.good()) {
            return Err(WorldError::UnknownGood(order.good()));
        }
        let available = self.firms()[&source]
            .inventories()
            .get(&order.good())
            .copied()
            .unwrap_or_default();
        if available.get() < order.quantity().get() {
            return Err(WorldError::InsufficientFirmInventory {
                firm: source,
                good: order.good(),
            });
        }
        let routes: Vec<_> = self.logistics_routes.values().cloned().collect();
        let plan: MultiLegShipmentPlan = plan_multileg_shipment(order, &routes, max_legs)?;
        let route_map: BTreeMap<_, _> = routes.iter().map(|route| (route.id(), route)).collect();
        let mut carrier_payments: BTreeMap<FirmId, i64> = BTreeMap::new();
        for route_id in &plan.routes {
            let route = route_map[route_id];
            let carrier = route.carrier().ok_or(WorldError::InvalidLogistics(
                "route requires a carrier firm",
            ))?;
            let cost = i128::from(route.cost_per_unit().minor_units())
                * i128::from(order.quantity().get())
                / i128::from(QuantityMilli::SCALE);
            let cost = i64::try_from(cost)
                .map_err(|_| WorldError::ArithmeticOverflow("carrier payment"))?;
            let current = carrier_payments.get(&carrier).copied().unwrap_or_default();
            carrier_payments.insert(
                carrier,
                current
                    .checked_add(cost)
                    .ok_or(WorldError::ArithmeticOverflow("carrier payment total"))?,
            );
        }
        if self.firms()[&source].cash().minor_units() < plan.total_cost.minor_units() {
            return Err(WorldError::InsufficientFirmCash(source));
        }
        let mut lifecycle = ShipmentLifecycle::from_plan(&plan, order.quantity());
        lifecycle.start(&mut self.route_capacity, &routes)?;
        self.firms
            .get_mut(&source)
            .ok_or(WorldError::UnknownFirm(source))?
            .debit_inventory(order.good(), order.quantity())?;
        self.firms
            .get_mut(&source)
            .ok_or(WorldError::UnknownFirm(source))?
            .debit_cash(plan.total_cost)?;
        for (carrier, value) in carrier_payments {
            self.firms
                .get_mut(&carrier)
                .ok_or(WorldError::UnknownFirm(carrier))?
                .credit_cash(crate::Money::from_minor_units(value))?;
        }
        self.events.append(
            self.date,
            crate::DomainEvent::ShipmentStarted {
                shipment: order.id(),
                good: order.good(),
                source,
                destination,
                quantity: order.quantity(),
                total_cost: plan.total_cost,
            },
        );
        self.inventory_shipments.insert(
            order.id(),
            InventoryShipment {
                id: order.id(),
                good: order.good(),
                source,
                destination,
                quantity: order.quantity(),
                total_cost: plan.total_cost,
                lifecycle,
            },
        );
        Ok(())
    }
    /// Advances a shipment and credits destination inventory exactly once on delivery.
    /// # Errors
    /// Returns an error for unknown shipment, invalid status, or inventory overflow.
    pub fn advance_inventory_shipment(
        &mut self,
        id: ShipmentId,
        days: u32,
    ) -> Result<(), WorldError> {
        let shipment = self
            .inventory_shipments
            .get_mut(&id)
            .ok_or(WorldError::UnknownShipment(id))?;
        shipment
            .lifecycle
            .advance_days(days, &mut self.route_capacity)?;
        if shipment.lifecycle.status() == ShipmentStatus::Delivered {
            self.firms
                .get_mut(&shipment.destination)
                .ok_or(WorldError::UnknownFirm(shipment.destination))?
                .credit_inventory(shipment.good, shipment.quantity)?;
            self.events.append(
                self.date,
                crate::DomainEvent::ShipmentDelivered {
                    shipment: id,
                    good: shipment.good,
                    destination: shipment.destination,
                    quantity: shipment.quantity,
                },
            );
        }
        Ok(())
    }
    #[must_use]
    pub fn logistics_routes(&self) -> &BTreeMap<RouteId, LogisticsRoute> {
        &self.logistics_routes
    }
    #[must_use]
    pub const fn route_capacity(&self) -> &RouteCapacityLedger {
        &self.route_capacity
    }
    #[must_use]
    pub fn inventory_shipments(&self) -> &BTreeMap<ShipmentId, InventoryShipment> {
        &self.inventory_shipments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Country, CountryId, Firm, Good, Money, Population, ProductionRecipe, QuantityMilli,
        RecipeId, Region, RegionId, SimDate, TransportMode, WorldSeed,
    };
    use std::collections::BTreeMap;
    #[test]
    #[allow(clippy::too_many_lines)]
    fn shipment_moves_inventory_and_releases_capacity() {
        let mut world = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("country");
        for id in [1, 2] {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        format!("R{id}"),
                        Population::new(1),
                        Money::from_minor_units(1),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
            .register_good(Good::new(GoodId::new(1), "Parts").expect("good"))
            .expect("good");
        world
            .register_production_recipe(
                ProductionRecipe::new(
                    RecipeId::new(1),
                    "Parts recipe",
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
                    "Sender",
                    RegionId::new(1),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(100),
                    BTreeMap::from([(GoodId::new(1), QuantityMilli::new(1000))]),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .register_firm(
                Firm::new(
                    FirmId::new(2),
                    "Receiver",
                    RegionId::new(2),
                    RecipeId::new(1),
                    1,
                    1,
                    Money::from_minor_units(100),
                    BTreeMap::new(),
                )
                .expect("firm"),
            )
            .expect("firm");
        world
            .register_logistics_route(
                LogisticsRoute::new(
                    RouteId::new(1),
                    RegionId::new(1),
                    RegionId::new(2),
                    TransportMode::Road,
                    QuantityMilli::new(1000),
                    Money::from_minor_units(10),
                    2,
                    9500,
                )
                .expect("route")
                .with_carrier(FirmId::new(2)),
            )
            .expect("route");
        let order = ShipmentOrder::new(
            ShipmentId::new(1),
            GoodId::new(1),
            RegionId::new(1),
            RegionId::new(2),
            QuantityMilli::new(400),
            Money::from_minor_units(100),
        );
        world
            .start_inventory_shipment(FirmId::new(1), FirmId::new(2), &order, 2)
            .expect("start");
        assert_eq!(
            world.firms()[&FirmId::new(1)].inventories()[&GoodId::new(1)].get(),
            600
        );
        assert_eq!(world.firms()[&FirmId::new(1)].cash().minor_units(), 96);
        assert_eq!(world.firms()[&FirmId::new(2)].cash().minor_units(), 104);
        assert_eq!(
            world.route_capacity().reserved()[&RouteId::new(1)].get(),
            400
        );
        world
            .advance_inventory_shipment(ShipmentId::new(1), 2)
            .expect("deliver");
        assert_eq!(
            world.firms()[&FirmId::new(2)].inventories()[&GoodId::new(1)].get(),
            400
        );
        assert!(world.route_capacity().reserved().is_empty());
        assert_eq!(
            world.inventory_shipments()[&ShipmentId::new(1)].status(),
            ShipmentStatus::Delivered
        );
    }
}
