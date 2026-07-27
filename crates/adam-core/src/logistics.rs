use crate::{FirmId, GoodId, Money, QuantityMilli, RegionId, RouteId, ShipmentId, WorldError};
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
    carrier: Option<FirmId>,
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
            carrier: None,
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
    #[must_use]
    pub const fn mode(&self) -> TransportMode {
        self.mode
    }
    #[must_use]
    pub const fn reliability_bps(&self) -> u16 {
        self.reliability_bps
    }
    #[must_use]
    pub const fn carrier(&self) -> Option<FirmId> {
        self.carrier
    }
    #[must_use]
    pub const fn with_carrier(mut self, carrier: FirmId) -> Self {
        self.carrier = Some(carrier);
        self
    }
    #[must_use]
    pub const fn with_cost_per_unit(mut self, cost: Money) -> Self {
        self.cost_per_unit = cost;
        self
    }
    pub(crate) fn add_capacity(&mut self, added: QuantityMilli) -> Result<(), WorldError> {
        if added.get() == 0 {
            return Err(WorldError::InvalidLogistics(
                "route capacity addition must be positive",
            ));
        }
        self.monthly_capacity = QuantityMilli::new(
            self.monthly_capacity
                .get()
                .checked_add(added.get())
                .ok_or(WorldError::ArithmeticOverflow("route capacity expansion"))?,
        );
        Ok(())
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
    #[must_use]
    pub const fn id(&self) -> ShipmentId {
        self.id
    }
    #[must_use]
    pub const fn good(&self) -> GoodId {
        self.good
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
    pub const fn quantity(&self) -> QuantityMilli {
        self.quantity
    }
    #[must_use]
    pub const fn max_total_cost(&self) -> Money {
        self.max_total_cost
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiLegShipmentPlan {
    pub shipment: ShipmentId,
    pub routes: Vec<RouteId>,
    pub total_cost: Money,
    pub arrival_days: u32,
}
/// Finds the cheapest feasible simple path of at most `max_legs`, with lexicographic route-ID tie-breaking.
/// # Errors
/// Returns [`WorldError::NoFeasibleLogisticsRoute`] when no capacity- and budget-feasible path exists.
pub fn plan_multileg_shipment(
    order: &ShipmentOrder,
    routes: &[LogisticsRoute],
    max_legs: usize,
) -> Result<MultiLegShipmentPlan, WorldError> {
    if max_legs == 0 {
        return Err(WorldError::NoFeasibleLogisticsRoute(order.id));
    }
    let mut candidates = Vec::new();
    let mut visited = std::collections::BTreeSet::from([order.origin]);
    let mut path = Vec::new();
    search_paths(
        order,
        routes,
        max_legs,
        order.origin,
        &mut visited,
        &mut path,
        0,
        0,
        &mut candidates,
    )?;
    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let (cost, ids, days) = candidates
        .into_iter()
        .next()
        .ok_or(WorldError::NoFeasibleLogisticsRoute(order.id))?;
    Ok(MultiLegShipmentPlan {
        shipment: order.id,
        routes: ids,
        total_cost: Money::from_minor_units(cost),
        arrival_days: days,
    })
}
#[allow(clippy::too_many_arguments)]
fn search_paths(
    order: &ShipmentOrder,
    routes: &[LogisticsRoute],
    max_legs: usize,
    current: RegionId,
    visited: &mut std::collections::BTreeSet<RegionId>,
    path: &mut Vec<RouteId>,
    cost: i64,
    days: u32,
    output: &mut Vec<(i64, Vec<RouteId>, u32)>,
) -> Result<(), WorldError> {
    if current == order.destination {
        if cost <= order.max_total_cost.minor_units() {
            output.push((cost, path.clone(), days));
        }
        return Ok(());
    }
    if path.len() >= max_legs {
        return Ok(());
    }
    let mut outgoing: Vec<_> = routes
        .iter()
        .filter(|r| r.origin() == current && r.capacity().get() >= order.quantity.get())
        .collect();
    outgoing.sort_by_key(|r| r.id());
    for route in outgoing {
        if visited.contains(&route.destination()) {
            continue;
        }
        let leg = i128::from(route.cost_per_unit().minor_units())
            * i128::from(order.quantity.get())
            / i128::from(QuantityMilli::SCALE);
        let leg = i64::try_from(leg)
            .map_err(|_| WorldError::ArithmeticOverflow("multi-leg shipment cost"))?;
        let next = cost
            .checked_add(leg)
            .ok_or(WorldError::ArithmeticOverflow("multi-leg shipment total"))?;
        if next > order.max_total_cost.minor_units() {
            continue;
        }
        visited.insert(route.destination());
        path.push(route.id());
        search_paths(
            order,
            routes,
            max_legs,
            route.destination(),
            visited,
            path,
            next,
            days + u32::from(route.transit_days()),
            output,
        )?;
        path.pop();
        visited.remove(&route.destination());
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RouteCapacityLedger {
    reserved: std::collections::BTreeMap<RouteId, QuantityMilli>,
}
impl RouteCapacityLedger {
    #[must_use]
    pub fn reserved(&self) -> &std::collections::BTreeMap<RouteId, QuantityMilli> {
        &self.reserved
    }
    /// Atomically reserves the same shipment quantity on every route in a plan.
    /// # Errors
    /// Returns [`WorldError::InsufficientRouteCapacity`] without partial reservations.
    pub fn reserve(
        &mut self,
        plan: &MultiLegShipmentPlan,
        quantity: QuantityMilli,
        routes: &[LogisticsRoute],
    ) -> Result<(), WorldError> {
        let by_id: std::collections::BTreeMap<_, _> = routes.iter().map(|r| (r.id(), r)).collect();
        for id in &plan.routes {
            let route = by_id
                .get(id)
                .ok_or(WorldError::UnknownLogisticsRoute(*id))?;
            let used = self.reserved.get(id).copied().unwrap_or_default().get();
            let next = used
                .checked_add(quantity.get())
                .ok_or(WorldError::ArithmeticOverflow("route reservation"))?;
            if next > route.capacity().get() {
                return Err(WorldError::InsufficientRouteCapacity(*id));
            }
        }
        for id in &plan.routes {
            let used = self.reserved.get(id).copied().unwrap_or_default().get();
            self.reserved
                .insert(*id, QuantityMilli::new(used + quantity.get()));
        }
        Ok(())
    }
    /// Releases capacity after delivery or cancellation.
    /// # Errors
    /// Returns an error when release exceeds the current reservation.
    pub fn release(
        &mut self,
        routes: &[RouteId],
        quantity: QuantityMilli,
    ) -> Result<(), WorldError> {
        for id in routes {
            let used = self.reserved.get(id).copied().unwrap_or_default().get();
            if used < quantity.get() {
                return Err(WorldError::InvalidLogistics("release exceeds reservation"));
            }
        }
        for id in routes {
            let used = self.reserved[id].get() - quantity.get();
            if used == 0 {
                self.reserved.remove(id);
            } else {
                self.reserved.insert(*id, QuantityMilli::new(used));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ShipmentStatus {
    Planned,
    Reserved,
    InTransit,
    Delivered,
    Cancelled,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ShipmentLifecycle {
    id: ShipmentId,
    routes: Vec<RouteId>,
    quantity: QuantityMilli,
    remaining_days: u32,
    status: ShipmentStatus,
}
impl ShipmentLifecycle {
    #[must_use]
    pub fn from_plan(plan: &MultiLegShipmentPlan, quantity: QuantityMilli) -> Self {
        Self {
            id: plan.shipment,
            routes: plan.routes.clone(),
            quantity,
            remaining_days: plan.arrival_days,
            status: ShipmentStatus::Planned,
        }
    }
    #[must_use]
    pub const fn status(&self) -> ShipmentStatus {
        self.status
    }
    #[must_use]
    pub const fn remaining_days(&self) -> u32 {
        self.remaining_days
    }
    #[must_use]
    pub const fn id(&self) -> ShipmentId {
        self.id
    }
    #[must_use]
    pub fn routes(&self) -> &[RouteId] {
        &self.routes
    }
    #[must_use]
    pub const fn quantity(&self) -> QuantityMilli {
        self.quantity
    }
    /// Reserves route capacity and starts transit.
    /// # Errors
    /// Returns an error for invalid status or insufficient capacity.
    pub fn start(
        &mut self,
        ledger: &mut RouteCapacityLedger,
        routes: &[LogisticsRoute],
    ) -> Result<(), WorldError> {
        if self.status != ShipmentStatus::Planned {
            return Err(WorldError::InvalidLogistics("shipment is not planned"));
        }
        ledger.reserve(
            &MultiLegShipmentPlan {
                shipment: self.id,
                routes: self.routes.clone(),
                total_cost: Money::default(),
                arrival_days: self.remaining_days,
            },
            self.quantity,
            routes,
        )?;
        self.status = ShipmentStatus::InTransit;
        Ok(())
    }
    /// Advances transit and releases capacity on delivery.
    /// # Errors
    /// Returns an error unless shipment is in transit or release fails.
    pub fn advance_days(
        &mut self,
        days: u32,
        ledger: &mut RouteCapacityLedger,
    ) -> Result<(), WorldError> {
        if self.status != ShipmentStatus::InTransit {
            return Err(WorldError::InvalidLogistics("shipment is not in transit"));
        }
        self.remaining_days = self.remaining_days.saturating_sub(days);
        if self.remaining_days == 0 {
            ledger.release(&self.routes, self.quantity)?;
            self.status = ShipmentStatus::Delivered;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LegShipmentLifecycle {
    shipment: ShipmentId,
    routes: Vec<RouteId>,
    leg_days: Vec<u16>,
    current_leg: usize,
    remaining_leg_days: u16,
    status: ShipmentStatus,
}
impl LegShipmentLifecycle {
    /// Builds per-leg transit state from a routed shipment plan.
    /// # Errors
    /// Returns an error for an empty path or missing route definition.
    pub fn from_plan(
        plan: &MultiLegShipmentPlan,
        routes: &[LogisticsRoute],
    ) -> Result<Self, WorldError> {
        if plan.routes.is_empty() {
            return Err(WorldError::InvalidLogistics("shipment path is empty"));
        }
        let by_id: std::collections::BTreeMap<_, _> = routes.iter().map(|r| (r.id(), r)).collect();
        let mut leg_days = Vec::with_capacity(plan.routes.len());
        for id in &plan.routes {
            leg_days.push(
                by_id
                    .get(id)
                    .ok_or(WorldError::UnknownLogisticsRoute(*id))?
                    .transit_days(),
            );
        }
        let remaining_leg_days = leg_days[0];
        Ok(Self {
            shipment: plan.shipment,
            routes: plan.routes.clone(),
            leg_days,
            current_leg: 0,
            remaining_leg_days,
            status: ShipmentStatus::InTransit,
        })
    }
    #[must_use]
    pub const fn status(&self) -> ShipmentStatus {
        self.status
    }
    #[must_use]
    pub const fn current_leg(&self) -> usize {
        self.current_leg
    }
    #[must_use]
    pub const fn remaining_leg_days(&self) -> u16 {
        self.remaining_leg_days
    }
    #[must_use]
    pub fn current_route(&self) -> Option<RouteId> {
        self.routes.get(self.current_leg).copied()
    }
    #[must_use]
    pub fn routes(&self) -> &[RouteId] {
        &self.routes
    }
    #[must_use]
    pub fn remaining_days(&self) -> u32 {
        if self.status == ShipmentStatus::Delivered {
            return 0;
        }
        u32::from(self.remaining_leg_days)
            + self
                .leg_days
                .iter()
                .skip(self.current_leg + 1)
                .map(|v| u32::from(*v))
                .sum::<u32>()
    }
    /// Advances across route legs and returns legs completed during this step.
    /// # Errors
    /// Returns an error unless the shipment is in transit.
    pub fn advance_days(&mut self, mut days: u32) -> Result<Vec<RouteId>, WorldError> {
        if self.status != ShipmentStatus::InTransit {
            return Err(WorldError::InvalidLogistics(
                "leg shipment is not in transit",
            ));
        }
        let mut completed = Vec::new();
        while days > 0 && self.status == ShipmentStatus::InTransit {
            let remaining = u32::from(self.remaining_leg_days);
            if days < remaining {
                self.remaining_leg_days -= u16::try_from(days)
                    .map_err(|_| WorldError::ArithmeticOverflow("leg transit days"))?;
                break;
            }
            days -= remaining;
            completed.push(self.routes[self.current_leg]);
            self.current_leg += 1;
            if self.current_leg == self.routes.len() {
                self.remaining_leg_days = 0;
                self.status = ShipmentStatus::Delivered;
            } else {
                self.remaining_leg_days = self.leg_days[self.current_leg];
            }
        }
        Ok(completed)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IntermodalPhase {
    Transit,
    WaitingForTerminal,
    TerminalHandling,
    Delivered,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipmentTransition {
    RouteCompleted(RouteId),
    TransferCompleted { after_leg: usize },
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IntermodalShipmentLifecycle {
    shipment: ShipmentId,
    routes: Vec<RouteId>,
    leg_days: Vec<u16>,
    transfer_days: Vec<u16>,
    current_leg: usize,
    remaining_days: u16,
    phase: IntermodalPhase,
}
impl IntermodalShipmentLifecycle {
    /// Creates explicit transit and terminal-handling phases.
    /// # Errors
    /// Returns an error for missing routes or a transfer-duration count different from `routes.len() - 1`.
    pub fn from_plan(
        plan: &MultiLegShipmentPlan,
        routes: &[LogisticsRoute],
        transfer_days: Vec<u16>,
    ) -> Result<Self, WorldError> {
        if plan.routes.is_empty()
            || transfer_days.len() != plan.routes.len().saturating_sub(1)
            || transfer_days.contains(&0)
        {
            return Err(WorldError::InvalidLogistics(
                "invalid intermodal transfer schedule",
            ));
        }
        let by_id: std::collections::BTreeMap<_, _> =
            routes.iter().map(|route| (route.id(), route)).collect();
        let mut leg_days = Vec::with_capacity(plan.routes.len());
        for id in &plan.routes {
            leg_days.push(
                by_id
                    .get(id)
                    .ok_or(WorldError::UnknownLogisticsRoute(*id))?
                    .transit_days(),
            );
        }
        Ok(Self {
            shipment: plan.shipment,
            routes: plan.routes.clone(),
            remaining_days: leg_days[0],
            leg_days,
            transfer_days,
            current_leg: 0,
            phase: IntermodalPhase::Transit,
        })
    }
    #[must_use]
    pub const fn phase(&self) -> IntermodalPhase {
        self.phase
    }
    #[must_use]
    pub const fn current_leg(&self) -> usize {
        self.current_leg
    }
    #[must_use]
    pub const fn remaining_days(&self) -> u16 {
        self.remaining_days
    }
    #[must_use]
    pub fn current_route(&self) -> Option<RouteId> {
        (self.phase == IntermodalPhase::Transit).then(|| self.routes[self.current_leg])
    }
    #[must_use]
    pub fn routes(&self) -> &[RouteId] {
        &self.routes
    }
    /// Starts handling after external terminal admission.
    /// # Errors
    /// Returns an error unless waiting for a terminal.
    pub fn begin_terminal_handling(&mut self) -> Result<(), WorldError> {
        if self.phase != IntermodalPhase::WaitingForTerminal {
            return Err(WorldError::InvalidLogistics(
                "shipment is not waiting for terminal",
            ));
        }
        self.remaining_days = self.transfer_days[self.current_leg];
        self.phase = IntermodalPhase::TerminalHandling;
        Ok(())
    }
    /// Advances through transit and terminal handling, reporting causal transitions.
    /// # Errors
    /// Returns an error after delivery.
    pub fn advance_days(&mut self, mut days: u32) -> Result<Vec<ShipmentTransition>, WorldError> {
        if self.phase == IntermodalPhase::Delivered {
            return Err(WorldError::InvalidLogistics(
                "intermodal shipment is delivered",
            ));
        }
        let mut transitions = Vec::new();
        while days > 0 && self.phase != IntermodalPhase::Delivered {
            let remaining = u32::from(self.remaining_days);
            if days < remaining {
                self.remaining_days -= u16::try_from(days)
                    .map_err(|_| WorldError::ArithmeticOverflow("intermodal days"))?;
                break;
            }
            days -= remaining;
            match self.phase {
                IntermodalPhase::Transit => {
                    transitions.push(ShipmentTransition::RouteCompleted(
                        self.routes[self.current_leg],
                    ));
                    if self.current_leg + 1 == self.routes.len() {
                        self.remaining_days = 0;
                        self.phase = IntermodalPhase::Delivered;
                    } else {
                        self.remaining_days = 0;
                        self.phase = IntermodalPhase::WaitingForTerminal;
                    }
                }
                IntermodalPhase::WaitingForTerminal | IntermodalPhase::Delivered => break,
                IntermodalPhase::TerminalHandling => {
                    transitions.push(ShipmentTransition::TransferCompleted {
                        after_leg: self.current_leg,
                    });
                    self.current_leg += 1;
                    self.remaining_days = self.leg_days[self.current_leg];
                    self.phase = IntermodalPhase::Transit;
                }
            }
        }
        Ok(transitions)
    }
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
    #[test]
    fn multi_leg_route_connects_regions_without_direct_service() {
        let order = ShipmentOrder::new(
            ShipmentId::new(2),
            GoodId::new(1),
            RegionId::new(1),
            RegionId::new(3),
            QuantityMilli::new(1000),
            Money::from_minor_units(100),
        );
        let routes = vec![
            LogisticsRoute::new(
                RouteId::new(1),
                RegionId::new(1),
                RegionId::new(2),
                TransportMode::Rail,
                QuantityMilli::new(5000),
                Money::from_minor_units(10),
                2,
                9500,
            )
            .expect("route"),
            LogisticsRoute::new(
                RouteId::new(2),
                RegionId::new(2),
                RegionId::new(3),
                TransportMode::Road,
                QuantityMilli::new(5000),
                Money::from_minor_units(20),
                3,
                9000,
            )
            .expect("route"),
        ];
        let plan = plan_multileg_shipment(&order, &routes, 3).expect("plan");
        assert_eq!(plan.routes, vec![RouteId::new(1), RouteId::new(2)]);
        assert_eq!(plan.arrival_days, 5);
    }
    #[test]
    fn shared_capacity_is_reserved_and_released() {
        let route = LogisticsRoute::new(
            RouteId::new(1),
            RegionId::new(1),
            RegionId::new(2),
            TransportMode::Rail,
            QuantityMilli::new(1000),
            Money::from_minor_units(10),
            2,
            9500,
        )
        .expect("route");
        let plan = MultiLegShipmentPlan {
            shipment: ShipmentId::new(3),
            routes: vec![RouteId::new(1)],
            total_cost: Money::from_minor_units(10),
            arrival_days: 2,
        };
        let mut ledger = RouteCapacityLedger::default();
        let mut shipment = ShipmentLifecycle::from_plan(&plan, QuantityMilli::new(700));
        shipment
            .start(&mut ledger, std::slice::from_ref(&route))
            .expect("start");
        assert_eq!(ledger.reserved()[&RouteId::new(1)].get(), 700);
        assert!(
            ledger
                .reserve(&plan, QuantityMilli::new(400), std::slice::from_ref(&route))
                .is_err()
        );
        shipment.advance_days(2, &mut ledger).expect("deliver");
        assert!(ledger.reserved().is_empty());
        assert_eq!(shipment.status(), ShipmentStatus::Delivered);
    }
    #[test]
    fn leg_progress_crosses_multiple_routes_exactly() {
        let routes = vec![
            LogisticsRoute::new(
                RouteId::new(1),
                RegionId::new(1),
                RegionId::new(2),
                TransportMode::Rail,
                QuantityMilli::new(1000),
                Money::from_minor_units(10),
                2,
                9500,
            )
            .expect("route"),
            LogisticsRoute::new(
                RouteId::new(2),
                RegionId::new(2),
                RegionId::new(3),
                TransportMode::Road,
                QuantityMilli::new(1000),
                Money::from_minor_units(10),
                3,
                9500,
            )
            .expect("route"),
        ];
        let plan = MultiLegShipmentPlan {
            shipment: ShipmentId::new(8),
            routes: vec![RouteId::new(1), RouteId::new(2)],
            total_cost: Money::from_minor_units(20),
            arrival_days: 5,
        };
        let mut lifecycle = LegShipmentLifecycle::from_plan(&plan, &routes).expect("lifecycle");
        assert_eq!(
            lifecycle.advance_days(3).expect("advance"),
            vec![RouteId::new(1)]
        );
        assert_eq!(lifecycle.current_route(), Some(RouteId::new(2)));
        assert_eq!(lifecycle.remaining_leg_days(), 2);
        assert_eq!(
            lifecycle.advance_days(2).expect("advance"),
            vec![RouteId::new(2)]
        );
        assert_eq!(lifecycle.status(), ShipmentStatus::Delivered);
    }
    #[test]
    fn intermodal_transfer_delays_next_route() {
        let routes = vec![
            LogisticsRoute::new(
                RouteId::new(1),
                RegionId::new(1),
                RegionId::new(2),
                TransportMode::Rail,
                QuantityMilli::new(1000),
                Money::from_minor_units(10),
                2,
                9500,
            )
            .expect("route"),
            LogisticsRoute::new(
                RouteId::new(2),
                RegionId::new(2),
                RegionId::new(3),
                TransportMode::Sea,
                QuantityMilli::new(1000),
                Money::from_minor_units(10),
                3,
                9500,
            )
            .expect("route"),
        ];
        let plan = MultiLegShipmentPlan {
            shipment: ShipmentId::new(9),
            routes: vec![RouteId::new(1), RouteId::new(2)],
            total_cost: Money::from_minor_units(20),
            arrival_days: 5,
        };
        let mut lifecycle =
            IntermodalShipmentLifecycle::from_plan(&plan, &routes, vec![1]).expect("lifecycle");
        assert_eq!(
            lifecycle.advance_days(2).expect("advance"),
            vec![ShipmentTransition::RouteCompleted(RouteId::new(1))]
        );
        assert_eq!(lifecycle.phase(), IntermodalPhase::WaitingForTerminal);
        assert!(lifecycle.advance_days(5).expect("wait").is_empty());
        lifecycle.begin_terminal_handling().expect("admit");
        assert_eq!(lifecycle.phase(), IntermodalPhase::TerminalHandling);
        assert_eq!(
            lifecycle.advance_days(1).expect("handling"),
            vec![ShipmentTransition::TransferCompleted { after_leg: 0 }]
        );
        assert_eq!(lifecycle.current_route(), Some(RouteId::new(2)));
        assert_eq!(lifecycle.remaining_days(), 3);
    }
}
