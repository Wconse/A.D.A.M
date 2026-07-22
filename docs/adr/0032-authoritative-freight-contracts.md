# ADR 0032: Authoritative freight contracts

- Status: Accepted foundation
- Date: 2026-07-22

Freight contracts and route operating costs are authoritative World state. Registration validates parties, route ownership, and capacity. Activation and expiry are explicit lifecycle transitions with events. Contracts, costs, commands, saves, replay, and fingerprints share the same deterministic state. Applying reserved capacity and contract tariffs to shipment planning remains the next integration step.
