use crate::{FirmId, Money, QuantityMilli, RegionId, TerminalId, World, WorldError};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LogisticsTerminal {
    id: TerminalId,
    region: RegionId,
    operator: FirmId,
    daily_handling_capacity: QuantityMilli,
    storage_cost_per_unit_day: Money,
    base_handling_days: u16,
}
impl LogisticsTerminal {
    /// Creates a physical transshipment terminal.
    /// # Errors
    /// Returns an error for zero capacity/duration or negative storage cost.
    pub fn new(
        id: TerminalId,
        region: RegionId,
        operator: FirmId,
        capacity: QuantityMilli,
        storage_cost: Money,
        handling_days: u16,
    ) -> Result<Self, WorldError> {
        if capacity.get() == 0 || storage_cost.minor_units() < 0 || handling_days == 0 {
            return Err(WorldError::InvalidTerminal("invalid terminal values"));
        }
        Ok(Self {
            id,
            region,
            operator,
            daily_handling_capacity: capacity,
            storage_cost_per_unit_day: storage_cost,
            base_handling_days: handling_days,
        })
    }
    #[must_use]
    pub const fn id(&self) -> TerminalId {
        self.id
    }
    #[must_use]
    pub const fn region(&self) -> RegionId {
        self.region
    }
    #[must_use]
    pub const fn operator(&self) -> FirmId {
        self.operator
    }
    #[must_use]
    pub const fn capacity(&self) -> QuantityMilli {
        self.daily_handling_capacity
    }
    #[must_use]
    pub const fn storage_cost(&self) -> Money {
        self.storage_cost_per_unit_day
    }
    #[must_use]
    pub const fn handling_days(&self) -> u16 {
        self.base_handling_days
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TerminalCapacityLedger {
    used: BTreeMap<TerminalId, QuantityMilli>,
}
impl TerminalCapacityLedger {
    #[must_use]
    pub fn used(&self) -> &BTreeMap<TerminalId, QuantityMilli> {
        &self.used
    }
    /// Reserves terminal handling throughput.
    /// # Errors
    /// Returns an error without mutation when throughput is exhausted.
    pub fn reserve(
        &mut self,
        terminal: &LogisticsTerminal,
        quantity: QuantityMilli,
    ) -> Result<(), WorldError> {
        let used = self
            .used
            .get(&terminal.id())
            .copied()
            .unwrap_or_default()
            .get();
        let next = used
            .checked_add(quantity.get())
            .ok_or(WorldError::ArithmeticOverflow("terminal capacity"))?;
        if next > terminal.capacity().get() {
            return Err(WorldError::InsufficientTerminalCapacity(terminal.id()));
        }
        self.used.insert(terminal.id(), QuantityMilli::new(next));
        Ok(())
    }
    /// Releases terminal throughput.
    /// # Errors
    /// Returns an error for excessive release.
    pub fn release(&mut self, id: TerminalId, quantity: QuantityMilli) -> Result<(), WorldError> {
        let used = self.used.get(&id).copied().unwrap_or_default().get();
        if used < quantity.get() {
            return Err(WorldError::InvalidTerminal(
                "release exceeds terminal usage",
            ));
        }
        let next = used - quantity.get();
        if next == 0 {
            self.used.remove(&id);
        } else {
            self.used.insert(id, QuantityMilli::new(next));
        }
        Ok(())
    }
}
impl World {
    /// Registers a terminal after validating region and operator.
    /// # Errors
    /// Returns an error for duplicates or unknown references.
    pub fn register_terminal(&mut self, terminal: LogisticsTerminal) -> Result<(), WorldError> {
        if self.terminals.contains_key(&terminal.id()) {
            return Err(WorldError::DuplicateTerminal(terminal.id()));
        }
        if !self.regions().contains_key(&terminal.region()) {
            return Err(WorldError::UnknownRegion(terminal.region()));
        }
        if !self.firms().contains_key(&terminal.operator()) {
            return Err(WorldError::UnknownFirm(terminal.operator()));
        }
        self.terminals.insert(terminal.id(), terminal);
        Ok(())
    }
    #[must_use]
    pub fn terminals(&self) -> &BTreeMap<TerminalId, LogisticsTerminal> {
        &self.terminals
    }
    #[must_use]
    pub const fn terminal_capacity(&self) -> &TerminalCapacityLedger {
        &self.terminal_capacity
    }
    /// Queues an existing shipment at a terminal.
    /// # Errors
    /// Returns an error for unknown references or duplicate queue membership.
    pub fn enqueue_terminal_shipment(
        &mut self,
        terminal: TerminalId,
        shipment: crate::ShipmentId,
    ) -> Result<(), WorldError> {
        if !self.terminals.contains_key(&terminal) {
            return Err(WorldError::UnknownTerminal(terminal));
        }
        let quantity = self
            .inventory_shipments()
            .get(&shipment)
            .ok_or(WorldError::UnknownShipment(shipment))?
            .quantity();
        self.terminal_queue
            .enqueue(terminal, TerminalQueueEntry::new(shipment, quantity))?;
        self.events.append(
            self.date,
            crate::DomainEvent::ShipmentQueuedAtTerminal { shipment, terminal },
        );
        Ok(())
    }
    /// Admits FIFO shipments against shared terminal capacity.
    /// # Errors
    /// Returns an error for an unknown terminal.
    pub fn admit_terminal_shipments(
        &mut self,
        terminal: TerminalId,
    ) -> Result<Vec<crate::ShipmentId>, WorldError> {
        let definition = self
            .terminals
            .get(&terminal)
            .cloned()
            .ok_or(WorldError::UnknownTerminal(terminal))?;
        let admitted = self
            .terminal_queue
            .admit(&definition, &mut self.terminal_capacity);
        let ids = admitted
            .iter()
            .map(|entry| entry.shipment())
            .collect::<Vec<_>>();
        for shipment in &ids {
            self.inventory_shipments
                .get_mut(shipment)
                .ok_or(WorldError::UnknownShipment(*shipment))?
                .begin_terminal_handling()?;
            self.events.append(
                self.date,
                crate::DomainEvent::ShipmentAdmittedToTerminal {
                    shipment: *shipment,
                    terminal,
                },
            );
        }
        Ok(ids)
    }
    #[must_use]
    pub const fn terminal_queue(&self) -> &TerminalQueue {
        &self.terminal_queue
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TerminalQueueEntry {
    shipment: crate::ShipmentId,
    quantity: QuantityMilli,
}
impl TerminalQueueEntry {
    #[must_use]
    pub const fn new(shipment: crate::ShipmentId, quantity: QuantityMilli) -> Self {
        Self { shipment, quantity }
    }
    #[must_use]
    pub const fn shipment(self) -> crate::ShipmentId {
        self.shipment
    }
    #[must_use]
    pub const fn quantity(self) -> QuantityMilli {
        self.quantity
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TerminalQueue {
    waiting: BTreeMap<TerminalId, Vec<TerminalQueueEntry>>,
}
impl TerminalQueue {
    #[must_use]
    pub fn waiting(&self) -> &BTreeMap<TerminalId, Vec<TerminalQueueEntry>> {
        &self.waiting
    }
    /// Appends a shipment to a terminal's deterministic FIFO queue.
    /// # Errors
    /// Returns an error for zero quantity or a shipment already queued anywhere.
    pub fn enqueue(
        &mut self,
        terminal: TerminalId,
        entry: TerminalQueueEntry,
    ) -> Result<(), WorldError> {
        if entry.quantity().get() == 0
            || self
                .waiting
                .values()
                .flatten()
                .any(|queued| queued.shipment() == entry.shipment())
        {
            return Err(WorldError::InvalidTerminal(
                "invalid or duplicate queued shipment",
            ));
        }
        self.waiting.entry(terminal).or_default().push(entry);
        Ok(())
    }
    /// Admits as many FIFO entries as fit, stopping when the head cannot fit.
    pub fn admit(
        &mut self,
        terminal: &LogisticsTerminal,
        capacity: &mut TerminalCapacityLedger,
    ) -> Vec<TerminalQueueEntry> {
        let mut admitted = Vec::new();
        loop {
            let Some(entry) = self
                .waiting
                .get(&terminal.id())
                .and_then(|queue| queue.first())
                .copied()
            else {
                break;
            };
            if capacity.reserve(terminal, entry.quantity()).is_err() {
                break;
            }
            let Some(queue) = self.waiting.get_mut(&terminal.id()) else {
                break;
            };
            let entry = queue.remove(0);
            admitted.push(entry);
        }
        if self.waiting.get(&terminal.id()).is_some_and(Vec::is_empty) {
            self.waiting.remove(&terminal.id());
        }
        admitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_capacity_is_shared_and_released() {
        let terminal = LogisticsTerminal::new(
            TerminalId::new(1),
            RegionId::new(1),
            FirmId::new(1),
            QuantityMilli::new(1000),
            Money::from_minor_units(2),
            1,
        )
        .expect("terminal");
        let mut ledger = TerminalCapacityLedger::default();
        ledger
            .reserve(&terminal, QuantityMilli::new(700))
            .expect("reserve");
        assert!(ledger.reserve(&terminal, QuantityMilli::new(301)).is_err());
        ledger
            .release(terminal.id(), QuantityMilli::new(700))
            .expect("release");
        assert!(ledger.used().is_empty());
    }
    #[test]
    fn terminal_queue_is_fifo_and_head_blocks_later_cargo() {
        let terminal = LogisticsTerminal::new(
            TerminalId::new(1),
            RegionId::new(1),
            FirmId::new(1),
            QuantityMilli::new(1000),
            Money::from_minor_units(1),
            1,
        )
        .expect("terminal");
        let mut queue = TerminalQueue::default();
        queue
            .enqueue(
                terminal.id(),
                TerminalQueueEntry::new(crate::ShipmentId::new(1), QuantityMilli::new(700)),
            )
            .expect("queue");
        queue
            .enqueue(
                terminal.id(),
                TerminalQueueEntry::new(crate::ShipmentId::new(2), QuantityMilli::new(400)),
            )
            .expect("queue");
        let mut capacity = TerminalCapacityLedger::default();
        let admitted = queue.admit(&terminal, &mut capacity);
        assert_eq!(
            admitted
                .iter()
                .map(|entry| entry.shipment())
                .collect::<Vec<_>>(),
            vec![crate::ShipmentId::new(1)]
        );
        assert_eq!(
            queue.waiting()[&terminal.id()][0].shipment(),
            crate::ShipmentId::new(2)
        );
    }
}
