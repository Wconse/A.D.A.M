# ADR 0034: Contract and spot capacity pools

- Status: Accepted foundation
- Date: 2026-07-22

Active freight contracts remove their guaranteed capacity from the route's spot pool. Contract shipments consume only their agreement's ledger; spot shipments consume only uncontracted route capacity. Reservations and releases are explicit and cannot cross pools. Integration with authoritative shipment start/finish is the next step.
