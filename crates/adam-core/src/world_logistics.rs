use crate::{
    ContractId, FirmId, FreightCapacityLedger, GoodId, IntermodalPhase,
    IntermodalShipmentLifecycle, LogisticsRoute, Money, MultiLegShipmentPlan, QuantityMilli,
    RouteCapacityLedger, RouteId, ShipmentId, ShipmentOrder, ShipmentStatus, ShipmentTransition,
    TerminalId, World, WorldError, plan_multileg_shipment,
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
    capacity_contracts: Vec<Option<ContractId>>,
    terminal_ids: Vec<TerminalId>,
    lifecycle: IntermodalShipmentLifecycle,
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
    pub fn capacity_contracts(&self) -> &[Option<ContractId>] {
        &self.capacity_contracts
    }
    #[must_use]
    pub fn status(&self) -> ShipmentStatus {
        if self.lifecycle.phase() == IntermodalPhase::Delivered {
            ShipmentStatus::Delivered
        } else {
            ShipmentStatus::InTransit
        }
    }
    #[must_use]
    pub fn remaining_days(&self) -> u32 {
        u32::from(self.lifecycle.remaining_days())
    }
    #[must_use]
    pub const fn phase(&self) -> IntermodalPhase {
        self.lifecycle.phase()
    }
    #[must_use]
    pub fn terminal_ids(&self) -> &[TerminalId] {
        &self.terminal_ids
    }
    pub(crate) fn begin_terminal_handling(&mut self) -> Result<(), WorldError> {
        self.lifecycle.begin_terminal_handling()
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
    #[allow(clippy::too_many_lines)]
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
        let base_routes: Vec<_> = self.logistics_routes.values().cloned().collect();
        let effective_routes: Vec<_> = base_routes
            .iter()
            .cloned()
            .map(|route| {
                let contract = self.freight_contracts().values().find(|contract| {
                    contract.status() == crate::ContractStatus::Active
                        && contract.shipper() == source
                        && contract.carrier() == route.carrier().unwrap_or(source)
                        && contract.route() == route.id()
                });
                let discount = contract.map_or(0, |value| value.discount().get());
                let tariff = i128::from(route.cost_per_unit().minor_units())
                    * i128::from(10_000 - discount)
                    / 10_000;
                route.with_cost_per_unit(Money::from_minor_units(
                    i64::try_from(tariff).unwrap_or(i64::MAX),
                ))
            })
            .collect();
        let plan: MultiLegShipmentPlan =
            plan_multileg_shipment(order, &effective_routes, max_legs)?;
        let route_map: BTreeMap<_, _> = base_routes
            .iter()
            .map(|route| (route.id(), route))
            .collect();
        let mut carrier_deltas: BTreeMap<FirmId, i64> = BTreeMap::new();
        let mut charged_total = 0_i64;
        let mut capacity_contracts = Vec::with_capacity(plan.routes.len());
        for route_id in &plan.routes {
            let route = route_map[route_id];
            let carrier = route.carrier().ok_or(WorldError::InvalidLogistics(
                "route requires a carrier firm",
            ))?;
            let contract = self.freight_contracts().values().find(|contract| {
                contract.status() == crate::ContractStatus::Active
                    && contract.shipper() == source
                    && contract.carrier() == carrier
                    && contract.route() == *route_id
            });
            capacity_contracts.push(contract.map(crate::FreightContract::id));
            let cost = self
                .route_operating_costs()
                .get(route_id)
                .copied()
                .unwrap_or_default();
            let economics = crate::evaluate_freight_economics(
                route.cost_per_unit(),
                order.quantity(),
                cost,
                contract,
            )?;
            charged_total = charged_total
                .checked_add(economics.revenue.minor_units())
                .ok_or(WorldError::ArithmeticOverflow("freight charge total"))?;
            let current = carrier_deltas.get(&carrier).copied().unwrap_or_default();
            carrier_deltas.insert(
                carrier,
                current
                    .checked_add(economics.margin.minor_units())
                    .ok_or(WorldError::ArithmeticOverflow("carrier margin total"))?,
            );
        }
        if charged_total > order.max_total_cost().minor_units() {
            return Err(WorldError::NoFeasibleLogisticsRoute(order.id()));
        }
        if self.firms()[&source].cash().minor_units() < charged_total {
            return Err(WorldError::InsufficientFirmCash(source));
        }
        for (carrier, delta) in &carrier_deltas {
            if self.firms()[carrier]
                .cash()
                .minor_units()
                .checked_add(*delta)
                .is_none_or(|value| value < 0)
            {
                return Err(WorldError::InsufficientFirmCash(*carrier));
            }
        }
        let charged_total = Money::from_minor_units(charged_total);
        let mut proposed_freight_capacity = self.freight_capacity.clone();
        for (route_id, contract_id) in plan.routes.iter().zip(&capacity_contracts) {
            let route = route_map[route_id];
            let contract = contract_id.and_then(|id| self.freight_contracts().get(&id));
            proposed_freight_capacity.reserve(
                route,
                order.quantity(),
                contract,
                self.freight_contracts(),
            )?;
        }
        let mut terminal_ids = Vec::new();
        let mut transfer_days = Vec::new();
        for pair in plan.routes.windows(2) {
            let region = route_map[&pair[0]].destination();
            let terminal = self
                .terminals()
                .values()
                .filter(|terminal| terminal.region() == region)
                .min_by_key(|terminal| terminal.id())
                .ok_or(WorldError::NoTerminalInRegion(region))?;
            terminal_ids.push(terminal.id());
            transfer_days.push(terminal.handling_days());
        }
        let lifecycle = IntermodalShipmentLifecycle::from_plan(&plan, &base_routes, transfer_days)?;
        self.route_capacity
            .reserve(&plan, order.quantity(), &base_routes)?;
        self.freight_capacity = proposed_freight_capacity;
        self.firms
            .get_mut(&source)
            .ok_or(WorldError::UnknownFirm(source))?
            .debit_inventory(order.good(), order.quantity())?;
        self.firms
            .get_mut(&source)
            .ok_or(WorldError::UnknownFirm(source))?
            .debit_cash(charged_total)?;
        for (carrier, value) in carrier_deltas {
            self.firms
                .get_mut(&carrier)
                .ok_or(WorldError::UnknownFirm(carrier))?
                .apply_cash_delta(crate::Money::from_minor_units(value))?;
        }
        self.events.append(
            self.date,
            crate::DomainEvent::ShipmentStarted {
                shipment: order.id(),
                good: order.good(),
                source,
                destination,
                quantity: order.quantity(),
                total_cost: charged_total,
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
                total_cost: charged_total,
                capacity_contracts,
                terminal_ids,
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
        let transitions = shipment.lifecycle.advance_days(days)?;
        for transition in transitions {
            match transition {
                ShipmentTransition::RouteCompleted(route) => {
                    let index = shipment
                        .lifecycle
                        .routes()
                        .iter()
                        .position(|candidate| *candidate == route)
                        .ok_or(WorldError::UnknownLogisticsRoute(route))?;
                    self.route_capacity.release(&[route], shipment.quantity)?;
                    self.freight_capacity.release(
                        route,
                        shipment.quantity,
                        shipment.capacity_contracts[index],
                    )?;
                    if shipment.lifecycle.phase() == IntermodalPhase::WaitingForTerminal {
                        let terminal = shipment.terminal_ids[index];
                        self.terminal_queue.enqueue(
                            terminal,
                            crate::TerminalQueueEntry::new(id, shipment.quantity),
                        )?;
                        self.events.append(
                            self.date,
                            crate::DomainEvent::ShipmentQueuedAtTerminal {
                                shipment: id,
                                terminal,
                            },
                        );
                    }
                }
                ShipmentTransition::TransferCompleted { after_leg } => {
                    let terminal = shipment.terminal_ids[after_leg];
                    self.terminal_capacity
                        .release(terminal, shipment.quantity)?;
                }
            }
        }
        if shipment.lifecycle.phase() == IntermodalPhase::Delivered {
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
    pub const fn freight_capacity(&self) -> &FreightCapacityLedger {
        &self.freight_capacity
    }
    /// Remaining uncontracted spot capacity per route available to immediate
    /// market imports this month: monthly capacity minus active freight
    /// contract reservations, spot freight usage, and in-transit shipment
    /// reservations.
    #[must_use]
    pub(crate) fn market_spot_route_capacity(&self) -> BTreeMap<RouteId, u64> {
        self.logistics_routes
            .values()
            .map(|route| {
                let contracted: u64 = self
                    .freight_contracts
                    .values()
                    .filter(|contract| {
                        contract.status() == crate::ContractStatus::Active
                            && contract.route() == route.id()
                    })
                    .map(|contract| contract.reserved_capacity().get())
                    .sum();
                let spot_used = self
                    .freight_capacity
                    .spot_used()
                    .get(&route.id())
                    .copied()
                    .unwrap_or_default()
                    .get();
                let in_transit = self
                    .route_capacity
                    .reserved()
                    .get(&route.id())
                    .copied()
                    .unwrap_or_default()
                    .get();
                let available = route
                    .capacity()
                    .get()
                    .saturating_sub(contracted)
                    .saturating_sub(spot_used)
                    .saturating_sub(in_transit);
                (route.id(), available)
            })
            .collect()
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
        assert!(world.freight_capacity().spot_used().is_empty());
        assert_eq!(
            world.inventory_shipments()[&ShipmentId::new(1)].status(),
            ShipmentStatus::Delivered
        );
    }
}
